use clap::Parser;

#[derive(Parser)]
#[command(name = "hunt")]
#[command(about = "A lion's hunt for dead translation keys in your codebase.")]
#[command(version = "0.1.0")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Path to the translation file (JSON) or directory containing JSON files
    pub translation_path: String,

    /// Source directories to search (can specify multiple). If not provided, uses current directory.
    #[arg(short = 'd', long = "dir")]
    pub source_dirs: Vec<String>,

    /// Show statistics (files processed, time elapsed, etc.)
    #[arg(short = 's', long = "stats")]
    pub show_stats: bool,

    /// Remove unused keys from translation files
    #[arg(short = 'c', long = "clear")]
    pub clear_unused: bool,

    /// Validate that there are no unused keys (exits with code 1 if unused keys found)
    /// Useful for pre-commit hooks
    #[arg(long = "validate")]
    pub validate: bool,

    /// Show the list of unused keys
    #[arg(long = "keys")]
    pub show_keys: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn validate_source_dirs(&self) -> Vec<String> {
        let valid_dirs: Vec<String> = self
            .source_dirs
            .iter()
            .filter(|dir| !dir.is_empty())
            .cloned()
            .collect();

        // If no directories provided, default to current directory
        if valid_dirs.is_empty() {
            vec![".".to_string()]
        } else {
            valid_dirs
        }
    }
}
