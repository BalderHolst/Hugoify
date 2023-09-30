use clap::Parser;

/// Convert an Obsidian directory to Hugo-compatible markdown.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to your Obsidian vault
    #[clap(index = 1)]
    pub vault: String,

    /// Path to your output directory
    #[clap(index = 2)]
    pub dest: String,
}
