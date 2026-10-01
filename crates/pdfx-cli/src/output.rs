use serde::Serialize;
use std::fmt::Display;

#[derive(Serialize, Debug)]
pub struct CliJsonSuccess<T: Serialize> {
    pub status: &'static str,
    pub command: &'static str,
    pub data: T,
}

#[derive(Serialize, Debug)]
pub struct CliJsonError {
    pub status: &'static str,
    pub command: &'static str,
    pub error: String,
}

pub fn print_json_success<T: Serialize>(command: &'static str, data: &T) {
    let resp = CliJsonSuccess {
        status: "success",
        command,
        data,
    };
    if let Ok(json) = serde_json::to_string_pretty(&resp) {
        println!("{}", json);
    }
}

pub fn print_json_error(command: &'static str, error: &str) {
    let resp = CliJsonError {
        status: "error",
        command,
        error: error.to_string(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&resp) {
        eprintln!("{}", json);
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

pub fn print_banner(title: &str) {
    println!("\n  {}", title);
    println!("  {}", "─".repeat(50));
}

pub fn print_kv(key: &str, val: impl Display) {
    println!("  {:<18} {}", key, val);
}

pub fn print_footer() {
    println!("  {}\n", "─".repeat(50));
}
