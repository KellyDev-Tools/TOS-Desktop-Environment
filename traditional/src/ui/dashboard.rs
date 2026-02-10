// Based on "Dashboard and UI Components.md"

pub trait Widget {
    fn render_html(&self) -> String;
    fn update(&mut self);
    fn get_name(&self) -> &str;
}

pub struct Dashboard {
    pub widgets: Vec<Box<dyn Widget>>,
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }

    pub fn render_all_html(&self) -> String {
        let mut html = String::new();
        for widget in &self.widgets {
            html.push_str(&widget.render_html());
        }
        html
    }
}

// Example Widgets

pub struct ClockWidget;
impl Widget for ClockWidget {
    fn render_html(&self) -> String {
        format!(r#"<div class="lcars-widget clock">
            <h3>CLOCK</h3>
            <p>12:42 PM</p>
        </div>"#)
    }
    fn update(&mut self) {
        // Update time logic
    }
    fn get_name(&self) -> &str {
        "Clock"
    }
}

pub struct SystemMonitorWidget {
    pub cpu_usage: u8,
    pub ram_usage: u8,
}

impl Widget for SystemMonitorWidget {
    fn render_html(&self) -> String {
        format!(r#"<div class="lcars-widget monitor">
            <h3>SYSTEM STATUS</h3>
            <div class="metric">CPU: {}%</div>
            <div class="metric">RAM: {}%</div>
        </div>"#, self.cpu_usage, self.ram_usage)
    }
    fn update(&mut self) {
        // Update stats logic
        self.cpu_usage = (self.cpu_usage + 5) % 100; // Mock update
    }
    fn get_name(&self) -> &str {
        "System Monitor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockWidget;
    impl Widget for MockWidget {
        fn render_html(&self) -> String {
            "<div>Mock</div>".to_string()
        }
        fn update(&mut self) {}
        fn get_name(&self) -> &str { "Mock" }
    }

    #[test]
    fn test_dashboard_aggregation() {
        let mut dash = Dashboard::new();
        dash.add_widget(Box::new(MockWidget));
        dash.add_widget(Box::new(MockWidget));
        
        // Should contain two "<div>Mock</div>"
        let html = dash.render_all_html();
        assert_eq!(html, "<div>Mock</div><div>Mock</div>");
    }

    #[test]
    fn test_system_monitor_update() {
        let mut monitor = SystemMonitorWidget { cpu_usage: 10, ram_usage: 20 };
        monitor.update(); // +5%
        assert_eq!(monitor.cpu_usage, 15);
        
        let html = monitor.render_html();
        assert!(html.contains("CPU: 15%"));
    }
}
