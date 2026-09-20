use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init_logger() {
    let log_path = crate::config::get_rsborg_dir().join("rsborg.log");

    // Rotate if file is larger than 5MB
    if let Ok(meta) = fs::metadata(&log_path)
        && meta.len() > 5 * 1024 * 1024
    {
        let old_path = crate::config::get_rsborg_dir().join("rsborg.log.1");
        let _ = fs::rename(&log_path, old_path);
    }

    {
        let mut lock = LOG_FILE.lock().unwrap();
        *lock = Some(log_path);
    }

    log_entry(
        "INFO",
        &format!(
            "=== RsBorg v{} started (PID: {}) ===",
            env!("CARGO_PKG_VERSION"),
            std::process::id()
        ),
    );
}

pub fn log_entry(level: &str, message: &str) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("[{}] [{}] {}\n", now, level, message);

    let path = {
        let lock = LOG_FILE.lock().unwrap();
        lock.clone()
    };

    let log_path = match path {
        Some(p) => p,
        None => crate::config::get_rsborg_dir().join("rsborg.log"),
    };

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = file.write_all(line.as_bytes());
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::log_entry("INFO", &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::log_entry("WARN", &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::log_entry("ERROR", &format!($($arg)*))
    };
}

pub fn read_log_entries() -> Vec<String> {
    let log_path = crate::config::get_rsborg_dir().join("rsborg.log");
    if let Ok(content) = fs::read_to_string(&log_path) {
        content.lines().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    }
}

pub fn clear_log_file() -> Result<(), String> {
    let log_path = crate::config::get_rsborg_dir().join("rsborg.log");
    if let Ok(mut f) = fs::File::create(&log_path) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(f, "[{}] [INFO] === Logs limpos pelo usuário via RsBorg ===", now);
        Ok(())
    } else {
        Err("Não foi possível limpar o arquivo de log".to_string())
    }
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::log_entry("DEBUG", &format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_writes_entries() {
        init_logger();
        log_info!("Test info message: {}", 42);
        log_warn!("Test warn message");
        log_error!("Test error message");

        let log_path = crate::config::get_rsborg_dir().join("rsborg.log");
        assert!(log_path.exists());
        let content = fs::read_to_string(log_path).unwrap();
        assert!(content.contains("Test info message: 42"));
        assert!(content.contains("Test warn message"));
        assert!(content.contains("Test error message"));
    }
}
