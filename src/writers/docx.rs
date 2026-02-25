use std::io::Cursor;

use docx_rs::{
    AlignmentType, BreakType, Docx, LineSpacing, Paragraph, Run, RunFonts, Shading, ShdType,
    Table, TableCell, TableCellBorder, TableCellBorderPosition, TableCellBorders, TableRow,
    WidthType,
};

use crate::ast::{Block, Document, Inline};
use crate::utils::error::{PandorustError, Result};

/// Parse fontsize metadata (e.g. "11pt") to half-points for DOCX.
/// DOCX sizes are in half-points: 11pt = 22, 12pt = 24, etc.
fn parse_fontsize(meta_fontsize: Option<&str>) -> usize {
    if let Some(s) = meta_fontsize {
        let num_str: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(pt) = num_str.parse::<usize>() {
            return pt * 2; // convert pt to half-points
        }
    }
    24 // default: 12pt = 24 half-points
}

/// Write a Document AST to DOCX bytes.
pub fn write_docx(doc: &Document) -> Result<Vec<u8>> {
    let mut docx = Docx::new();
    let base_size = parse_fontsize(doc.meta.get_str("fontsize"));
    let body_font = RunFonts::new()
        .ascii("Calibri")
        .hi_ansi("Calibri")
        .cs("Calibri");

    // --- Metadata block ---
    if let Some(title) = doc.meta.title() {
        let p = Paragraph::new()
            .align(AlignmentType::Center)
            .line_spacing(LineSpacing::new().after(60))
            .add_run(Run::new().fonts(body_font.clone()).bold().size(48).add_text(title));
        docx = docx.add_paragraph(p);
    }
    if let Some(subtitle) = doc.meta.subtitle() {
        let p = Paragraph::new()
            .align(AlignmentType::Center)
            .line_spacing(LineSpacing::new().after(60))
            .add_run(Run::new().fonts(body_font.clone()).size(32).add_text(subtitle));
        docx = docx.add_paragraph(p);
    }
    if let Some(author) = doc.meta.author() {
        let p = Paragraph::new()
            .align(AlignmentType::Center)
            .line_spacing(LineSpacing::new().after(40))
            .add_run(Run::new().fonts(body_font.clone()).size(base_size).add_text(format!("Author: {}", author)));
        docx = docx.add_paragraph(p);
    }
    if let Some(date) = doc.meta.date() {
        let p = Paragraph::new()
            .align(AlignmentType::Center)
            .line_spacing(LineSpacing::new().after(200))
            .add_run(Run::new().fonts(body_font.clone()).size(base_size).add_text(date));
        docx = docx.add_paragraph(p);
    }

    // --- Body blocks ---
    for block in &doc.blocks {
        docx = write_block(docx, block, base_size, &body_font);
    }

    // --- Pack to bytes ---
    let mut buf = Vec::new();
    docx.build()
        .pack(Cursor::new(&mut buf))
        .map_err(|e| PandorustError::DocxError(e.to_string()))?;

    Ok(buf)
}

