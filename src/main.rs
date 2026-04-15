pub mod app;
pub mod util;
use crate::app::App;

#[tokio::main]
async fn main() {
    let hosts: Vec<String> = ["vm-app", "vm-network", "vm-monitor", "vm-public"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let _ = App::new(hosts).start().await;
    std::process::exit(0);
}
