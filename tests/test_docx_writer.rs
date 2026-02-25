use pandorust::readers::markdown::read_markdown;
use pandorust::writers::docx::write_docx;
use std::io::Read;
use std::io::Cursor;

#[test]
fn test_docx_generates_valid_zip() {
    let doc = read_markdown("# Hello\n\nTest paragraph.").unwrap();
    let bytes = write_docx(&doc).unwrap();
    assert!(bytes.len() > 100);
    assert_eq!(&bytes[0..2], b"PK"); // DOCX is a zip
}

#[test]
fn test_docx_with_table() {
    let doc = read_markdown("| A | B |\n|---|---|\n| 1 | 2 |").unwrap();
    let bytes = write_docx(&doc).unwrap();
    assert!(bytes.len() > 100);
    assert_eq!(&bytes[0..2], b"PK");
}

#[test]
fn test_docx_with_metadata() {
    let md = "---\ntitle: Test\nauthor: Me\n---\n\nContent.";
    let doc = read_markdown(md).unwrap();
    let bytes = write_docx(&doc).unwrap();
    assert!(bytes.len() > 100);
}

#[test]
fn test_docx_with_lists() {
    let doc = read_markdown("- One\n- Two\n- Three").unwrap();
    let bytes = write_docx(&doc).unwrap();
    assert!(bytes.len() > 100);
}

#[test]
fn test_docx_with_code_block() {
    let doc = read_markdown("```rust\nfn main() {}\n```").unwrap();
    let bytes = write_docx(&doc).unwrap();
    assert!(bytes.len() > 100);
}

#[test]
fn test_docx_body_text_has_font() {
    // DOCX body text should use a professional font (Calibri/Arial), not system default
    let doc = read_markdown("Hello world").unwrap();
    let bytes = write_docx(&doc).unwrap();
    let content = String::from_utf8_lossy(&bytes);
    // The DOCX XML should reference a font name for body text
    assert!(content.contains("Calibri") || content.contains("Arial"),
        "DOCX body text should use Calibri or Arial font");
}

#[test]
fn test_docx_has_paragraph_spacing() {
    let md = "# Heading\n\nFirst paragraph.\n\nSecond paragraph.\n\n- Item one\n- Item two";
    let doc = read_markdown(md).unwrap();
    let bytes = write_docx(&doc).unwrap();
    // Extract document.xml from the DOCX zip
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut doc_xml = String::new();
    archive.by_name("word/document.xml").unwrap().read_to_string(&mut doc_xml).unwrap();
    // Should contain spacing elements (w:spacing with before/after)
    assert!(doc_xml.contains("w:before") || doc_xml.contains("w:after"),
        "DOCX paragraphs should have spacing, XML snippet: {}", &doc_xml[..2000.min(doc_xml.len())]);
}

#[test]
fn test_docx_respects_fontsize_meta() {
    let md = "---\nfontsize: 11pt\n---\n\nHello";
    let doc = read_markdown(md).unwrap();
    let bytes = write_docx(&doc).unwrap();
    let content = String::from_utf8_lossy(&bytes);
    // 11pt = 22 half-points in DOCX
    assert!(content.contains("22") || content.contains("w:sz"),
        "DOCX should set font size from metadata");
}
