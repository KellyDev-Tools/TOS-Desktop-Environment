use std::fs;
use std::io;

pub struct ProcessStats {
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub uptime_seconds: f32,
    pub cmdline: String,
    pub uid: u32,
}

pub fn get_process_buffer_sample(pid: u32) -> Vec<u8> {
    // Read command line and environment as "buffer" sample
    let mut buffer = Vec::new();
    
    if let Ok(cmd) = fs::read(format!("/proc/{}/cmdline", pid)) {
        buffer.extend_from_slice(&cmd);
    }
    buffer.push(0); // Separator
    
    // Append environment sample
    if let Ok(env) = fs::read(format!("/proc/{}/environ", pid)) {
        buffer.extend_from_slice(&env.iter().take(256).cloned().collect::<Vec<u8>>());
    }
    
    // Fill to 512 bytes if small
    if buffer.len() < 512 {
        buffer.resize(512, 0);
    }
    
    buffer
}

pub fn get_process_stats(pid: u32) -> io::Result<ProcessStats> {
    // Read /proc/[pid]/stat for CPU info
    // Read /proc/[pid]/status for Memory info
    
    // 1. Get Memory (VmRSS) & UID from /proc/[pid]/status
    let status_path = format!("/proc/{}/status", pid);
    let status_content = fs::read_to_string(status_path)?;
    
    let mut memory_bytes = 0;
    let mut uid = 0;
    
    for line in status_content.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let kb: u64 = parts[1].parse().unwrap_or(0);
                memory_bytes = kb * 1024;
            }
        } else if line.starts_with("Uid:") {
             let parts: Vec<&str> = line.split_whitespace().collect();
             if parts.len() >= 2 {
                 uid = parts[1].parse().unwrap_or(0);
             }
        }
    }
    
    // 2. Get CPU stats & Uptime from /proc/[pid]/stat
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(stat_path)?;
    let stat_parts: Vec<&str> = stat_content.split_whitespace().collect();
    
    let mut cpu_usage = 0.0;
    let mut uptime_seconds = 0.0;
    
    if stat_parts.len() >= 22 {
        let utime: u64 = stat_parts[13].parse().unwrap_or(0);
        let stime: u64 = stat_parts[14].parse().unwrap_or(0);
        let starttime: u64 = stat_parts[21].parse().unwrap_or(0);
        
        // Get system uptime
        if let Ok(uptime_content) = fs::read_to_string("/proc/uptime") {
            if let Some(uptime_str) = uptime_content.split_whitespace().next() {
                if let Ok(sys_uptime) = uptime_str.parse::<f32>() {
                    let hertz = 100.0; // Standard on Linux
                    let total_time = (utime + stime) as f32;
                    let seconds = sys_uptime - (starttime as f32 / hertz);
                    
                    if seconds > 0.0 {
                        cpu_usage = (total_time / hertz) / seconds * 100.0;
                        uptime_seconds = seconds;
                    }
                }
            }
        }
    }

    // 3. Get Command line for display
    let cmdline = fs::read_to_string(format!("/proc/{}/cmdline", pid))
        .unwrap_or_default()
        .replace('\0', " ");

    Ok(ProcessStats {
        pid,
        cpu_usage,
        memory_bytes,
        uptime_seconds,
        cmdline,
        uid,
    })
}

pub fn kill_process(pid: u32) {
    send_signal(pid, libc::SIGKILL);
}

pub fn send_signal(pid: u32, signal: i32) {
    unsafe {
        libc::kill(pid as libc::pid_t, signal);
    }
}
