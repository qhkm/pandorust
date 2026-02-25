use crate::ast::{
    Alignment, Attr, Block, Document, Inline, MathType, QuoteType,
};

/// Convert a Document AST into a full HTML string.
pub fn write_html(doc: &Document) -> String {
    let mut out = String::new();

    // ---- <head> ----
    let title = doc.meta.title().unwrap_or("");
    let fontsize = doc.meta.get_str("fontsize").unwrap_or("12pt");
    out.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n");
    if !title.is_empty() {
        out.push_str(&format!("<title>{}</title>\n", escape_html(title)));
    }
    out.push_str(&format!(
        "<style>\nbody {{ font-family: \"Calibri\", \"Segoe UI\", \"Arial\", sans-serif; font-size: {}; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 2em; color: #333; }}\ntable {{ border-collapse: collapse; width: 100%; margin: 1em 0; }}\nth, td {{ border: 1px solid #999; padding: 8px 12px; text-align: left; }}\nth {{ background-color: #1F4E79; color: white; font-weight: bold; }}\ntr:nth-child(even) {{ background-color: #EDF2F7; }}\npre {{ background: #f5f5f5; padding: 1em; overflow-x: auto; border-radius: 4px; }}\ncode {{ font-family: \"Courier New\", monospace; }}\nblockquote {{ border-left: 4px solid #1F4E79; margin: 1em 0; padding: 0.5em 1em; background: #f9f9f9; }}\nh1, h2, h3 {{ color: #1F4E79; }}\nhr {{ border: none; border-top: 2px solid #ccc; margin: 2em 0; }}\n</style>\n",
        escape_html(fontsize)
    ));
    out.push_str("</head>\n<body>\n");

    // ---- metadata header block ----
    if !title.is_empty()
        || doc.meta.subtitle().is_some()
        || doc.meta.author().is_some()
        || doc.meta.date().is_some()
    {
        out.push_str("<header>\n");
        if !title.is_empty() {
            out.push_str(&format!(
                "<h1 class=\"title\">{}</h1>\n",
                escape_html(title)
            ));
        }
        if let Some(subtitle) = doc.meta.subtitle() {
            out.push_str(&format!(
                "<p class=\"subtitle\">{}</p>\n",
                escape_html(subtitle)
            ));
        }
        if let Some(author) = doc.meta.author() {
            out.push_str(&format!(
                "<p class=\"author\">{}</p>\n",
                escape_html(author)
            ));
        }
        if let Some(date) = doc.meta.date() {
            out.push_str(&format!(
                "<p class=\"date\">{}</p>\n",
                escape_html(date)
            ));
        }
        out.push_str("</header>\n");
    }

    // ---- body blocks ----
    for block in &doc.blocks {
        write_block(&mut out, block);
    }

    out.push_str("</body>\n</html>");
    out
}

// ---------------------------------------------------------------------------
// Block rendering
// ---------------------------------------------------------------------------

