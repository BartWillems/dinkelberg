use std::time::Duration;

use chrono::{Local, NaiveDateTime};
use date_time_parser::{DateParser, TimeParser};
use teloxide::prelude::*;
use teloxide::RequestError;

use crate::commands::Context;

#[tracing::instrument(name = "commands::remind_me", skip(cx))]
pub(crate) async fn remind_me(cx: &Context, query: String) -> anyhow::Result<(), RequestError> {
    let date = DateParser::parse(&query);
    let time = TimeParser::parse(&query);

    let now = Local::now().naive_utc();

    let deadline: Duration = match (date, time) {
        (None, None) => {
            cx.answer("No date or time found").await?;
            return Ok(());
        }
        (Some(date), Some(time)) => {
            debug!(
                "Reminder found with date: ({:?}) and time: ({:?})",
                date, time
            );
            NaiveDateTime::new(date, time)
                .signed_duration_since(now)
                .to_std()
                .unwrap_or_default()
        }
        (Some(date), None) => {
            debug!("Reminder found with only a date: {:?}", date);
            date.signed_duration_since(now.date())
                .to_std()
                .unwrap_or_default()
        }
        (None, Some(time)) => {
            debug!("Reminder found with only a time: {:?}", time);
            time.signed_duration_since(now.time())
                .to_std()
                .unwrap_or_default()
        }
    };

    debug!("Reminder created with deadline: {:?}", deadline);

    let group_id = cx.update.chat_id();

    tokio::task::spawn(async move {
        tokio::time::sleep(deadline).await;

        if let Err(e) = crate::BOT.send_message(group_id, query).await {
            error!("Reminder failure: {}", e);
        }
    });

    Ok(())
}