fn write_block(docx: Docx, block: &Block, base_size: usize, body_font: &RunFonts) -> Docx {
    match block {
        Block::Para(inlines) | Block::Plain(inlines) => {
            let p = build_paragraph(inlines, Some(base_size), None, body_font)
                .line_spacing(LineSpacing::new().after(120).line(276));
            docx.add_paragraph(p)
        }

        Block::Heading(_, level, inlines) => {
            let size = heading_size(*level, base_size);
            let before = if *level <= 2 { 360 } else { 240 }; // more space before major headings
            let p = build_paragraph(inlines, Some(size), Some(true), body_font)
                .line_spacing(LineSpacing::new().before(before).after(120));
            docx.add_paragraph(p)
        }

        Block::CodeBlock(_, code) => {
            let courier = RunFonts::new()
                .ascii("Courier New")
                .hi_ansi("Courier New")
                .cs("Courier New");
            // Render each line separately so newlines work
            let mut d = docx;
            for line in code.lines() {
                let run = Run::new()
                    .fonts(courier.clone())
                    .add_text(line);
                let p = Paragraph::new().add_run(run);
                d = d.add_paragraph(p);
            }
            // If code was empty, still add one paragraph
            if code.is_empty() {
                let run = Run::new().fonts(courier).add_text("");
                d = d.add_paragraph(Paragraph::new().add_run(run));
            }
            d
        }

        Block::BlockQuote(inner_blocks) => {
            let mut d = docx;
            for inner in inner_blocks {
                d = write_block_quote_block(d, inner, base_size, body_font);
            }
            d
        }

        Block::BulletList(items) => {
            let mut d = docx;
            for item_blocks in items {
                let text = extract_inline_text_from_blocks(item_blocks);
                let p = Paragraph::new()
                    .indent(Some(720), None, None, None)
                    .line_spacing(LineSpacing::new().after(60).line(276))
                    .add_run(Run::new().fonts(body_font.clone()).size(base_size).add_text(format!("\u{2022} {}", text)));
                d = d.add_paragraph(p);
            }
            d
        }

        Block::OrderedList(attrs, items) => {
            let mut d = docx;
            let start = attrs.start;
            for (i, item_blocks) in items.iter().enumerate() {
                let num = start as usize + i;
                let text = extract_inline_text_from_blocks(item_blocks);
                let p = Paragraph::new()
                    .indent(Some(720), None, None, None)
                    .line_spacing(LineSpacing::new().after(60).line(276))
                    .add_run(Run::new().fonts(body_font.clone()).size(base_size).add_text(format!("{}. {}", num, text)));
                d = d.add_paragraph(p);
            }
            d
        }

        Block::Table(table) => {
            let num_cols = table.col_specs.len().max(1);
            let col_width = 9000 / num_cols;
            let grid: Vec<usize> = (0..num_cols).map(|_| col_width).collect();

            let mut rows: Vec<TableRow> = Vec::new();

            // Header rows
            for (row_idx, row) in table.head.rows.iter().enumerate() {
                let cells: Vec<TableCell> = row
                    .cells
                    .iter()
                    .map(|cell| {
                        let text = extract_inline_text_from_blocks(&cell.content);
                        let run = Run::new()
                            .fonts(body_font.clone())
                            .size(base_size)
                            .bold()
                            .color("FFFFFF")
                            .add_text(text);
                        let p = Paragraph::new().add_run(run);
                        let shading = Shading::new()
                            .shd_type(ShdType::Clear)
                            .color("auto")
                            .fill("1F4E79");
                        let borders = make_cell_borders("333333", 6);
                        TableCell::new()
                            .width(col_width, WidthType::Dxa)
                            .shading(shading)
                            .set_borders(borders)
                            .add_paragraph(p)
                    })
                    .collect();
                let _ = row_idx;
                rows.push(TableRow::new(cells));
            }

            // Body rows
            for (body_idx, body) in table.bodies.iter().enumerate() {
                let all_rows = body.head.iter().chain(body.body.iter());
                for (row_idx, row) in all_rows.enumerate() {
                    let fill = if row_idx % 2 == 0 { "FFFFFF" } else { "EDF2F7" };
                    let _ = body_idx;
                    let cells: Vec<TableCell> = row
                        .cells
                        .iter()
                        .map(|cell| {
                            let text = extract_inline_text_from_blocks(&cell.content);
                            let run = Run::new().fonts(body_font.clone()).size(base_size).add_text(text);
                            let p = Paragraph::new().add_run(run);
                            let shading = Shading::new()
                                .shd_type(ShdType::Clear)
                                .color("auto")
                                .fill(fill);
                            let borders = make_cell_borders("333333", 6);
                            TableCell::new()
                                .width(col_width, WidthType::Dxa)
                                .shading(shading)
                                .set_borders(borders)
                                .add_paragraph(p)
                        })
                        .collect();
                    rows.push(TableRow::new(cells));
                }
            }

            // Footer rows
            for row in &table.foot.rows {
                let cells: Vec<TableCell> = row
                    .cells
                    .iter()
                    .map(|cell| {
                        let text = extract_inline_text_from_blocks(&cell.content);
                        let run = Run::new().fonts(body_font.clone()).size(base_size).add_text(text);
                        let p = Paragraph::new().add_run(run);
                        let borders = make_cell_borders("333333", 6);
                        TableCell::new()
                            .width(col_width, WidthType::Dxa)
                            .set_borders(borders)
                            .add_paragraph(p)
                    })
                    .collect();
                rows.push(TableRow::new(cells));
            }

            if rows.is_empty() {
                rows.push(TableRow::new(vec![TableCell::new()]));
            }

            let tbl = Table::new(rows)
                .width(9000, WidthType::Dxa)
                .set_grid(grid);

            // Add spacing after table
            docx.add_table(tbl)
                .add_paragraph(Paragraph::new().line_spacing(LineSpacing::new().before(0).after(120)))
        }

        Block::HorizontalRule => {
            let p = Paragraph::new()
                .add_run(Run::new().fonts(body_font.clone()).size(base_size).add_text("—".repeat(40)));
            docx.add_paragraph(p)
        }

        Block::PageBreak => {
            let p = Paragraph::new()
                .add_run(Run::new().add_break(BreakType::Page));
            docx.add_paragraph(p)
        }

        Block::LineBlock(lines) => {
            let mut d = docx;
            for line_inlines in lines {
                let p = build_paragraph(line_inlines, Some(base_size), None, body_font);
                d = d.add_paragraph(p);
            }
            d
        }

        Block::RawBlock(_, _) => docx,
        Block::Figure(_, _, blocks) | Block::Div(_, blocks) => {
            let mut d = docx;
            for b in blocks {
                d = write_block(d, b, base_size, body_font);
            }
            d
        }
        Block::DefinitionList(items) => {
            let mut d = docx;
            for (term_inlines, definitions) in items {
                let p = build_paragraph(term_inlines, Some(base_size), Some(true), body_font);
                d = d.add_paragraph(p);
                for def_blocks in definitions {
                    for b in def_blocks {
                        d = write_block_quote_block(d, b, base_size, body_font);
                    }
                }
            }
            d
        }
    }
}

