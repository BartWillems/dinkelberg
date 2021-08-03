use std::fmt::{Debug, Display};
use std::sync::atomic::{AtomicUsize, Ordering};

use deadpool_redis::{redis::cmd, ConnectionWrapper, Pool};
use redis::RedisError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::config::Config;

lazy_static! {
    static ref STATS: Stats = Stats::new();
}

pub trait CacheIdentifier {
    fn cache_key<T: Display>(id: T) -> String;
}

pub struct Cache {
    pool: Option<Pool>,
    ttl: usize,
}

pub struct Stats {
    cache_hit: AtomicUsize,
    cache_miss: AtomicUsize,
}

impl Stats {
    fn new() -> Self {
        Self {
            cache_hit: AtomicUsize::default(),
            cache_miss: AtomicUsize::default(),
        }
    }
    fn cache_hit() {
        STATS.cache_hit.fetch_add(1, Ordering::Relaxed);
    }
    fn cache_miss() {
        STATS.cache_miss.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Serialize, Debug)]
pub struct CacheStatus {
    /// is true when the redis url is set and is a valid url
    enabled: bool,
    /// is true when the cache is enabled and a connection can be retrieved
    healthy: bool,
}

impl CacheStatus {
    pub(crate) fn is_healthy(&self) -> bool {
        self.healthy
    }
}

lazy_static! {
    static ref CACHE_POOL: RwLock<Cache> = RwLock::new(Cache::new());
}

impl Cache {
    fn default() -> Self {
        Cache {
            pool: None,
            ttl: 3600 * 12,
        }
    }

    /// create a new cache object, this ignores all errors to make sure the cache doesn't break the application
    fn new() -> Self {
        info!("creating cache pool");
        let mut cache_pool = Cache::default();
        let redis_url = match Config::redis_url() {
            Some(redis_url) => redis_url,
            None => {
                info!("cache pool not initialising due to missing `REDIS_URL`");
                return cache_pool;
            }
        };

        let mut cfg = deadpool_redis::Config::default();
        cfg.url = Some(redis_url.to_owned());
        // Should be removed in a PR...
        cfg.connection = None;

        match cfg.create_pool() {
            Ok(pool) => {
                cache_pool.pool = Some(pool);
            }
            Err(err) => {
                error!("unable to initiate cache pool: {}", err);
            }
        };

        cache_pool
    }

    pub(crate) fn init() {
        info!("initializing redis cache");
        lazy_static::initialize(&CACHE_POOL);
    }

    /// returns true if the cache is initialized and ready for usage
    pub(crate) async fn is_enabled() -> bool {
        let cache = CACHE_POOL.read().await;
        cache.pool.is_some()
    }

    #[tracing::instrument]
    async fn connection() -> Option<ConnectionWrapper> {
        let cache = CACHE_POOL.read().await;

        match cache.pool.as_ref()?.get().await {
            Ok(connection) => Some(connection),
            Err(err) => {
                error!("unable to get cache connection: {}", err);
                None
            }
        }
    }

    #[tracing::instrument(name = "cache::get")]
    pub(crate) async fn get<T: DeserializeOwned, I: Display + Debug>(id: I) -> Option<T> {
        let mut conn = Cache::connection().await?;
        let cache_key = format!("{}.{}", std::any::type_name::<T>(), id);

        let res: Result<Vec<u8>, RedisError> =
            cmd("GET").arg(&cache_key).query_async(&mut conn).await;

        match res {
            Ok(res) => {
                let cache_hit = serde_json::from_slice::<T>(&res).ok();

                if cache_hit.is_some() {
                    Stats::cache_hit();
                    debug!("found {} in cache", &cache_key);
                } else {
                    Stats::cache_miss();
                }

                cache_hit
            }
            Err(err) => {
                error!("unable to fetch {} from cache: {}", &cache_key, err);
                None
            }
        }
    }

    /// Store an item in the cache that expires after a while
    /// The expiry time is configured in the cache pool
    #[tracing::instrument(name = "cache::setex", skip(object))]
    pub(crate) async fn setex<T: Serialize, I: Display + Debug>(object: &T, id: I) {
        let mut conn = match Cache::connection().await {
            Some(conn) => conn,
            None => return,
        };

        let cache_key = format!("{}.{}", std::any::type_name::<T>(), id);

        let object_string = match serde_json::to_vec(object) {
            Ok(res) => res,
            Err(err) => {
                error!("unable to serialize object for cache {}", err);
                return;
            }
        };

        let ttl = CACHE_POOL.read().await.ttl;

        let res = cmd("SETEX")
            .arg(cache_key)
            .arg(ttl)
            .arg(object_string)
            .query_async::<_, ()>(&mut conn)
            .await;

        if let Err(err) = res {
            error!("unable to store object in cache: {}", err);
        }
    }

    fn scoped_key<S: Display>(scope: S, key: &str) -> String {
        format!("scope.{}.{}", scope, key)
    }

    /// Store an item in cache, scoped behind an identifier
    #[tracing::instrument(name = "cache::set_scoped", skip(object))]
    pub(crate) async fn set_scoped<T: Serialize, S: Display + Debug>(object: &T, scope: S) {
        let mut conn = match Cache::connection().await {
            Some(conn) => conn,
            None => return,
        };

        let cache_key = Cache::scoped_key(scope, std::any::type_name::<T>());

        let object_string = match serde_json::to_vec(object) {
            Ok(res) => res,
            Err(err) => {
                error!("unable to serialize object for cache {}", err);
                return;
            }
        };

        let res = cmd("SET")
            .arg(cache_key)
            .arg(object_string)
            .query_async::<_, ()>(&mut conn)
            .await;

        if let Err(err) = res {
            error!("unable to store object in cache: {}", err);
        }
    }

    #[tracing::instrument(name = "cache::get_scoped")]
    pub(crate) async fn get_scoped<T: DeserializeOwned, S: Display + Debug>(scope: S) -> Option<T> {
        let mut conn = Cache::connection().await?;
        let cache_key = Cache::scoped_key(scope, std::any::type_name::<T>());

        let res: Result<Vec<u8>, RedisError> =
            cmd("GET").arg(&cache_key).query_async(&mut conn).await;

        match res {
            Ok(res) => serde_json::from_slice::<T>(&res).ok(),
            Err(err) => {
                error!("unable to fetch {} from cache: {}", &cache_key, err);
                None
            }
        }
    }

    #[allow(dead_code)]
    #[tracing::instrument(name = "cache::delete")]
    pub(crate) async fn delete(cache_key: String) {
        let mut conn = match Cache::connection().await {
            Some(conn) => conn,
            None => return,
        };

        let res = cmd("DEL")
            .arg(&cache_key)
            .query_async::<_, ()>(&mut conn)
            .await;

        if let Err(err) = res {
            error!("unable to delete object from cache: {}", err);
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn disable_cache() {
        let mut cache = CACHE_POOL.write().await;

        cache.pool = None;
    }

    #[allow(dead_code)]
    pub(crate) async fn enable_cache() {
        let mut cache = CACHE_POOL.write().await;

        *cache = Cache::new();
    }

    pub(crate) async fn status() -> CacheStatus {
        let enabled = Cache::is_enabled().await;
        let mut healthy = true;
        if enabled {
            healthy = Cache::connection().await.is_some();
        }
        CacheStatus { enabled, healthy }
    }
}
