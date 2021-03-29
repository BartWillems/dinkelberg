use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Dinkelberg", about = "A neat Telegram bot")]
pub struct Opt {
    /// Print the bot commands and exit
    #[structopt(short, long)]
    pub commands: bool,
}
