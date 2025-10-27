mod cli;
mod ignore;
mod output;
mod search;
mod stats;
mod translation;

fn main() {
    if let Err(e) = run() {
        output::print_error(&e.to_string());
        eprintln!("  Example: hunt <translation_path> --dir src");
        eprintln!("           hunt <translation_path>  # searches current directory");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse_args();
    let source_dirs = cli.validate_source_dirs();

    let has_unused = handle_unused(&cli, &source_dirs)?;

    // In validate mode, exit with error code if unused keys found
    if cli.validate && has_unused {
        std::process::exit(1);
    }

    Ok(())
}

fn handle_unused(
    cli: &cli::Cli,
    source_dirs: &[String],
) -> Result<bool, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    let translations = translation::load_translations(&cli.translation_path)?;
    let source_files = search::discover_source_files(source_dirs)?;
    let used_keys = search::check_translation_usage(&translations, &source_files);

    let unused_keys: Vec<_> = translations
        .keys()
        .filter(|key: &&String| !used_keys.contains(key.as_str()))
        .cloned()
        .collect();

    let stats = stats::HuntStats {
        files_total: source_files.len(),
        keys_total: translations.len(),
        unused_keys_count: unused_keys.len(),
        duration: start_time.elapsed(),
    };

    let has_unused = !unused_keys.is_empty();

    if cli.clear_unused {
        translation::remove_unused_keys(&cli.translation_path, &unused_keys, &used_keys)?;
        output::print_cleared_results(
            &unused_keys,
            &stats,
            cli.show_stats,
            cli.show_keys,
            cli.clear_unused,
        );
    } else {
        // In validate mode, show minimal output
        if cli.validate {
            output::print_validate_results(&unused_keys, &stats);
        } else {
            output::print_results(
                &unused_keys,
                &stats,
                cli.show_stats,
                cli.show_keys,
                cli.clear_unused,
            );
        }
    }

    Ok(has_unused)
}
