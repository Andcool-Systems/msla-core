use clap::{Parser, Subcommand};

// cli start --zip --local "file.zip"
// cli start --zip --remote "rem-file.zip"
// cli start --other-ext --local "file.ext"

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start print
    Start(StartArgs),

    /// Abort print
    Abort,

    /// Pause print
    Pause,

    /// Resume print
    Resume,

    /// Get current printer status
    Status(StatusArgs),

    /// Search printers in local network
    Search(SearchArgs),

    /// Home Z axis
    Home,

    /// Disable Z stepper
    DisableStepper,

    /// Show model preview
    ShowPreview,

    /// Add "Send to printer" context option
    ContextRegister,

    /// Remove "Send to printer" context option
    ContextUnregister,

    /// Display model info
    ModelInfo(ModelInfoArgs),
}

#[derive(Parser, Debug)]
pub struct ModelInfoArgs {
    #[arg(long, help = "Zip model extension"/*, conflicts_with = "other_ext"*/)]
    pub zip: bool,

    #[arg(long, help = "Path to the model")]
    pub path: String,
}

#[derive(Parser, Debug)]
pub struct StartArgs {
    #[arg(long, help = "Zip model extension"/*, conflicts_with = "other_ext"*/)]
    pub zip: bool,

    /*
    #[arg(long = "other-ext", conflicts_with = "zip")]
    other_ext: bool,

    */
    #[arg(long, conflicts_with = "remote", help = "Path to file on printer")]
    pub local: Option<String>,
    #[arg(long, conflicts_with = "local", help = "Path to file on this device")]
    pub remote: Option<String>,

    #[arg(long, help = "Do not shutdown status")]
    pub watch: bool,
}

#[derive(Parser, Debug)]
pub struct StatusArgs {
    #[arg(long, help = "Do not shutdown status, and update them every <period>")]
    pub watch: bool,

    #[arg(long, help = "Printer polling interval")]
    pub period: Option<u64>,
}

#[derive(Parser, Debug)]
pub struct SearchArgs {
    #[arg(long, help = "How long wait for a response from the printers")]
    pub timeout: Option<u64>,

    #[arg(long, help = "Use unicast method (broadcast by default)")]
    pub alt: bool,
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, help = "Specify printer IP/host")]
    pub host: Option<String>,

    #[arg(long, help = "Use unicast search method (broadcast by default)")]
    pub alt_scan: bool,

    #[arg(long, help = "Scanning port (710 by default)")]
    pub scan_port: Option<u16>,

    #[arg(long, help = "Specify printer port")]
    pub port: Option<u16>,

    #[arg(long, help = "Print is started from context menu")]
    pub from_context_menu: bool,
}
