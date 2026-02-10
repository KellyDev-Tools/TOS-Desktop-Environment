use crate::navigation::zoom::ZoomLevel;

pub struct StatusBar {
    pub user: String,
    pub host: String,
    pub uptime_secs: u64,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            user: "tim".to_string(),
            host: "tos-navigator".to_string(),
            uptime_secs: 0,
        }
    }

    pub fn tick(&mut self) {
        self.uptime_secs += 1;
    }

    pub fn render_html(&self, level: ZoomLevel, sector: Option<usize>) -> String {
        let level_str = match level {
            ZoomLevel::Level1Root => "ROOT",
            ZoomLevel::Level2Sector => "SECTOR",
            ZoomLevel::Level3Focus => "FOCUS",
            ZoomLevel::Level3aPicker => "PICKER",
            ZoomLevel::Level3Split => "SPLIT",
        };

        let sector_str = sector.map_or("---".to_string(), |s| s.to_string());

        let hours = self.uptime_secs / 3600;
        let minutes = (self.uptime_secs % 3600) / 60;
        let seconds = self.uptime_secs % 60;

        format!(
            r#"<div class="lcars-status-bar">
                <div class="status-segment user-host">{0}@{1}</div>
                <div class="status-segment zoom-path">LOC: {2} / SEC: {3}</div>
                <div class="status-segment uptime">UPTIME: {4:02}:{5:02}:{6:02}</div>
                <div class="status-segment terminal-entry">
                    <span class="prompt">></span>
                    <input type="text" id="terminal-input" placeholder="ENTER COMMAND..." autocomplete="off">
                </div>
            </div>"#,
            self.user, self.host, level_str, sector_str, hours, minutes, seconds
        )
    }
}
