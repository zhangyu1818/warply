use super::*;
use crate::clipboard::{ClipboardContent, ImageData};

// ============================================================================
// FILENAME EXTRACTION TESTS
// ============================================================================

#[test]
fn test_extract_filename_from_html() {
    // Test extraction from src attribute with file:// URL (common on macOS)
    let html1 = r##"<img src="file:///Users/test/Pictures/screenshot.png" alt="Screenshot">"##;
    let filename = extract_filename_from_html(html1);
    assert_eq!(filename, Some("screenshot.png".to_string()));

    // Test extraction from src attribute with http URL
    let html2 = r##"<img src="https://example.com/images/photo.jpg" alt="Photo">"##;
    let filename = extract_filename_from_html(html2);
    assert_eq!(filename, Some("photo.jpg".to_string()));

    // Test extraction from title attribute
    let html3 = r##"<img title="document.gif" src="data:image/gif;base64,R0lGOD...">"##;
    let filename = extract_filename_from_html(html3);
    assert_eq!(filename, Some("document.gif".to_string()));

    // Test extraction from alt attribute
    let html4 = r##"<img alt="image.webp" src="data:image/webp;base64,UklGR...">"##;
    let filename = extract_filename_from_html(html4);
    assert_eq!(filename, Some("image.webp".to_string()));

    // Test extraction from free text
    let html5 = r##"<div>Here is my image: myfile.jpeg that I copied</div>"##;
    let filename = extract_filename_from_html(html5);
    assert_eq!(filename, Some("myfile.jpeg".to_string()));

    // Test no filename found
    let html6 = r##"<div>Just some text with no image references</div>"##;
    let filename = extract_filename_from_html(html6);
    assert_eq!(filename, None);

    // Test non-image extension ignored
    let html7 = r##"<div>document.pdf and archive.zip should be ignored</div>"##;
    let filename = extract_filename_from_html(html7);
    assert_eq!(filename, None);

    // Test case-insensitive extension matching
    let html8 = r##"<img src="test.PNG" alt="Test">"##;
    let filename = extract_filename_from_html(html8);
    assert_eq!(filename, Some("test.PNG".to_string()));

    // Test extraction with various punctuation
    let html9 = r##"<div>Look at "my-image.jpg", (another.gif), or <file.webp>!</div>"##;
    let filename = extract_filename_from_html(html9);
    // Should find the first one
    assert_eq!(filename, Some("my-image.jpg".to_string()));
}

#[test]
fn test_extract_filename_from_text() {
    // Test full file path
    let file_path = "/Users/test/Documents/screenshot.png";
    let result = extract_filename_from_text(file_path);
    assert_eq!(result, Some("screenshot.png".to_string()));

    // Test file:// URL
    let file_url = "file:///Users/test/screenshot.gif";
    let result = extract_filename_from_text(file_url);
    assert_eq!(result, Some("screenshot.gif".to_string()));

    // Test multiline with file path
    let multiline = "Some text\n/path/to/image.webp\nMore text";
    let result = extract_filename_from_text(multiline);
    assert_eq!(result, Some("image.webp".to_string()));

    // Test non-image file (should return None)
    let text_file = "/Users/test/document.txt";
    let result = extract_filename_from_text(text_file);
    assert_eq!(result, None);

    // Test no file path
    let plain_text = "Just some plain text";
    let result = extract_filename_from_text(plain_text);
    assert_eq!(result, None);

    // Test just filename
    let just_filename = "my-screenshot.png";
    let result = extract_filename_from_text(just_filename);
    assert_eq!(result, Some("my-screenshot.png".to_string()));

    // Test empty string
    let empty = "";
    let result = extract_filename_from_text(empty);
    assert_eq!(result, None);
}

#[test]
fn test_extract_filename_from_clipboard_content() {
    // Test HTML takes precedence over text
    let html_content = Some(r##"<img src="test.png" alt="Test">"##.to_string());
    let text_content = "other-file.jpg";
    let result = extract_filename_from_clipboard_content(&html_content, text_content);
    assert_eq!(result, Some("test.png".to_string()));

    // Test fallback to text when HTML has no filename
    let html_content = Some("<div>No images here</div>".to_string());
    let text_content = "/path/to/image.gif";
    let result = extract_filename_from_clipboard_content(&html_content, text_content);
    assert_eq!(result, Some("image.gif".to_string()));

    // Test fallback to text when no HTML
    let html_content = None;
    let text_content = "screenshot.webp";
    let result = extract_filename_from_clipboard_content(&html_content, text_content);
    assert_eq!(result, Some("screenshot.webp".to_string()));

    // Test no filename found
    let html_content = Some("<div>Just text</div>".to_string());
    let text_content = "No images here either";
    let result = extract_filename_from_clipboard_content(&html_content, text_content);
    assert_eq!(result, None);
}

// ============================================================================
// CLIPBOARD CONTENT STRUCTURE TESTS
// ============================================================================

#[test]
fn test_clipboard_content_with_images() {
    let image_data = ImageData {
        data: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        mime_type: "image/png".to_string(),
        filename: Some("test.png".to_string()),
    };

    let content = ClipboardContent {
        plain_text: "Test text".to_string(),
        html: Some(r##"<img src="test.png">"##.to_string()),
        images: Some(vec![image_data.clone()]),
        paths: None,
    };

    assert!(!content.is_empty());
    assert!(content.images.is_some());
    assert_eq!(content.images.as_ref().unwrap().len(), 1);
    assert_eq!(content.images.as_ref().unwrap()[0].mime_type, "image/png");
}
