use ratatui::style::palette::tailwind::{self, Palette};

pub const PALETTES: [Palette; 8] = [
    tailwind::EMERALD,
    tailwind::GREEN,
    tailwind::LIME,
    tailwind::YELLOW,
    tailwind::AMBER,
    tailwind::ORANGE,
    tailwind::RED,
    tailwind::ROSE,
];

pub fn get_palette(percentage: f64) -> &'static Palette {
    let num_segments = PALETTES.len();
    let segment_index = (percentage / 100.0 * num_segments as f64)
        .floor()
        .min(num_segments as f64 - 1.0) as usize;

    &PALETTES[segment_index]
}

pub struct NetworkData {
    pub rx_data: Vec<(f64, f64)>,
    pub tx_data: Vec<(f64, f64)>,
    pub max_net: f64,
    pub title: String,
}

impl NetworkData {
    pub fn y_labels(&self) -> Vec<String> {
        let (max_val, max_unit) = format_bytes(self.max_net);
        let max_label = if max_val.fract() == 0.0 {
            format!("{:.0}{}", max_val, max_unit)
        } else {
            format!("{:.1}{}", max_val, max_unit)
        };
        vec![max_label.clone(), "0".to_string(), max_label]
    }
}

pub fn prepare_network_data(rx: &[f64], tx: &[f64]) -> NetworkData {
    let rx_data: Vec<(f64, f64)> = rx.iter().enumerate().map(|(i, &v)| (i as f64, v)).collect();
    let tx_data: Vec<(f64, f64)> = tx.iter().enumerate().map(|(i, &v)| (i as f64, v)).collect();

    let mut max_val: f64 = 0.0;
    for &(_, y) in &rx_data {
        max_val = max_val.max(y);
    }
    for &(_, y) in &tx_data {
        max_val = max_val.max(y);
    }

    let max_net = if max_val > 0.0 {
        10.0_f64.powf(max_val.log10().floor() + 1.0)
    } else {
        1.0
    };

    let rx_latest = rx.last().copied().unwrap_or(0.0);
    let tx_latest = tx.last().copied().unwrap_or(0.0);
    let title = format!(
        "Net RX: {} TX: {}",
        format_rate(rx_latest),
        format_rate(tx_latest)
    );

    NetworkData {
        rx_data,
        tx_data,
        max_net,
        title,
    }
}

pub fn format_bytes(bytes: f64) -> (f64, &'static str) {
    if bytes >= 1000.0 * 1000.0 * 1000.0 {
        (bytes / (1000.0 * 1000.0 * 1000.0), "G")
    } else if bytes >= 1000.0 * 1000.0 {
        (bytes / (1000.0 * 1000.0), "M")
    } else if bytes >= 1000.0 {
        (bytes / 1000.0, "K")
    } else {
        (bytes, "B")
    }
}

pub fn format_mem_title(used: u64, total: u64) -> String {
    let used_gb = used as f64 / (1000.0 * 1000.0);
    let total_gb = total as f64 / (1000.0 * 1000.0);
    format!("RAM {:.1}G/{:.1}G", used_gb, total_gb)
}

pub fn format_load_avg(load: (f64, f64, f64)) -> String {
    format!("{:.2} {:.2} {:.2}", load.0, load.1, load.2)
}

pub fn format_rate(rate: f64) -> String {
    let (val, unit) = format_bytes(rate);
    format!("{:.1} {}/s", val, unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_uses_expected_units() {
        assert_eq!(format_bytes(999.0), (999.0, "B"));
        assert_eq!(format_bytes(1_000.0), (1.0, "K"));
        assert_eq!(format_bytes(1_000_000.0), (1.0, "M"));
        assert_eq!(format_bytes(1_000_000_000.0), (1.0, "G"));
    }

    #[test]
    fn prepare_network_data_builds_series_and_title() {
        let data = prepare_network_data(&[512.0, 1_500.0], &[100.0, 2_500.0]);

        assert_eq!(data.rx_data, vec![(0.0, 512.0), (1.0, 1_500.0)]);
        assert_eq!(data.tx_data, vec![(0.0, 100.0), (1.0, 2_500.0)]);
        assert_eq!(data.max_net, 10_000.0);
        assert_eq!(data.title, "Net RX: 1.5 K/s TX: 2.5 K/s");
        assert_eq!(data.y_labels(), vec!["10K", "0", "10K"]);
    }

    #[test]
    fn prepare_network_data_defaults_to_one_when_empty() {
        let data = prepare_network_data(&[], &[]);

        assert_eq!(data.max_net, 1.0);
        assert_eq!(data.title, "Net RX: 0.0 B/s TX: 0.0 B/s");
        assert_eq!(data.y_labels(), vec!["1B", "0", "1B"]);
    }

    #[test]
    fn formatting_helpers_are_stable() {
        assert_eq!(format_mem_title(2_500_000, 8_000_000), "RAM 2.5G/8.0G");
        assert_eq!(format_load_avg((0.25, 1.0, 12.345)), "0.25 1.00 12.35");
        assert_eq!(format_rate(1_500_000.0), "1.5 M/s");
    }
}
