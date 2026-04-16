mod model;
mod update;
mod view;

use std::{error::Error, sync::Arc, time::Duration};

use openssh::{KnownHosts, SessionBuilder};
use ratatui::{DefaultTerminal, crossterm::event};
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::{interval, timeout},
};

use crate::remote::host_stats::HostStats;

pub use model::{App, AppAction, ConnectionStatus, HostState};

const INTERVAL: u64 = 2;
const MAX_HISTORY: usize = 200;

impl App {
    pub fn new(hosts: Vec<crate::HostConfig>) -> Self {
        let current_user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        Self {
            running: true,
            hosts: hosts
                .into_iter()
                .map(|config| {
                    let user = config.user.as_deref().unwrap_or(&current_user);
                    let port = config.port.unwrap_or(22);
                    HostState {
                        name: format!("{}@{}:{}", user, config.address, port),
                        config: Some(config),
                        connection_status: ConnectionStatus::Connecting,
                        ..Default::default()
                    }
                })
                .collect(),
            ..Default::default()
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        color_eyre::install()?;
        let terminal = ratatui::init();

        let (tx, rx) = mpsc::channel(100);
        let current_user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        let mut background_tasks = self.spawn_connection_tasks(&tx, &current_user);
        background_tasks.push(Self::spawn_input_task(tx.clone()));

        let res = self.run(terminal, tx, rx, &mut background_tasks).await;

        ratatui::restore();
        self.shutdown(&mut background_tasks).await;

        res
    }

    async fn run(
        &mut self,
        mut terminal: DefaultTerminal,
        tx: mpsc::Sender<AppAction>,
        mut rx: mpsc::Receiver<AppAction>,
        background_tasks: &mut Vec<JoinHandle<()>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut stats_interval = interval(Duration::from_secs(INTERVAL));
        stats_interval.tick().await;

        loop {
            tokio::select! {
                Some(action) = rx.recv() => {
                    self.update(action);
                }
                _ = stats_interval.tick() => {
                    self.spawn_stats_tasks(&tx, background_tasks);
                }
            }

            background_tasks.retain(|task| !task.is_finished());

            if !self.running {
                break;
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn spawn_connection_tasks(
        &self,
        tx: &mpsc::Sender<AppAction>,
        current_user: &str,
    ) -> Vec<JoinHandle<()>> {
        let mut tasks = Vec::with_capacity(self.hosts.len());
        for host in &self.hosts {
            let host_name = host.name.clone();
            let config = host.config.clone().unwrap();
            let tx = tx.clone();
            let default_user = current_user.to_string();

            tasks.push(tokio::spawn(async move {
                let mut builder = SessionBuilder::default();

                let user = config.user.unwrap_or(default_user);
                builder.user(user);
                builder.port(config.port.unwrap_or(22));

                if let Some(identity_file) = config.identity_file
                    && !identity_file.is_empty()
                {
                    builder.keyfile(identity_file);
                }

                builder.known_hosts_check(KnownHosts::Strict);

                match builder.connect(&config.address).await {
                    Ok(session) => {
                        let session = Arc::new(session);
                        let _ = tx
                            .send(AppAction::Connected(
                                host_name.clone(),
                                Arc::clone(&session),
                            ))
                            .await;

                        if let Ok(stats) = HostStats::fetch(session).await {
                            let _ = tx.send(AppAction::StatsFetched(host_name, stats)).await;
                        }
                    }
                    Err(err) => {
                        let _ = tx
                            .send(AppAction::ConnectionFailed(host_name, err.to_string()))
                            .await;
                    }
                }
            }));
        }
        tasks
    }

    fn spawn_input_task(tx: mpsc::Sender<AppAction>) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                if event::poll(Duration::from_millis(100)).unwrap_or(false)
                    && let Ok(ev) = event::read()
                    && tx.send(AppAction::Input(ev)).await.is_err()
                {
                    break;
                }
            }
        })
    }

    fn spawn_stats_tasks(&self, tx: &mpsc::Sender<AppAction>, tasks: &mut Vec<JoinHandle<()>>) {
        for host in &self.hosts {
            if let Some(session) = &host.session {
                let session = Arc::clone(session);
                let host_name = host.name.clone();
                let tx = tx.clone();

                tasks.push(tokio::spawn(async move {
                    if let Ok(stats) = HostStats::fetch(session).await {
                        let _ = tx.send(AppAction::StatsFetched(host_name, stats)).await;
                    }
                }));
            }
        }
    }

    async fn shutdown(&mut self, background_tasks: &mut Vec<JoinHandle<()>>) {
        for task in background_tasks.iter() {
            task.abort();
        }

        let tasks = std::mem::take(background_tasks);
        let _ = timeout(Duration::from_millis(250), async move {
            for task in tasks {
                let _ = task.await;
            }
        })
        .await;

        let mut sessions = Vec::new();
        for host in &mut self.hosts {
            if let Some(session) = host.session.take() {
                sessions.push((host.name.clone(), session));
            }
        }

        for (name, session) in sessions {
            match Arc::try_unwrap(session) {
                Ok(session) => match timeout(Duration::from_millis(500), session.close()).await {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => {
                        eprintln!("failed to close SSH session for {name}: {err}");
                    }
                    Err(_) => {
                        eprintln!("timed out closing SSH session for {name}");
                    }
                },
                Err(_session) => {}
            }
        }
    }
}
