use clap::Parser;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use pandorust::readers::markdown::read_markdown;
use pandorust::utils::error::{PandorustError, Result};
use pandorust::writers::docx::write_docx;
use pandorust::writers::html::write_html;

#[derive(Parser)]
#[command(
    name = "pandorust",
    version,
    about = "A pure-Rust document converter — single binary, no runtime dependencies.",
    long_about = "A pure-Rust document converter — single binary, no runtime dependencies.\n\n\
        Converts Markdown to HTML or DOCX. Supports YAML front matter for metadata\n\
        (title, author, date, fontsize), pandoc-style grid tables, and fenced divs.\n\n\
        INPUT FORMATS:  markdown (md)\n\
        OUTPUT FORMATS: html, docx\n\n\
        Use \"-\" as input to read from stdin. Formats auto-detect from file extensions.",
    after_help = "\
EXAMPLES:\n\
  pandorust input.md -o output.html          Convert Markdown to HTML\n\
  pandorust input.md -o output.docx          Convert Markdown to DOCX\n\
  pandorust input.md -o out.html -t html     Explicit output format\n\
  pandorust data.txt -f md -t html -o o.html Non-standard extension with format flags\n\
  cat input.md | pandorust - -t html -o o.html  Read from stdin\n\n\
YAML FRONT MATTER:\n\
  ---\n\
  title: My Document\n\
  author: Jane Doe\n\
  date: 2026-01-01\n\
  fontsize: 11pt\n\
  ---\n\n\
  title    → HTML <title>, DOCX core properties\n\
  author   → DOCX core properties\n\
  date     → DOCX core properties\n\
  fontsize → body text size (default: 12pt). DOCX uses half-points (11pt=22).\n\n\
SUPPORTED MARKDOWN FEATURES:\n\
  GFM (GitHub Flavored Markdown), pipe tables, grid tables (+---+---+),\n\
  fenced code blocks, blockquotes, ordered/unordered lists, inline formatting\n\
  (bold, italic, strikethrough, code, links, images), horizontal rules,\n\
  YAML front matter, fenced divs (::: syntax), \\newpage.\n\n\
EXIT CODES:\n\
  0  Success\n\
  1  Error (details on stderr)"
)]
struct Cli {
    /// Input file path. Use "-" to read from stdin.
    input: Option<String>,

    /// Output file path (required). Extension determines format unless -t is set.
    #[arg(short, long)]
    output: Option<String>,

    /// Input format: markdown, md. Auto-detected from extension if omitted.
    #[arg(short = 'f', long, value_name = "FORMAT")]
    from: Option<String>,

    /// Output format: html, docx. Auto-detected from extension if omitted.
    #[arg(short = 't', long, value_name = "FORMAT")]
    to: Option<String>,

    /// List supported input and output formats, then exit.
    #[arg(long)]
    list_formats: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.list_formats {
        println!("Input formats:");
        println!("  markdown  (.md)   GitHub Flavored Markdown with YAML front matter");
        println!();
        println!("Output formats:");
        println!("  html      (.html) Styled HTML with embedded CSS");
        println!("  docx      (.docx) Microsoft Word (Open XML)");
        return;
    }

    let input_path = match &cli.input {
        Some(i) => i.clone(),
        None => {
            eprintln!("Error: <INPUT> is required. Run with --help for usage.");
            std::process::exit(1);
        }
    };
    let output_path = match &cli.output {
        Some(o) => o.clone(),
        None => {
            eprintln!("Error: --output <OUTPUT> is required. Run with --help for usage.");
            std::process::exit(1);
        }
    };

    if let Err(e) = run(&input_path, &output_path, &cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(input_path: &str, output_path: &str, cli: &Cli) -> Result<()> {
    let from_fmt = cli
        .from
        .clone()
        .unwrap_or_else(|| detect_format(input_path));
    let to_fmt = cli.to.clone().unwrap_or_else(|| detect_format(output_path));

    // Read input: from stdin if "-", otherwise from file
    let input = if input_path == "-" {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(PandorustError::Io)?;
        buf
    } else {
        fs::read_to_string(input_path).map_err(PandorustError::Io)?
    };

    // Parse
    let doc = match from_fmt.as_str() {
        "md" | "markdown" => read_markdown(&input)?,
        other => {
            return Err(PandorustError::UnsupportedInputFormat(other.to_string()))
        }
    };

    // Write
    match to_fmt.as_str() {
        "html" => {
            let html = write_html(&doc);
            fs::write(output_path, html).map_err(PandorustError::Io)?;
        }
        "docx" => {
            let bytes = write_docx(&doc)?;
            fs::write(output_path, bytes).map_err(PandorustError::Io)?;
        }
        other => {
            return Err(PandorustError::UnsupportedOutputFormat(other.to_string()))
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
