// TOS Dashboard Widget Module
// Module: dashboard-widget
// Version: 1.0.0

const TOS = {
    name: "dashboard-widget",
    version: "1.0.0",
    
    // Widget state
    state: {
        cpu_usage: 0,
        memory_usage: 0,
        disk_usage: 0,
        last_update: null
    },
    
    // Called when module is loaded
    onLoad: function(tosState) {
        console.log(`Dashboard widget loaded: ${this.name}`);
        this.startMonitoring();
    },
    
    // Called when module is unloaded
    onUnload: function(tosState) {
        console.log(`Dashboard widget unloaded: ${this.name}`);
        this.stopMonitoring();
    },
    
    // Render override for dashboard level
    render: function(level) {
        if (level === "GlobalOverview") {
            return `
                <div class="dashboard-widget system-metrics">
                    <div class="widget-header">SYSTEM METRICS</div>
                    <div class="metric-row">
                        <span class="metric-label">CPU</span>
                        <span class="metric-value">${this.state.cpu_usage}%</span>
                        <div class="metric-bar">
                            <div class="metric-fill" style="width: ${this.state.cpu_usage}%"></div>
                        </div>
                    </div>
                    <div class="metric-row">
                        <span class="metric-label">MEM</span>
                        <span class="metric-value">${this.state.memory_usage}%</span>
                        <div class="metric-bar">
                            <div class="metric-fill" style="width: ${this.state.memory_usage}%"></div>
                        </div>
                    </div>
                    <div class="metric-row">
                        <span class="metric-label">DSK</span>
                        <span class="metric-value">${this.state.disk_usage}%</span>
                        <div class="metric-bar">
                            <div class="metric-fill" style="width: ${this.state.disk_usage}%"></div>
                        </div>
                    </div>
                    <div class="widget-footer">Last update: ${this.state.last_update || 'Never'}</div>
                </div>
            `;
        }
        return null;
    },
    
    // Start system monitoring
    startMonitoring: function() {
        // In a real implementation, this would read from /proc/stat, etc.
        this.updateMetrics();
        
        // Schedule periodic updates
        this.updateInterval = setInterval(() => {
            this.updateMetrics();
        }, 5000);
    },
    
    // Stop system monitoring
    stopMonitoring: function() {
        if (this.updateInterval) {
            clearInterval(this.updateInterval);
        }
    },
    
    // Update system metrics
    updateMetrics: function() {
        // Mock data - in real implementation, read from system
        this.state.cpu_usage = Math.floor(Math.random() * 100);
        this.state.memory_usage = Math.floor(Math.random() * 100);
        this.state.disk_usage = Math.floor(Math.random() * 100);
        this.state.last_update = new Date().toLocaleTimeString();
        
        // Notify TOS of state change
        this.notifyUpdate();
    },
    
    // Notify TOS that widget needs re-render
    notifyUpdate: function() {
        // This would trigger a re-render in the actual implementation
        console.log("Dashboard widget updated");
    },
    
    // Handle bezel actions
    bezelActions: function() {
        return ["refresh", "configure", "close"];
    },
    
    // Handle commands
    handleCommand: function(cmd) {
        switch(cmd) {
            case "refresh":
                this.updateMetrics();
                return "Metrics refreshed";
            case "configure":
                return "Opening configuration...";
            case "close":
                return "Closing widget...";
            default:
                return null;
        }
    }
};

// Export for TOS module system
if (typeof module !== 'undefined' && module.exports) {
    module.exports = TOS;
}
