use openssh::Session;
use std::{error::Error, sync::Arc};

const DELIMITER: &str = "---SECTION---";

#[derive(Debug, Clone, Default)]
pub struct HostStats {
    pub mem_total: u64,
    pub mem_available: u64,
    pub cpu_total: u64,
    pub cpu_idle: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub uptime: String,
}

pub async fn fetch_stats(session: Arc<Session>) -> Result<HostStats, Box<dyn Error + Send + Sync>> {
    let delim = format!("{}\n", DELIMITER);
    let cmd = format!(
        "cat /proc/meminfo; echo '{d}'; cat /proc/stat; echo '{d}'; cat /proc/uptime; echo '{d}'; df -B1 / | tail -n 1",
        d = DELIMITER
    );

    let output = session.command("sh").arg("-c").arg(&cmd).output().await?;
    let stdout = String::from_utf8(output.stdout)?;

    let mut sections = stdout.splitn(4, &delim);
    let meminfo = sections.next().unwrap_or("");
    let stat = sections.next().unwrap_or("");
    let uptime_str = sections.next().unwrap_or("").trim();
    let df_out = sections.next().unwrap_or("").trim();

    let mut stats = HostStats {
        uptime: uptime_str
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .to_string(),
        ..Default::default()
    };

    // Parse meminfo
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            stats.mem_total = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            stats.mem_available = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
        }
    }

    // Parse stat (first line is total cpu)
    if let Some(line) = stat.lines().next() {
        let parts: Vec<&str> = line.split_whitespace().skip(1).collect();
        let mut total: u64 = 0;
        let mut idle: u64 = 0;
        for (i, part) in parts.iter().enumerate() {
            let val: u64 = part.parse().unwrap_or(0);
            total += val;
            if i == 3 {
                // idle is the 4th field
                idle = val;
            }
        }
        stats.cpu_total = total;
        stats.cpu_idle = idle;
    }

    // Parse df output: Filesystem 1B-blocks Used Available Use% Mounted on
    // tail -n 1 gives: /dev/sda1 123456 78910 111213 65% /
    let df_parts: Vec<&str> = df_out.split_whitespace().collect();
    if df_parts.len() >= 3 {
        stats.disk_total = df_parts[1].parse().unwrap_or(0);
        stats.disk_used = df_parts[2].parse().unwrap_or(0);
    }

    Ok(stats)
}