fn write_block(out: &mut String, block: &Block) {
    match block {
        Block::Para(inlines) | Block::Plain(inlines) => {
            out.push_str("<p>");
            write_inlines(out, inlines);
            out.push_str("</p>\n");
        }

        Block::Heading(attr, level, inlines) => {
            let tag = heading_tag(*level);
            let attr_str = render_attr(attr);
            out.push_str(&format!("<{tag}{attr_str}>"));
            write_inlines(out, inlines);
            out.push_str(&format!("</{tag}>\n"));
        }

        Block::CodeBlock(attr, code) => {
            // First class is treated as the language identifier
            let lang_class = attr.classes.first().map(|s| s.as_str()).unwrap_or("");
            if lang_class.is_empty() {
                out.push_str("<pre><code>");
            } else {
                out.push_str(&format!(
                    "<pre><code class=\"language-{}\">",
                    escape_attr(lang_class)
                ));
            }
            out.push_str(&escape_html(code));
            out.push_str("</code></pre>\n");
        }

        Block::BlockQuote(blocks) => {
            out.push_str("<blockquote>\n");
            for b in blocks {
                write_block(out, b);
            }
            out.push_str("</blockquote>\n");
        }

        Block::BulletList(items) => {
            out.push_str("<ul>\n");
            for item in items {
                out.push_str("<li>");
                write_list_item_blocks(out, item);
                out.push_str("</li>\n");
            }
            out.push_str("</ul>\n");
        }

        Block::OrderedList(attrs, items) => {
            let start = attrs.start;
            if start == 1 {
                out.push_str("<ol>\n");
            } else {
                out.push_str(&format!("<ol start=\"{start}\">\n"));
            }
            for item in items {
                out.push_str("<li>");
                write_list_item_blocks(out, item);
                out.push_str("</li>\n");
            }
            out.push_str("</ol>\n");
        }

        Block::DefinitionList(items) => {
            out.push_str("<dl>\n");
            for (term, defs) in items {
                out.push_str("<dt>");
                write_inlines(out, term);
                out.push_str("</dt>\n");
                for def in defs {
                    out.push_str("<dd>");
                    write_list_item_blocks(out, def);
                    out.push_str("</dd>\n");
                }
            }
            out.push_str("</dl>\n");
        }

        Block::Table(table) => {
            out.push_str("<table>\n");

            // thead
            if !table.head.rows.is_empty() {
                out.push_str("<thead>\n");
                for row in &table.head.rows {
                    out.push_str("<tr>");
                    for cell in &row.cells {
                        let align_style = alignment_style(&cell.align);
                        let span_attrs = cell_span_attrs(cell.row_span, cell.col_span);
                        out.push_str(&format!("<th{align_style}{span_attrs}>"));
                        write_cell_content(out, &cell.content);
                        out.push_str("</th>");
                    }
                    out.push_str("</tr>\n");
                }
                out.push_str("</thead>\n");
            }

            // tbody
            let has_body = table
                .bodies
                .iter()
                .any(|b| !b.head.is_empty() || !b.body.is_empty());
            if has_body {
                out.push_str("<tbody>\n");
                for body in &table.bodies {
                    for row in body.head.iter().chain(body.body.iter()) {
                        out.push_str("<tr>");
                        for cell in &row.cells {
                            let align_style = alignment_style(&cell.align);
                            let span_attrs = cell_span_attrs(cell.row_span, cell.col_span);
                            out.push_str(&format!("<td{align_style}{span_attrs}>"));
                            write_cell_content(out, &cell.content);
                            out.push_str("</td>");
                        }
                        out.push_str("</tr>\n");
                    }
                }
                out.push_str("</tbody>\n");
            }

            // tfoot
            if !table.foot.rows.is_empty() {
                out.push_str("<tfoot>\n");
                for row in &table.foot.rows {
                    out.push_str("<tr>");
                    for cell in &row.cells {
                        let align_style = alignment_style(&cell.align);
                        let span_attrs = cell_span_attrs(cell.row_span, cell.col_span);
                        out.push_str(&format!("<td{align_style}{span_attrs}>"));
                        write_cell_content(out, &cell.content);
                        out.push_str("</td>");
                    }
                    out.push_str("</tr>\n");
                }
                out.push_str("</tfoot>\n");
            }

            out.push_str("</table>\n");
        }

        Block::Figure(attr, _caption, blocks) => {
            let attr_str = render_attr(attr);
            out.push_str(&format!("<figure{attr_str}>\n"));
            for b in blocks {
                write_block(out, b);
            }
            out.push_str("</figure>\n");
        }

        Block::Div(attr, blocks) => {
            let attr_str = render_attr(attr);
            out.push_str(&format!("<div{attr_str}>\n"));
            for b in blocks {
                write_block(out, b);
            }
            out.push_str("</div>\n");
        }

        Block::LineBlock(lines) => {
            out.push_str("<div class=\"line-block\">\n");
            for line in lines {
                write_inlines(out, line);
                out.push_str("<br>\n");
            }
            out.push_str("</div>\n");
        }

        Block::RawBlock(fmt, content) => {
            if fmt.0 == "html" {
                out.push_str(content);
                if !content.ends_with('\n') {
                    out.push('\n');
                }
            }
            // Other formats are silently ignored in HTML output
        }

        Block::HorizontalRule => {
            out.push_str("<hr>\n");
        }

        Block::PageBreak => {
            out.push_str("<div style=\"page-break-after: always;\"></div>\n");
        }
    }
}

// ---------------------------------------------------------------------------
// Inline rendering
// ---------------------------------------------------------------------------

fn write_inlines(out: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        write_inline(out, inline);
    }
}

