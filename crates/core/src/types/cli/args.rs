use clap::{Parser, Subcommand};

// cli start --zip --local "file.zip"
// cli start --zip --remote "rem-file.zip"
// cli start --other-ext --local "file.ext"

#[derive(Subcommand, Debug)]
pub enum Command {
    Start(StartArgs),
    Abort,
    Pause,
    Resume,

    Status(StatusArgs),
    Search(SearchArgs),
}

#[derive(Parser, Debug)]
pub struct StartArgs {
    #[arg(long/*, conflicts_with = "other_ext"*/)]
    pub zip: bool,

    /*
    #[arg(long = "other-ext", conflicts_with = "zip")]
    other_ext: bool,

    */
    #[arg(long, conflicts_with = "remote")]
    pub local: Option<String>,
    #[arg(long, conflicts_with = "local")]
    pub remote: Option<String>,
}

#[derive(Parser, Debug)]
pub struct StatusArgs {
    #[arg(long)]
    pub watch: bool,

    #[arg(long)]
    pub period: Option<u64>,
}

#[derive(Parser, Debug)]
pub struct SearchArgs {
    #[arg(long)]
    pub interval: Option<u64>,

    #[arg(long)]
    pub alt: bool,
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}
