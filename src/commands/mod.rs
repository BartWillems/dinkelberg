use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;

pub mod img;

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Fetch an image")]
    Image(String),
    #[command(description = "Fetch more images")]
    More,
}

#[tracing::instrument(skip(cx))]
pub(crate) async fn responder(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> anyhow::Result<(), anyhow::Error> {
    match command {
        Command::Help => cx.answer(Command::descriptions()).send().await?,
        Command::Image(query) => img::image(&cx, &query).await?,
        Command::More => img::more(&cx).await?,
    };

    Ok(())
}
