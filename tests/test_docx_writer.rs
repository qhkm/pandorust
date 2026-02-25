use pandorust::readers::markdown::read_markdown;
use pandorust::writers::docx::write_docx;

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
