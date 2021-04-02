use teloxide::prelude::*;
use teloxide::RequestError;

use crate::cache::Cache;
use crate::commands::Context;

#[tracing::instrument(name = "commands::health::status", skip(cx))]
pub(crate) async fn status(cx: &Context) -> anyhow::Result<Message, RequestError> {
    let resp: String;

    if Cache::status().await.is_healthy() {
        resp = String::from("Cache: healthy");
    } else {
        resp = String::from("Cache: unhealthy");
    }

    cx.answer(resp).await
}
