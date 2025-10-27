use crate::stats::HuntStats;
use colored::*;

/// Print results with optional statistics and keys list
pub fn print_results(
    unused_keys: &[String],
    stats: &HuntStats,
    show_stats: bool,
    show_keys: bool,
    is_clear_unused_flag: bool,
) {
    // Default: only show count
    if !show_stats && !show_keys {
        if unused_keys.is_empty() {
            println!("{}", "✓ No unused translation keys found!".green());
        } else {
            println!(
                "{} {} unused translation keys",
                "⚠️".yellow(),
                unused_keys.len().to_string().red().bold()
            );
        }
        return;
    }

    // Show keys list first if flag is set (but not when clearing unused keys)
    if show_keys && !is_clear_unused_flag {
        print_unused_keys(unused_keys);
        // Add spacing between keys and stats if both are shown
        if show_stats {
            println!();
        }
    }

    // Show stats last if flag is set
    if show_stats {
        print_stats(stats, is_clear_unused_flag);
    }
}

/// Print unused keys list
pub fn print_unused_keys(unused_keys: &[String]) {
    if unused_keys.is_empty() {
        return;
    }

    for key in unused_keys {
        println!("- {key}");
    }

    println!(
        "\n{} {} unused translation keys\n",
        "⚠️".yellow(),
        unused_keys.len().to_string().red().bold()
    );
}

/// Print statistics about the hunt
fn print_stats(stats: &HuntStats, is_clear_unused_flag: bool) {
    println!(
        "{} {} {}",
        "Files scanned:".cyan(),
        stats.files_total.to_string().bold(),
        "".dimmed()
    );
    println!(
        "{} {} {}",
        "Keys checked:".cyan(),
        stats.keys_total.to_string().bold(),
        "".dimmed()
    );
    println!(
        "{} {} {}",
        "Time spent:".cyan(),
        stats.formatted_duration().bold(),
        "".dimmed()
    );

    if !is_clear_unused_flag {
        println!(
            "{} {} {}",
            "Keys not used:".cyan(),
            stats.unused_keys_count.to_string().red().bold(),
            "".dimmed()
        );
    }
}

/// Print error messages with consistent styling
pub fn print_error(message: &str) {
    eprintln!("{} {}", "Error:".red().bold(), message);
}

/// Print cleared results message
pub fn print_cleared_results(
    unused_keys: &[String],
    stats: &HuntStats,
    show_stats: bool,
    show_keys: bool,
    is_clear_unused_flag: bool,
) {
    // Default: only show count
    if !show_stats && !show_keys {
        if unused_keys.is_empty() {
            println!("{}", "✓ No unused translation keys found!".green());
        } else {
            println!(
                "{} {} unused translation keys removed from translation files",
                "✓".green(),
                unused_keys.len().to_string().green().bold()
            );
        }
        return;
    }

    // Show count message
    if unused_keys.is_empty() {
        println!("{}", "✓ No unused translation keys found!".green());
    } else {
        println!(
            "{} {} unused translation keys removed from translation files",
            "✓".green(),
            unused_keys.len().to_string().green().bold()
        );
    }

    // Show keys list first if flag is set (but not when clearing unused keys)
    if show_keys && !is_clear_unused_flag {
        println!("\nRemoved keys:");
        for key in unused_keys {
            println!("- {key}");
        }
        // Add spacing between keys and stats if both are shown
        if show_stats {
            println!();
        }
    }

    // Show stats last if flag is set
    if show_stats {
        println!();
        print_stats(stats, is_clear_unused_flag);
    }
}

/// Print validation results (minimal output for pre-commit hooks)
pub fn print_validate_results(unused_keys: &[String], _stats: &HuntStats) {
    if unused_keys.is_empty() {
        println!("{}", "✓ No unused translation keys found!".green());
    } else {
        println!(
            "{} {} unused translation keys found",
            "✗".red(),
            unused_keys.len().to_string().red().bold()
        );
    }
}
