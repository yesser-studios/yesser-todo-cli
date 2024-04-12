use clap:: {
    Args,
    Parser,
    Subcommand
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct TodoArgs {
    /// Name of the person to greet
    #[arg(short, long)]
    pub(crate) name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    pub(crate) count: u8,
}