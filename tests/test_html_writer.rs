use pandorust::readers::markdown::read_markdown;
use pandorust::writers::html::write_html;

#[test]
fn test_heading_to_html() {
    let doc = read_markdown("# Hello").unwrap();
    let html = write_html(&doc);
    assert!(html.contains("<h1>Hello</h1>"));
}

#[test]
fn test_bold_to_html() {
    let doc = read_markdown("This is **bold**").unwrap();
    let html = write_html(&doc);
    assert!(html.contains("<strong>bold</strong>"));
}

#[test]
fn test_table_to_html() {
    let doc = read_markdown("| A | B |\n|---|---|\n| 1 | 2 |").unwrap();
    let html = write_html(&doc);
    assert!(html.contains("<table>"));
    assert!(html.contains("<th>"));
    assert!(html.contains("<td>"));
}

#[test]
fn test_list_to_html() {
    let doc = read_markdown("- One\n- Two").unwrap();
    let html = write_html(&doc);
    assert!(html.contains("<ul>"));
    assert!(html.contains("<li>"));
}

#[test]
fn test_link_to_html() {
    let doc = read_markdown("[test](https://example.com)").unwrap();
    let html = write_html(&doc);
    assert!(html.contains("<a href=\"https://example.com\">test</a>"));
}

#[test]
fn test_code_block_to_html() {
    let doc = read_markdown("```rust\nlet x = 1;\n```").unwrap();
    let html = write_html(&doc);
    assert!(html.contains("<pre><code class=\"language-rust\">"));
}

#[test]
fn test_metadata_in_html() {
    let md = "---\ntitle: My Doc\nauthor: Tester\n---\n\nHello";
    let doc = read_markdown(md).unwrap();
    let html = write_html(&doc);
    assert!(html.contains("<title>My Doc</title>"));
    assert!(html.contains("<h1 class=\"title\">My Doc</h1>"));
    assert!(html.contains("Tester"));
}

#[test]
fn test_horizontal_rule_to_html() {
    let doc = read_markdown("Above\n\n---\n\nBelow").unwrap();
    let html = write_html(&doc);
    assert!(html.contains("<hr>"));
}

#[test]
fn test_html_has_default_font_styling() {
    let doc = read_markdown("Hello").unwrap();
    let html = write_html(&doc);
    assert!(html.contains("font-family"), "HTML should include font-family styling");
    assert!(html.contains("line-height"), "HTML should include line-height");
}

#[test]
fn test_html_respects_fontsize_meta() {
    let md = "---\nfontsize: 11pt\n---\n\nHello";
    let doc = read_markdown(md).unwrap();
    let html = write_html(&doc);
    assert!(html.contains("11pt"), "HTML should respect fontsize from metadata, got: {}", &html[..500.min(html.len())]);
}
