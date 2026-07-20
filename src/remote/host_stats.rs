use std::{error::Error, sync::Arc};

use openssh::Session;

const DELIMITER: &str = "---SECTION---";

const MEM_CMD: &str = "cat /proc/meminfo | head -n 3";
const CPU_CMD: &str = "cat /proc/stat | head -n 1";
const UPTIME_CMD: &str = "cat /proc/uptime";
const DISK_CMD: &str = "df -B1 / | tail -n 1";
const IP_CMD: &str = "echo $SSH_CONNECTION";
const LOAD_AVG_CMD: &str = "cat /proc/loadavg";
const NET_CMD: &str = "cat /proc/net/dev";
const TOP_CMD: &str = "top -bn1 -w 512 | head -n 57 | tail -n 51";
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
    pub load_avg: (f64, f64, f64),
    pub net_rx: u64,
    pub net_tx: u64,
    pub uptime: String,
    pub ip_address: String,
    pub processes: Vec<String>,
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
            LOAD_AVG_CMD,
            NET_CMD,
            TOP_CMD,
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
            stats.parse_load_avg(out.trim());
        }
        if let Some(out) = sections.next() {
            stats.parse_net(out.trim());
        }
        if let Some(out) = sections.next() {
            stats.parse_processes(out.trim());
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

    fn parse_load_avg(&mut self, loadavg_out: &str) {
        let parts: Vec<&str> = loadavg_out.split_whitespace().collect();
        if parts.len() >= 3 {
            self.load_avg = (
                parts[0].parse().unwrap_or(0.0),
                parts[1].parse().unwrap_or(0.0),
                parts[2].parse().unwrap_or(0.0),
            );
        }
    }

    fn parse_net(&mut self, net_out: &str) {
        let mut rx: u64 = 0;
        let mut tx: u64 = 0;
        for line in net_out.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 && !parts[0].starts_with("lo") {
                rx += parts[1].parse().unwrap_or(0);
                tx += parts[9].parse().unwrap_or(0);
            }
        }
        self.net_rx = rx;
        self.net_tx = tx;
    }

    fn parse_processes(&mut self, top_out: &str) {
        self.processes = top_out
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    fn parse_failed_units(&mut self, failed_units_out: &str) {
        self.failed_units = failed_units_out
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::HostStats;

    #[test]
    fn parse_mem_extracts_total_and_available_memory() {
        let mut stats = HostStats::default();
        stats.parse_mem(
            "MemTotal:       32768000 kB\n\
             MemFree:         1024000 kB\n\
             MemAvailable:   24576000 kB\n",
        );

        assert_eq!(stats.mem_total, 32_768_000);
        assert_eq!(stats.mem_available, 24_576_000);
    }

    #[test]
    fn parse_mem_defaults_missing_or_invalid_values_to_zero() {
        let mut stats = HostStats {
            mem_total: 1,
            mem_available: 1,
            ..Default::default()
        };
        stats.parse_mem("MemTotal:\nMemAvailable: not-a-number\n");

        assert_eq!(stats.mem_total, 0);
        assert_eq!(stats.mem_available, 0);
    }

    #[test]
    fn parse_cpu_sums_fields_and_tracks_idle_column() {
        let mut stats = HostStats::default();
        stats.parse_cpu("cpu  10 20 30 40 50 60 70 80 90 100");

        assert_eq!(stats.cpu_total, 550);
        assert_eq!(stats.cpu_idle, 40);
    }

    #[test]
    fn parse_cpu_treats_invalid_numbers_as_zero() {
        let mut stats = HostStats::default();
        stats.parse_cpu("cpu  10 20 nope 40");

        assert_eq!(stats.cpu_total, 70);
        assert_eq!(stats.cpu_idle, 40);
    }

    #[test]
    fn parse_uptime_formats_multiple_ranges() {
        let mut stats = HostStats::default();
        stats.parse_uptime("59.9 0.0");
        assert_eq!(stats.uptime, "59s");

        stats.parse_uptime("121.0 0.0");
        assert_eq!(stats.uptime, "2m 1s");

        stats.parse_uptime("3661.0 0.0");
        assert_eq!(stats.uptime, "1h 1m 1s");

        stats.parse_uptime("90061.0 0.0");
        assert_eq!(stats.uptime, "1d 1h 1m 1s");
    }

    #[test]
    fn parse_disk_extracts_total_and_used_bytes() {
        let mut stats = HostStats::default();
        stats.parse_disk("/dev/sda1 1000000000 250000000 750000000 25% /\n");

        assert_eq!(stats.disk_total, 1_000_000_000);
        assert_eq!(stats.disk_used, 250_000_000);
    }

    #[test]
    fn parse_disk_ignores_short_lines() {
        let mut stats = HostStats {
            disk_total: 1,
            disk_used: 1,
            ..Default::default()
        };
        stats.parse_disk("filesystem only\n");

        assert_eq!(stats.disk_total, 1);
        assert_eq!(stats.disk_used, 1);
    }

    #[test]
    fn parse_ip_uses_remote_address_from_ssh_connection() {
        let mut stats = HostStats::default();
        stats.parse_ip("192.168.1.10 53124 10.0.0.5 22");

        assert_eq!(stats.ip_address, "10.0.0.5");
    }

    #[test]
    fn parse_ip_defaults_to_unknown_when_missing() {
        let mut stats = HostStats::default();
        stats.parse_ip("127.0.0.1");

        assert_eq!(stats.ip_address, "Unknown");
    }

    #[test]
    fn parse_load_avg_reads_first_three_values() {
        let mut stats = HostStats::default();
        stats.parse_load_avg("0.55 1.23 4.56 2/123 4567");

        assert_eq!(stats.load_avg, (0.55, 1.23, 4.56));
    }

    #[test]
    fn parse_load_avg_defaults_invalid_values_to_zero() {
        let mut stats = HostStats::default();
        stats.parse_load_avg("0.55 nope 4.56");

        assert_eq!(stats.load_avg, (0.55, 0.0, 4.56));
    }

    #[test]
    fn parse_net_ignores_loopback_and_sums_other_interfaces() {
        let mut stats = HostStats::default();
        stats.parse_net(
            "Inter-|   Receive                                                |  Transmit\n\
             face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n\
             lo: 100 0 0 0 0 0 0 0 200 0 0 0 0 0 0 0\n\
             eth0: 300 0 0 0 0 0 0 0 400 0 0 0 0 0 0 0\n\
             wlan0: 500 0 0 0 0 0 0 0 600 0 0 0 0 0 0 0",
        );

        assert_eq!(stats.net_rx, 800);
        assert_eq!(stats.net_tx, 1_000);
    }

    #[test]
    fn parse_processes_and_failed_units_drop_empty_lines() {
        let mut stats = HostStats::default();
        stats.parse_processes("PID USER\n123 root top\n\n456 admin sshd");
        stats.parse_failed_units("foo.service\n\n bar.service \n");

        assert_eq!(
            stats.processes,
            vec![
                "PID USER".to_string(),
                "123 root top".to_string(),
                "456 admin sshd".to_string()
            ]
        );
        assert_eq!(
            stats.failed_units,
            vec!["foo.service".to_string(), "bar.service".to_string()]
        );
    }
}