/// Write a block inside a block quote (indented).
fn write_block_quote_block(docx: Docx, block: &Block, base_size: usize, body_font: &RunFonts) -> Docx {
    match block {
        Block::Para(inlines) | Block::Plain(inlines) => {
            let p = build_paragraph(inlines, Some(base_size), None, body_font)
                .indent(Some(720), None, None, None)
                .line_spacing(LineSpacing::new().after(80).line(276));
            docx.add_paragraph(p)
        }
        other => write_block(docx, other, base_size, body_font),
    }
}

/// Build a paragraph from a slice of Inline elements.
/// `size` is in half-points (e.g. 24 = 12pt).
/// `bold` overrides all runs to bold.
fn build_paragraph(inlines: &[Inline], size: Option<usize>, bold_override: Option<bool>, body_font: &RunFonts) -> Paragraph {
    let mut p = Paragraph::new();
    let runs = build_runs(inlines, size, bold_override, body_font);
    for run in runs {
        p = p.add_run(run);
    }
    p
}

/// Recursively convert Inline elements to docx-rs Runs.
fn build_runs(inlines: &[Inline], size: Option<usize>, bold_override: Option<bool>, body_font: &RunFonts) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();

    for inline in inlines {
        match inline {
            Inline::Str(s) => {
                let mut run = Run::new().fonts(body_font.clone()).add_text(s.clone());
                if let Some(sz) = size { run = run.size(sz); }
                if bold_override == Some(true) { run = run.bold(); }
                runs.push(run);
            }

            Inline::Space | Inline::SoftBreak => {
                let mut run = Run::new().fonts(body_font.clone()).add_text(" ");
                if let Some(sz) = size { run = run.size(sz); }
                if bold_override == Some(true) { run = run.bold(); }
                runs.push(run);
            }

            Inline::LineBreak => {
                let mut run = Run::new().fonts(body_font.clone()).add_break(BreakType::TextWrapping);
                if let Some(sz) = size { run = run.size(sz); }
                runs.push(run);
            }

            Inline::Strong(inner) => {
                for mut r in build_runs(inner, size, Some(true), body_font) {
                    r = r.bold();
                    runs.push(r);
                }
            }

            Inline::Emph(inner) => {
                for mut r in build_runs(inner, size, bold_override, body_font) {
                    r = r.italic();
                    runs.push(r);
                }
            }

            Inline::Strikeout(inner) => {
                for mut r in build_runs(inner, size, bold_override, body_font) {
                    r = r.strike();
                    runs.push(r);
                }
            }

            Inline::Underline(inner) => {
                for mut r in build_runs(inner, size, bold_override, body_font) {
                    r = r.underline("single");
                    runs.push(r);
                }
            }

            Inline::Code(_, code_str) => {
                let courier = RunFonts::new()
                    .ascii("Courier New")
                    .hi_ansi("Courier New")
                    .cs("Courier New");
                let mut run = Run::new().fonts(courier).add_text(code_str.clone());
                if let Some(sz) = size { run = run.size(sz); }
                runs.push(run);
            }

            Inline::Link(_, content_inlines, target) => {
                let link_text = if content_inlines.is_empty() {
                    target.url.clone()
                } else {
                    inline_text_content(content_inlines)
                };
                let mut run = Run::new().fonts(body_font.clone())
                    .color("0000FF").underline("single").add_text(link_text);
                if let Some(sz) = size { run = run.size(sz); }
                if bold_override == Some(true) { run = run.bold(); }
                runs.push(run);
            }

            Inline::Image(_, alt_inlines, target) => {
                let alt = if alt_inlines.is_empty() {
                    target.url.clone()
                } else {
                    inline_text_content(alt_inlines)
                };
                let mut run = Run::new().fonts(body_font.clone()).italic().add_text(format!("[Image: {}]", alt));
                if let Some(sz) = size { run = run.size(sz); }
                runs.push(run);
            }

            Inline::Superscript(inner) => {
                runs.extend(build_runs(inner, size, bold_override, body_font));
            }

            Inline::Subscript(inner) => {
                runs.extend(build_runs(inner, size, bold_override, body_font));
            }

            Inline::SmallCaps(inner) => {
                runs.extend(build_runs(inner, size, bold_override, body_font));
            }

            Inline::Quoted(_, inner) => {
                let mut open = Run::new().fonts(body_font.clone()).add_text("\u{201C}");
                if let Some(sz) = size { open = open.size(sz); }
                runs.push(open);
                runs.extend(build_runs(inner, size, bold_override, body_font));
                let mut close = Run::new().fonts(body_font.clone()).add_text("\u{201D}");
                if let Some(sz) = size { close = close.size(sz); }
                runs.push(close);
            }

            Inline::Math(_, math_str) => {
                let courier = RunFonts::new().ascii("Courier New").hi_ansi("Courier New");
                let mut run = Run::new().fonts(courier).add_text(math_str.clone());
                if let Some(sz) = size { run = run.size(sz); }
                runs.push(run);
            }

            Inline::Span(_, inner) => {
                runs.extend(build_runs(inner, size, bold_override, body_font));
            }

            Inline::Note(blocks) => {
                let text = extract_inline_text_from_blocks(blocks);
                let mut run = Run::new().fonts(body_font.clone()).add_text(format!(" ({})", text));
                if let Some(sz) = size { run = run.size(sz); }
                runs.push(run);
            }

            Inline::RawInline(_, raw) => {
                let mut run = Run::new().fonts(body_font.clone()).add_text(raw.clone());
                if let Some(sz) = size { run = run.size(sz); }
                runs.push(run);
            }
        }
    }

    runs
}

