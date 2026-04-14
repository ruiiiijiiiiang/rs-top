use futures::future::join_all;
use openssh::{KnownHosts, Session};
use std::error::Error;
use tokio::time::{Duration, sleep};

const DELIMITER: &str = "---SECTION---";

async fn fetch_stats(
    host: &str,
) -> Result<(String, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let session = Session::connect(host, KnownHosts::Strict).await?;

    let delim = format!("{}\n", DELIMITER);
    let cmd = format!(
        "cat /proc/meminfo; echo '{d}'; cat /proc/stat; echo '{d}'; cat /proc/uptime",
        d = DELIMITER
    );

    let output = session.command("sh").arg("-c").arg(&cmd).output().await?;

    session.close().await?;

    let stdout = String::from_utf8(output.stdout)?;
    let mut sections = stdout.splitn(3, &delim);

    let meminfo = sections.next().unwrap_or("").trim().to_string();
    let stat = sections.next().unwrap_or("").trim().to_string();
    let uptime = sections.next().unwrap_or("").trim().to_string();

    Ok((meminfo, stat, uptime))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let hosts = ["vm-app", "vm-network", "vm-monitor", "vm-public"];

    loop {
        let tasks: Vec<_> = hosts
            .iter()
            .map(|host| {
                let host = host.to_string();
                tokio::task::spawn(async move { fetch_stats(&host).await })
            })
            .collect();

        let results = join_all(tasks).await;

        for (host, result) in hosts.iter().zip(results) {
            match result {
                Ok(Ok((meminfo, stat, uptime))) => {
                    println!("=== {} ===", host);
                    println!("meminfo: {}", meminfo.lines().next().unwrap_or(""));
                    println!("stat:    {}", stat.lines().next().unwrap_or(""));
                    println!("uptime:  {}", uptime);
                }
                Ok(Err(e)) => eprintln!("{}: command error: {}", host, e),
                Err(e) => eprintln!("{}: task panicked: {}", host, e),
            }
        }

        sleep(Duration::from_secs(2)).await;
    }
}