fn write_inline(out: &mut String, inline: &Inline) {
    match inline {
        Inline::Str(s) => out.push_str(&escape_html(s)),

        Inline::Space => out.push(' '),

        Inline::SoftBreak => out.push('\n'),

        Inline::LineBreak => out.push_str("<br>\n"),

        Inline::Emph(inlines) => {
            out.push_str("<em>");
            write_inlines(out, inlines);
            out.push_str("</em>");
        }

        Inline::Strong(inlines) => {
            out.push_str("<strong>");
            write_inlines(out, inlines);
            out.push_str("</strong>");
        }

        Inline::Underline(inlines) => {
            out.push_str("<u>");
            write_inlines(out, inlines);
            out.push_str("</u>");
        }

        Inline::Strikeout(inlines) => {
            out.push_str("<del>");
            write_inlines(out, inlines);
            out.push_str("</del>");
        }

        Inline::Superscript(inlines) => {
            out.push_str("<sup>");
            write_inlines(out, inlines);
            out.push_str("</sup>");
        }

        Inline::Subscript(inlines) => {
            out.push_str("<sub>");
            write_inlines(out, inlines);
            out.push_str("</sub>");
        }

        Inline::SmallCaps(inlines) => {
            out.push_str("<span style=\"font-variant: small-caps;\">");
            write_inlines(out, inlines);
            out.push_str("</span>");
        }

        Inline::Quoted(quote_type, inlines) => {
            let (open, close) = match quote_type {
                QuoteType::SingleQuote => ("&#8216;", "&#8217;"),
                QuoteType::DoubleQuote => ("&#8220;", "&#8221;"),
            };
            out.push_str(open);
            write_inlines(out, inlines);
            out.push_str(close);
        }

        Inline::Code(_, code) => {
            out.push_str("<code>");
            out.push_str(&escape_html(code));
            out.push_str("</code>");
        }

        Inline::Math(math_type, content) => match math_type {
            MathType::InlineMath => {
                out.push_str(&format!("\\({}\\)", escape_html(content)));
            }
            MathType::DisplayMath => {
                out.push_str(&format!("\\[{}\\]", escape_html(content)));
            }
        },

        Inline::Link(attr, inlines, target) => {
            let mut extra = format!(" href=\"{}\"", escape_attr(&target.url));
            if !target.title.is_empty() {
                extra.push_str(&format!(" title=\"{}\"", escape_attr(&target.title)));
            }
            let attr_str = render_attr(attr);
            out.push_str(&format!("<a{extra}{attr_str}>"));
            write_inlines(out, inlines);
            out.push_str("</a>");
        }

        Inline::Image(attr, inlines, target) => {
            // Collect alt text from inlines
            let mut alt = String::new();
            write_inlines(&mut alt, inlines);

            let attr_str = render_attr(attr);
            out.push_str(&format!(
                "<img src=\"{}\" alt=\"{}\"",
                escape_attr(&target.url),
                escape_attr(&alt)
            ));
            if !target.title.is_empty() {
                out.push_str(&format!(" title=\"{}\"", escape_attr(&target.title)));
            }
            out.push_str(&format!("{attr_str}>"));
        }

        Inline::Note(blocks) => {
            // Render footnote inline as a span (simplified)
            out.push_str("<span class=\"footnote\">");
            for b in blocks {
                write_block(out, b);
            }
            out.push_str("</span>");
        }

        Inline::Span(attr, inlines) => {
            let attr_str = render_attr(attr);
            out.push_str(&format!("<span{attr_str}>"));
            write_inlines(out, inlines);
            out.push_str("</span>");
        }

        Inline::RawInline(fmt, content) => {
            if fmt.0 == "html" {
                out.push_str(content);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Render list-item block content: unwrap a single Para into plain text,
/// otherwise render full blocks.
fn write_list_item_blocks(out: &mut String, blocks: &[Block]) {
    if blocks.len() == 1 {
        match &blocks[0] {
            Block::Para(inlines) | Block::Plain(inlines) => {
                write_inlines(out, inlines);
                return;
            }
            _ => {}
        }
    }
    for b in blocks {
        write_block(out, b);
    }
}

/// Render table cell content (similar to list items: unwrap single Para).
fn write_cell_content(out: &mut String, blocks: &[Block]) {
    if blocks.len() == 1 {
        match &blocks[0] {
            Block::Para(inlines) | Block::Plain(inlines) => {
                write_inlines(out, inlines);
                return;
            }
            _ => {}
        }
    }
    for b in blocks {
        write_block(out, b);
    }
}

fn heading_tag(level: u8) -> &'static str {
    match level {
        1 => "h1",
        2 => "h2",
        3 => "h3",
        4 => "h4",
        5 => "h5",
        _ => "h6",
    }
}

/// Build the HTML attribute string for an Attr (id, class, extra key=value pairs).
fn render_attr(attr: &Attr) -> String {
    let mut s = String::new();
    if !attr.id.is_empty() {
        s.push_str(&format!(" id=\"{}\"", escape_attr(&attr.id)));
    }
    if !attr.classes.is_empty() {
        let classes = attr
            .classes
            .iter()
            .map(|c| escape_attr(c))
            .collect::<Vec<_>>()
            .join(" ");
        s.push_str(&format!(" class=\"{classes}\""));
    }
    for (k, v) in &attr.attrs {
        s.push_str(&format!(" {}=\"{}\"", escape_attr(k), escape_attr(v)));
    }
    s
}

fn alignment_style(align: &Alignment) -> String {
    match align {
        Alignment::AlignLeft => " style=\"text-align: left;\"".to_string(),
        Alignment::AlignRight => " style=\"text-align: right;\"".to_string(),
        Alignment::AlignCenter => " style=\"text-align: center;\"".to_string(),
        Alignment::AlignDefault => String::new(),
    }
}

fn cell_span_attrs(row_span: u32, col_span: u32) -> String {
    let mut s = String::new();
    if row_span > 1 {
        s.push_str(&format!(" rowspan=\"{row_span}\""));
    }
    if col_span > 1 {
        s.push_str(&format!(" colspan=\"{col_span}\""));
    }
    s
}

/// Escape characters that are special in HTML text content.
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Escape characters that are special inside HTML attribute values (double-quoted).
fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}
