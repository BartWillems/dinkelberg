use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;

mod health;
mod img;
mod reminders;
mod roll;
mod what;

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
    #[command(description = "Lookup what something is")]
    What(String),
    #[command(description = "Praise Kek")]
    Roll,
}

#[tracing::instrument(skip(cx))]
pub(crate) async fn responder(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> anyhow::Result<(), anyhow::Error> {
    debug!(
        "Incomming command: `{:?}`, Group ID: `{}`",
        command,
        cx.chat_id()
    );
    match command {
        Command::Help => {
            cx.answer(Command::descriptions()).send().await?;
        }
        Command::Img(query) => {
            img::image(&cx, &query).await?;
        }
        Command::More => {
            img::more(&cx).await?;
        }
        Command::Health => {
            health::status(&cx).await?;
        }
        Command::Bodegem => {
            cx.answer_location(50.8614773, 4.211304).await?;
        }
        Command::RemindMe(query) => {
            reminders::remind_me(Arc::new(cx), query).await?;
        }
        Command::What(query) => {
            what::what(&cx, &query).await?;
        }
        Command::Roll => {
            roll::roll(&cx).await?;
        }
    }

    Ok(())
}
