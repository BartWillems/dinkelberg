use std::sync::Arc;

use chrono::{Local, NaiveDateTime};
use date_time_parser::{DateParser, TimeParser};
use teloxide::prelude::*;
use teloxide::RequestError;

use crate::commands::Context;

#[tracing::instrument(name = "commands::remind_me", skip(cx))]
pub(crate) async fn remind_me(cx: Arc<Context>, query: String) -> anyhow::Result<(), RequestError> {
    let date = DateParser::parse(&query);
    let time = TimeParser::parse(&query);

    let now = Local::now().naive_utc();

    let deadline: chrono::Duration = match (date, time) {
        (None, None) => {
            cx.reply_to("No date or time found").await?;
            return Ok(());
        }
        (Some(date), Some(time)) => {
            debug!(
                "Reminder found with date: ({:?}) and time: ({:?})",
                date, time
            );
            NaiveDateTime::new(date, time).signed_duration_since(now)
        }
        (Some(date), None) => {
            debug!("Reminder found with only a date: {:?}", date);
            date.signed_duration_since(now.date())
        }
        (None, Some(time)) => {
            debug!("Reminder found with only a time: {:?}", time);
            time.signed_duration_since(now.time())
        }
    };

    debug!("Reminder created with deadline: {:?}", deadline);

    let respond_cx = cx.clone();
    tokio::task::spawn(async move {
        tokio::time::sleep(deadline.to_std().unwrap_or_default()).await;

        if let Err(e) = respond_cx.reply_to(query).send().await {
            error!("Reminder failure: {}", e);
        }
    });

    cx.reply_to(format!(
        "Reminder saved for: {:?}",
        now.checked_add_signed(deadline)
    ))
    .await?;

    Ok(())
}
