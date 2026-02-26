#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MorphPhase {
    Static,
    Entering, // Zooming in
    Exiting,  // Zooming out
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecorationStyle {
    Default,
    Minimal,
    Full,
    Warning,
}

pub struct DecorationManager;

impl DecorationManager {
    pub fn get_html_frame(
        title: &str, 
        style: DecorationStyle, 
        phase: MorphPhase, 
        content_id: &str
    ) -> String {
        let accent_color = match style {
            DecorationStyle::Warning => "var(--lcars-alert)",
            _ => "var(--lcars-accent)",
        };

        let phase_class = match phase {
            MorphPhase::Entering => "morph-entering",
            MorphPhase::Exiting => "morph-exiting",
            MorphPhase::Static => "morph-static",
        };

        let style_class = match style {
            DecorationStyle::Minimal => "frame-minimal",
            DecorationStyle::Full => "frame-full",
            DecorationStyle::Warning => "frame-warning",
            _ => "frame-default",
        };

        format!(
            r#"<div class="lcars-window-frame {} {}" style="--frame-accent: {};">
                <div class="lcars-frame-elbow"></div>
                <div class="lcars-frame-header">
                    <span class="lcars-title">{}</span>
                    <div class="lcars-header-fill"></div>
                </div>
                <div class="lcars-frame-side">
                    <div class="lcars-side-bar"></div>
                </div>
                <div id="{}" class="lcars-window-content">
                    <!-- Surface content injected here -->
                </div>
                <div class="lcars-frame-footer"></div>
            </div>"#,
            style_class, phase_class, accent_color, title, content_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morph_classes() {
        let html = DecorationManager::get_html_frame(
            "Terminal", 
            DecorationStyle::Default, 
            MorphPhase::Entering, 
            "s1"
        );
        assert!(html.contains("morph-entering"));
        assert!(html.contains("frame-default"));
    }
}
