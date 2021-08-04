use std::convert::TryFrom;

use derive_more::Display;
use num_enum::TryFromPrimitive;
use rand::Rng;
use teloxide::prelude::*;
use teloxide::RequestError;

use crate::commands::Context;

#[derive(Debug, Display, TryFromPrimitive)]
#[repr(u8)]
enum Outcome {
    #[display(fmt = "")]
    Nothing = 1,
    #[display(fmt = "👌 Dubs")]
    Dubs = 2,
    #[display(fmt = "🙈 Trips")]
    Trips = 3,
    #[display(fmt = "😱 Quads")]
    Quads = 4,
    #[display(fmt = "🤣😂 Penta")]
    Pentas = 5,
    #[display(fmt = "👌👌🤔🤔😂😂 Hexa")]
    Hexa = 6,
    #[display(fmt = "🙊🙉🙈🐵 Septa")]
    Septa = 7,
    #[display(fmt = "🅱️Octa")]
    Octa = 8,
    #[display(fmt = "💯💯💯 El Niño")]
    Nino = 9,
}

fn random_number() -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1000000000..9999999999)
}

#[tracing::instrument(name = "commands::roll", skip(cx))]
pub(crate) async fn roll(cx: &Context) -> anyhow::Result<Message, RequestError> {
    let roll = random_number();
    let mut count: u8 = 1;

    let suffix = roll % 10;
    let mut roll_iter = roll / 10;

    while roll_iter != 0 {
        if roll_iter % 10 != suffix {
            break;
        }
        roll_iter /= 10;
        count += 1;
    }

    match Outcome::try_from(count) {
        Ok(outcome) => {
            log::trace!("Roll: `{}` count: `{}`", roll, count);
            cx.reply_to(format!("{} {}", roll, outcome)).await
        }
        Err(err) => {
            log::error!(
                "Unable to convert roll `{}` with count `{}` to result; error: `{}`",
                roll,
                count,
                err
            );
            cx.reply_to("Something weird happened 🤔").await
        }
    }
}
