use anyhow::Error;
use teloxide::prelude::*;

use crate::commands::Context;
use crate::ddg::{self, DuckDuckGoError};

#[tracing::instrument(name = "commands::what", skip(cx))]
pub(crate) async fn what(cx: &Context, query: &str) -> anyhow::Result<Message, Error> {
    match ddg::Client::wiki_lookup(query).await {
        Ok(resp) => cx.reply_to(resp).await.map_err(|e| e.into()),
        Err(err) => {
            if matches!(err, DuckDuckGoError::EmptyResponse) {
                cx.reply_to("I don't know 🤔").await.map_err(|e| e.into())
            } else {
                log::error!("DuckDuckGo error: {}", err);
                cx.reply_to("Something went wrong...")
                    .await
                    .map_err(|e| e.into())
            }
        }
    }
}
