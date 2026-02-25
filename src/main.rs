use clap::Parser;
use std::fs;
use std::path::Path;

use pandorust::readers::markdown::read_markdown;
use pandorust::utils::error::{PandorustError, Result};
use pandorust::writers::docx::write_docx;
use pandorust::writers::html::write_html;

#[derive(Parser)]
#[command(name = "pandorust", version, about = "A pure-Rust document converter")]
struct Cli {
    /// Input file path
    input: String,

    /// Output file path
    #[arg(short, long)]
    output: String,

    /// Input format (md). Auto-detected from extension if omitted.
    #[arg(short = 'f', long)]
    from: Option<String>,

    /// Output format (html, docx). Auto-detected from extension if omitted.
    #[arg(short = 't', long)]
    to: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<()> {
    // Detect formats from flags or file extensions
    let from_fmt = cli
        .from
        .clone()
        .unwrap_or_else(|| detect_format(&cli.input));
    let to_fmt = cli
        .to
        .clone()
        .unwrap_or_else(|| detect_format(&cli.output));

    // Read input file
    let input =
        fs::read_to_string(&cli.input).map_err(PandorustError::Io)?;

    // Parse based on input format
    let doc = match from_fmt.as_str() {
        "md" | "markdown" => read_markdown(&input)?,
        other => {
            return Err(PandorustError::UnsupportedInputFormat(
                other.to_string(),
            ))
        }
    };

    // Write based on output format
    match to_fmt.as_str() {
        "html" => {
            let html = write_html(&doc);
            fs::write(&cli.output, html).map_err(PandorustError::Io)?;
        }
        "docx" => {
            let bytes = write_docx(&doc)?;
            fs::write(&cli.output, bytes).map_err(PandorustError::Io)?;
        }
        other => {
            return Err(PandorustError::UnsupportedOutputFormat(
                other.to_string(),
            ))
        }
    }

    Ok(())
}

fn detect_format(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}
