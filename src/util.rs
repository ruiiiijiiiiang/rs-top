pub fn format_bytes(bytes: f64) -> (f64, &'static str) {
    if bytes >= 1024.0 * 1024.0 * 1024.0 {
        (bytes / (1024.0 * 1024.0 * 1024.0), "G")
    } else if bytes >= 1024.0 * 1024.0 {
        (bytes / (1024.0 * 1024.0), "M")
    } else if bytes >= 1024.0 {
        (bytes / 1024.0, "K")
    } else {
        (bytes, "B")
    }
}

pub struct NetworkData(pub Vec<(f64, f64)>, pub Vec<(f64, f64)>, pub f64);

pub fn prepare_network_data(rx: &[f64], tx: &[f64]) -> NetworkData {
    let rx_data: Vec<(f64, f64)> = rx.iter().enumerate().map(|(i, &v)| (i as f64, v)).collect();
    let tx_data: Vec<(f64, f64)> = tx.iter().enumerate().map(|(i, &v)| (i as f64, v)).collect();

    let mut max_net: f64 = 1.0;
    for &(_, y) in &rx_data {
        max_net = max_net.max(y);
    }
    for &(_, y) in &tx_data {
        max_net = max_net.max(y);
    }

    NetworkData(rx_data, tx_data, max_net)
}

pub fn format_load_avg(load: (f64, f64, f64)) -> String {
    format!("{:.2} {:.2} {:.2}", load.0, load.1, load.2)
}

pub fn format_rate(rate: f64) -> String {
    let (val, unit) = format_bytes(rate);
    format!("{:.1} {}/s", val, unit)
}
