use std::process::Command;

/// Check and report system file descriptor limits
pub fn check_file_descriptor_limits() {
    #[cfg(unix)]
    {
        // Check current limits on Unix systems
        match Command::new("sh").arg("-c").arg("ulimit -n").output() {
            Ok(output) => {
                if output.status.success() {
                    let limit_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if let Ok(limit) = limit_str.parse::<u32>() {
                        println!("📊 Current file descriptor limit: {}", limit);
                        
                        if limit < 1024 {
                            println!("⚠️  File descriptor limit is low ({}). Consider increasing with 'ulimit -n 4096'", limit);
                        }
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Could not check file descriptor limits: {}", e);
            }
        }
    }
}

/// Get current process file descriptor usage (Unix only)
#[cfg(unix)]
pub fn get_current_fd_count() -> Option<usize> {
    use std::fs;
    
    // On Linux and macOS, count open file descriptors in /proc/self/fd or /dev/fd
    let fd_paths = ["/proc/self/fd", "/dev/fd"];
    
    for fd_path in &fd_paths {
        if let Ok(entries) = fs::read_dir(fd_path) {
            let count = entries.count();
            if count > 0 {
                return Some(count);
            }
        }
    }
    
    None
}

/// Monitor and report file descriptor usage
pub fn monitor_fd_usage() {
    #[cfg(unix)]
    {
        if let Some(count) = get_current_fd_count() {
            
            if count > 500 {
                println!("⚠️  High file descriptor usage detected: {} open files", count);
            }
        }
    }
}

/// Check system resources and print diagnostics
pub fn system_diagnostics() {
    println!("🔍 System Diagnostics:");
    check_file_descriptor_limits();
    monitor_fd_usage();
}