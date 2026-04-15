use openssh::Session;
use std::{error::Error, sync::Arc};

const DELIMITER: &str = "---SECTION---";

const MEM_CMD: &str = "cat /proc/meminfo | head -n 3";
const CPU_CMD: &str = "cat /proc/stat | head -n 1";
const UPTIME_CMD: &str = "cat /proc/uptime";
const DISK_CMD: &str = "df -B1 / | tail -n 1";
const IP_CMD: &str = "echo $SSH_CONNECTION";
const FAILED_UNITS_CMD: &str =
    "systemctl list-units --state=failed --no-legend --plain | awk '{print $1}'";

#[derive(Debug, Clone, Default)]
pub struct HostStats {
    pub mem_total: u64,
    pub mem_available: u64,
    pub cpu_total: u64,
    pub cpu_idle: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub uptime: String,
    pub ip_address: String,
    pub failed_units: Vec<String>,
}

impl HostStats {
    pub async fn fetch(session: Arc<Session>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let commands = [
            MEM_CMD,
            CPU_CMD,
            UPTIME_CMD,
            DISK_CMD,
            IP_CMD,
            FAILED_UNITS_CMD,
        ];

        let separator = format!("; echo '{}'; ", DELIMITER);
        let cmd = commands.join(&separator);

        let output = session.command("sh").arg("-c").arg(&cmd).output().await?;
        let stdout = String::from_utf8(output.stdout)?;

        let mut sections = stdout.split(DELIMITER);
        let mut stats = HostStats::default();

        if let Some(out) = sections.next() {
            stats.parse_mem(out);
        }
        if let Some(out) = sections.next() {
            stats.parse_cpu(out);
        }
        if let Some(out) = sections.next() {
            stats.parse_uptime(out.trim());
        }
        if let Some(out) = sections.next() {
            stats.parse_disk(out.trim());
        }
        if let Some(out) = sections.next() {
            stats.parse_ip(out.trim());
        }
        if let Some(out) = sections.next() {
            stats.parse_failed_units(out.trim());
        }

        Ok(stats)
    }

    fn parse_mem(&mut self, meminfo_out: &str) {
        for line in meminfo_out.lines() {
            if line.starts_with("MemTotal:") {
                self.mem_total = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                self.mem_available = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
            }
        }
    }

    fn parse_cpu(&mut self, stat_out: &str) {
        let parts: Vec<&str> = stat_out.split_whitespace().skip(1).collect();
        let mut total: u64 = 0;
        let mut idle: u64 = 0;
        for (i, part) in parts.iter().enumerate() {
            let val: u64 = part.parse().unwrap_or(0);
            total += val;
            if i == 3 {
                idle = val;
            }
        }
        self.cpu_total = total;
        self.cpu_idle = idle;
    }

    fn parse_uptime(&mut self, uptime_out: &str) {
        let seconds: f64 = uptime_out
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);

        let total_seconds = seconds as u64;
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;

        if days > 0 {
            self.uptime = format!("{}d {}h {}m {}s", days, hours, minutes, secs);
        } else if hours > 0 {
            self.uptime = format!("{}h {}m {}s", hours, minutes, secs);
        } else if minutes > 0 {
            self.uptime = format!("{}m {}s", minutes, secs);
        } else {
            self.uptime = format!("{}s", secs);
        }
    }

    fn parse_disk(&mut self, df_out: &str) {
        let df_parts: Vec<&str> = df_out.split_whitespace().collect();
        if df_parts.len() >= 3 {
            self.disk_total = df_parts[1].parse().unwrap_or(0);
            self.disk_used = df_parts[2].parse().unwrap_or(0);
        }
    }

    fn parse_ip(&mut self, connection_out: &str) {
        self.ip_address = connection_out
            .split_whitespace()
            .nth(2)
            .unwrap_or("Unknown")
            .to_string();
    }

    fn parse_failed_units(&mut self, failed_units_out: &str) {
        self.failed_units = failed_units_out
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
}
