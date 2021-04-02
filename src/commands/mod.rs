use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;

mod health;
mod img;

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
}

#[tracing::instrument(skip(cx))]
pub(crate) async fn responder(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> anyhow::Result<(), anyhow::Error> {
    debug!("Incomming command: {:?}", command);
    match command {
        Command::Help => cx.answer(Command::descriptions()).send().await?,
        Command::Img(query) => img::image(&cx, &query).await?,
        Command::More => img::more(&cx).await?,
        Command::Health => health::status(&cx).await?,
    };

    Ok(())
}
