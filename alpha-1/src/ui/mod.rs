pub mod render;

// Tactical Mini-Map Implementation
pub mod minimap;

/// Generates the JavaScript code to update the UI view safely.
/// Handles escaping of HTML content (backticks, quotes, etc.) by serializing to JSON.
pub fn generate_view_update_script(html: &str, level: crate::HierarchyLevel) -> String {
    // Escaping backticks and other special chars via JSON serialization
    let html_json = serde_json::to_string(html).unwrap_or_else(|_| String::from("\"Error\""));
    
    format!(
        r#"window.updateView({}, "{:?}");
           document.querySelectorAll('.terminal-output').forEach(el => el.scrollTop = el.scrollHeight);"#,
        html_json, level
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HierarchyLevel;

    #[test]
    fn test_generate_view_update_script_escapes_backticks() {
        // Setup input with characters that would break template literals
        let tricky_html = r#"<div class="test">Hello `World` ${injection}</div>
<script>const x = `inner backticks`;</script>"#;
        let level = HierarchyLevel::GlobalOverview;

        let script = generate_view_update_script(tricky_html, level);

        // Verify the HTML is JSON stringified (enclosed in quotes)
        // and NOT using backticks for the outer wrapper in the generated JS call
        assert!(script.contains("window.updateView(\"")); 
        
        // Verify content is preserved but escaped
        // JSON string of tricky_html should contain escaped backticks or just be a valid JSON string
        // The key is that `window.updateView` receives a valid string argument.
        
        // Check for presence of content
        assert!(script.contains("Hello `World`"));
        assert!(script.contains("inner backticks"));
        
        // Ensure no syntax error pattern like `window.updateView(`...`)`
        assert!(!script.contains("window.updateView(`"));
    }
}
