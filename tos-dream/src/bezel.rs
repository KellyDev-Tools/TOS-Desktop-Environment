use crate::Viewport;

impl crate::TosState {
    /// Render the tactical bezel overlay based on expansion state.
    /// Returns HTML string that can be appended to rendered viewport content.
    fn render_bezel(&self, viewport: &Viewport) -> String {
        // Base colors / styling
        let accent_color = self.get_bezel_color();

        // Determine what to show
        if !viewport.bezel_expanded {
            // Collapsed bar only
            let title = {
                let sector = &self.sectors[viewport.sector_index];
                sector.name.clone()
            };

            let html = format!(
                r#"<div class="bezel-collapsed-bar" style="--bezel-accent:{color};">
                    <span class="bezel-title">{title}</span>
                    <button class="bezel-toggle" onclick="window.ipc.postMessage('toggle_bezel')">▼</button>
                </div>"#,
                color = accent_color,
                title = title
            );
            html
        } else {
            // Fully expanded bezel with controls
            let sector = &self.sectors[viewport.sector_index];
            let _hub = &sector.hubs[viewport.hub_index];

            // Determine priority indicator class
            let priority_class = match sector.priority_score(self) {
                s if s >= 8.0 => "high",
                s if s >= 5.0 => "medium",
                _ => "low",
            };

            // Sector name for title
            let title = sector.name.clone();

            let html = format!(
                r#"<div class="tactical-bezel overlay" style="--bezel-accent:{color};">
                    <div class="bezel-header collapsed">
                        <button class="bezel-btn" onclick="window.ipc.postMessage('zoom_out')">▼</button>
                        <span class="bezel-title">{title}</span>
                        <button class="bezel-btn" onclick="window.ipc.postMessage('toggle_bezel')">▼</button>
                    </div>
                    <div class="bezel-content expanded">
                        <div class="bezel-navigation">
                            <button class="bezel-btn" onclick="window.ipc.postMessage('zoom_out')">Zoom Out</button>
                            <button class="bezel-btn" onclick="window.ipc.postMessage('split_view')">Split</button>
                            <button class="bezel-btn" onclick="window.ipc.postMessage('teleport')">Teleport</button>
                            <button class="bezel-btn" onclick="window.ipc.postMessage('close_application')">Close</button>
                        </div>
                        <div class="bezel-tools">
                            <div class="priority-indicator priority-{priority_class}"></div>
                            <div class="sector-meta">
                                <span class="sector-name">{title}</span>
                                <span class="sector-type">{type_name}</span>
                            </div>
                        </div>
                        <div class="bezel-meta">
                            <!-- placeholder for future meta -->
                        </div>
                    </div>
                </div>"#,
                color = accent_color,
                title = title,
                priority_class = priority_class,
                type_name = sector.sector_type_name
            );
            html
        }
    }

    fn get_bezel_color(&self) -> String {
        // Return CSS variable for accent color based on current context
        // For simplicity, return a transparent default that can be overridden via CSS
        "0".to_string()
    }
}
