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
    #[display(fmt = "π Dubs")]
    Dubs = 2,
    #[display(fmt = "π Trips")]
    Trips = 3,
    #[display(fmt = "π± Quads")]
    Quads = 4,
    #[display(fmt = "π€£π Penta")]
    Pentas = 5,
    #[display(fmt = "πππ€π€ππ Hexa")]
    Hexa = 6,
    #[display(fmt = "ππππ΅ Septa")]
    Septa = 7,
    #[display(fmt = "π±οΈOcta")]
    Octa = 8,
    #[display(fmt = "π―π―π― El NiΓ±o")]
    Nino = 9,
}

fn random_number() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(100000000..999999999)
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
            cx.reply_to("Something weird happened π€").await
        }
    }
}