/// Extract plain text from a list of blocks (best-effort, for tables/lists).
fn extract_inline_text_from_blocks(blocks: &[Block]) -> String {
    blocks
        .iter()
        .map(|b| match b {
            Block::Para(inlines) | Block::Plain(inlines) => inline_text_content(inlines),
            Block::Heading(_, _, inlines) => inline_text_content(inlines),
            Block::CodeBlock(_, code) => code.clone(),
            Block::BulletList(items) => items
                .iter()
                .map(|i| format!("• {}", extract_inline_text_from_blocks(i)))
                .collect::<Vec<_>>()
                .join(" "),
            Block::OrderedList(_, items) => items
                .iter()
                .enumerate()
                .map(|(i, item)| format!("{}. {}", i + 1, extract_inline_text_from_blocks(item)))
                .collect::<Vec<_>>()
                .join(" "),
            Block::BlockQuote(inner) => extract_inline_text_from_blocks(inner),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Extract plain text from a list of inlines (recursive, no formatting).
fn inline_text_content(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .map(|i| match i {
            Inline::Str(s) => s.clone(),
            Inline::Space | Inline::SoftBreak => " ".to_string(),
            Inline::LineBreak => "\n".to_string(),
            Inline::Strong(inner)
            | Inline::Emph(inner)
            | Inline::Underline(inner)
            | Inline::Strikeout(inner)
            | Inline::Superscript(inner)
            | Inline::Subscript(inner)
            | Inline::SmallCaps(inner)
            | Inline::Span(_, inner) => inline_text_content(inner),
            Inline::Quoted(_, inner) => {
                format!("\u{201C}{}\u{201D}", inline_text_content(inner))
            }
            Inline::Code(_, s) => s.clone(),
            Inline::Math(_, s) => s.clone(),
            Inline::Link(_, content, target) => {
                if content.is_empty() {
                    target.url.clone()
                } else {
                    inline_text_content(content)
                }
            }
            Inline::Image(_, alt, _) => inline_text_content(alt),
            Inline::Note(blocks) => extract_inline_text_from_blocks(blocks),
            Inline::RawInline(_, s) => s.clone(),
        })
        .collect()
}

/// Returns heading font size in half-points for a given heading level (1-6).
/// Sizes are relative to the base_size (body text size in half-points).
fn heading_size(level: u8, base_size: usize) -> usize {
    match level {
        1 => base_size + 14, // e.g. 22 + 14 = 36 for 11pt base
        2 => base_size + 8,  // e.g. 22 + 8 = 30
        3 => base_size + 4,  // e.g. 22 + 4 = 26
        4 => base_size + 2,  // e.g. 22 + 2 = 24
        5 => base_size,
        _ => base_size - 2,  // level 6
    }
}

/// Build a TableCellBorders with all four sides set to a given color and size.
fn make_cell_borders(color: &str, size: usize) -> TableCellBorders {
    TableCellBorders::new()
        .set(
            TableCellBorder::new(TableCellBorderPosition::Top)
                .color(color)
                .size(size),
        )
        .set(
            TableCellBorder::new(TableCellBorderPosition::Bottom)
                .color(color)
                .size(size),
        )
        .set(
            TableCellBorder::new(TableCellBorderPosition::Left)
                .color(color)
                .size(size),
        )
        .set(
            TableCellBorder::new(TableCellBorderPosition::Right)
                .color(color)
                .size(size),
        )
        .set(
            TableCellBorder::new(TableCellBorderPosition::InsideH)
                .color(color)
                .size(size),
        )
        .set(
            TableCellBorder::new(TableCellBorderPosition::InsideV)
                .color(color)
                .size(size),
        )
}
