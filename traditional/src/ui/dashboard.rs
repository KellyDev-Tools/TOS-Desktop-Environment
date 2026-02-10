// Based on "Dashboard and UI Components.md"

pub trait Widget: std::any::Any {
    fn render_html(&self) -> String;
    fn update(&mut self);
    fn get_name(&self) -> &str;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
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
    fn update(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
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
        self.cpu_usage = (self.cpu_usage + 5) % 100;
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn get_name(&self) -> &str {
        "System Monitor"
    }
}

pub struct ProcessManagerWidget {
    pub processes: Vec<String>,
}

impl Widget for ProcessManagerWidget {
    fn render_html(&self) -> String {
        let mut list_html = String::new();
        if self.processes.is_empty() {
            list_html.push_str("<p>NO ACTIVE PROCESSES</p>");
        } else {
            for proc in &self.processes {
                list_html.push_str(&format!(r#"<div class="process-item">{}</div>"#, proc));
            }
        }

        format!(r#"<div class="lcars-widget processes">
            <h3>PROCESS MANAGER</h3>
            {}
        </div>"#, list_html)
    }
    fn update(&mut self) {
        // Updated externally by the DesktopEnvironment
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn get_name(&self) -> &str {
        "Process Manager"
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
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        fn get_name(&self) -> &str { "Mock" }
    }

    #[test]
    fn test_process_manager_widget() {
        let mut pw = ProcessManagerWidget { processes: vec![] };
        assert!(pw.render_html().contains("NO ACTIVE PROCESSES"));
        
        pw.processes.push("Terminal".to_string());
        assert!(pw.render_html().contains("Terminal"));
    }
}
