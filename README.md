# rs-top (remote-server-top)

`rs-top` is a lightweight, agentless, and read-only remote system monitor. It provides a real-time TUI dashboard for monitoring multiple remote hosts simultaneously via SSH. Heavily influenced by classic tools like `top`, `htop`, and `btop`, it aims to provide a similar experience for remote server clusters.

![rs-top screenshot](https://git.ruijiang.me/rui/rs-top/raw/branch/screenshot/screenshot.png)

## Key Features

- **Agentless**: No software installation is required on the remote hosts. It uses standard Linux tools (like `top`, `procfs`, and `systemctl`) already present on most systems.
- **Lightweight**: Written in Rust, utilizing `tokio` for asynchronous task management and `ratatui` for a responsive terminal UI.
- **Read-Only**: The tool only fetches system statistics and does not perform any modifications to the remote hosts.
- **SSH-Based**: Relies on the host's native `ssh` binary and configuration (e.g., `~/.ssh/config`, `known_hosts`). It seamlessly integrates with your existing SSH setup, including identity files and multiplexing.

## Configuration

`rs-top` looks for a configuration file named `rs-top.toml` in your system's standard configuration directory (e.g., `~/.config/rs-top.toml` on Linux).

### Example `rs-top.toml`

```toml
[[hosts]]
address = "192.168.1.10"
user = "admin"
port = 22

[[hosts]]
address = "my-web-server.com"
user = "root"
identity_file = "/home/user/.ssh/id_ed25519"

[[hosts]]
address = "backup-node"
# Uses default user and port (22) if omitted
```

## Controls

| Key | Action |
| --- | --- |
| `Tab` | Focus next host |
| `BackTab` | Focus previous host |
| `j` or `Down` | Scroll process list down |
| `k` or `Up` | Scroll process list up |
| `h` or `Left` | Scroll failed units horizontally left |
| `l` or `Right` | Scroll failed units horizontally right |
| `q` | Quit |

## Requirements

- Local machine: `ssh` binary installed and accessible in the system path.
- Remote hosts: Standard Linux environment with access to `/proc` and `systemctl` (for failed units monitoring).

## Contributing

Contributions are welcome! Whether it's reporting bugs, suggesting features, or submitting pull requests, your help is appreciated. Please feel free to open an issue or submit a PR on GitHub.
