use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;

mod health;
mod img;
mod reminders;

pub(crate) type Context = UpdateWithCx<AutoSend<Bot>, Message>;

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Fetch an image")]
    Img(String),
    #[command(description = "Fetch more images")]
    More,
    #[command(description = "Get the bot's health status")]
    Health,
    #[command(description = "A place that is real and exists")]
    Bodegem,
    #[command(description = "Remind me in a given time")]
    RemindMe(String),
}

#[tracing::instrument(skip(cx))]
pub(crate) async fn responder(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> anyhow::Result<(), anyhow::Error> {
    debug!("Incomming command: {:?}", command);
    debug!("Group ID: {}", cx.chat_id());
    match command {
        Command::Help => {
            let _ = cx.answer(Command::descriptions()).send().await?;
        }
        Command::Img(query) => {
            let _ = img::image(&cx, &query).await?;
        }
        Command::More => {
            let _ = img::more(&cx).await?;
        }
        Command::Health => {
            let _ = health::status(&cx).await?;
        }
        Command::Bodegem => {
            let _ = cx.answer_location(50.8614773, 4.211304).await?;
        }
        Command::RemindMe(query) => {
            let _ = reminders::remind_me(Arc::new(cx), query).await?;
        }
    }

    Ok(())
}
