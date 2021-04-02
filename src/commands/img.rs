use teloxide::{prelude::*, types::InputFile, RequestError};

use crate::cache::Cache;
use crate::commands::Context;
use crate::ddg;
use crate::ddg::ImageResponse;

#[tracing::instrument(name = "commands::image", skip(cx))]
pub(crate) async fn image(cx: &Context, query: &str) -> anyhow::Result<Message, anyhow::Error> {
    if query.is_empty() {
        return cx
            .answer("Please provide an image query")
            .await
            .map_err(|e| e.into());
    }

    let images = ddg::Client::search_images(&query).await?;

    Cache::set_scoped(&images, cx.chat_id()).await;

    let message: Message = match images.random() {
        Some(image) => cx.answer_photo(InputFile::url(image.image_url())).await?,
        None => cx.answer("No image found").await?,
    };

    Ok(message)
}

#[tracing::instrument(name = "commands::more", skip(cx))]
pub(crate) async fn more(cx: &Context) -> anyhow::Result<Message, RequestError> {
    let images: ImageResponse = match Cache::get_scoped(cx.chat_id()).await {
        Some(res) => res,
        None => {
            return cx.answer("You have to fetch images first").await;
        }
    };

    let message: Message = match images.random() {
        Some(image) => cx.answer_photo(InputFile::url(image.image_url())).await?,
        None => cx.answer("No image found").await?,
    };

    Ok(message)
}
