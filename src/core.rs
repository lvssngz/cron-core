use crate::store::Store;
use crate::task::Task;
use crate::scheduler::{Scheduler, ScheduleCmd};
use crate::runner::Runner;
use std::sync::Arc;
use tokio::sync::broadcast::{channel as broadcast_channel, Receiver as BroadcastReceiver, Sender as BroadcastSender};
use tracing::info;
use uuid::Uuid;

pub struct Core {
    store: Arc<Store>,
    /// Core -> Scheduler: 调度命令
    cmd_tx: BroadcastSender<ScheduleCmd>,
    shutdown_tx: BroadcastSender<()>,
    shutdown_rx: BroadcastReceiver<()>,
    running: bool,
    scheduler: Scheduler,
    runner: Runner,
}

impl Core {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> sled::Result<Self> {
        let store = Arc::new(Store::new(path)?);

        // Core -> Scheduler 通道
        let (cmd_tx, cmd_rx) = broadcast_channel(200);
        // Scheduler -> Runner 通道
        let (exec_tx, exec_rx) = broadcast_channel(200);
        // Shutdown 通道
        let (shutdown_tx, shutdown_rx) = broadcast_channel(1);

        let scheduler = Scheduler::new(cmd_rx, exec_tx);
        let runner = Runner::new(store.clone(), exec_rx, cmd_tx.clone());

        Ok(Self {
            store,
            cmd_tx,
            shutdown_tx,
            shutdown_rx,
            running: false,
            scheduler,
            runner,
        })
    }

    /// 加载所有启用的任务到 scheduler
    fn load_tasks(&self) {
        if let Ok(tasks) = self.store.list() {
            for task in &tasks {
                if task.enabled {
                    if let Some(next_run) = task.next_tick() {
                        let _ = self.cmd_tx.send((task.id, next_run));
                    }
                }
            }
            info!(count = tasks.len(), "Tasks loaded");
        }
    }

    pub async fn create_task(&self, name: &str, cron: &str, command: &str) -> anyhow::Result<()> {
        let task = Task::new(name, cron, command)?;
        self.store.save(&task)?;

        info!(
            task_id = %task.id,
            name = %task.name,
            cron = %task.cron,
            "Task created"
        );

        if let Some(next_run) = task.next_tick() {
            let _ = self.cmd_tx.send((task.id, next_run));
        }

        Ok(())
    }

    pub async fn delete_task(&self, id: Uuid) -> anyhow::Result<bool> {
        let deleted = self.store.delete(&id)?;
        if deleted {
            info!(task_id = %id, "Task deleted");
            let _ = self.cmd_tx.send((id, chrono::DateTime::UNIX_EPOCH));
        }
        Ok(deleted)
    }

    pub async fn enable_task(&self, id: Uuid) -> anyhow::Result<bool> {
        if let Some(mut task) = self.store.get(&id)? {
            if !task.enabled {
                task.enabled = true;
                self.store.save(&task)?;
                info!(task_id = %id, "Task enabled");
                if let Some(next_run) = task.next_tick() {
                    let _ = self.cmd_tx.send((task.id, next_run));
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn disable_task(&self, id: Uuid) -> anyhow::Result<bool> {
        if let Some(mut task) = self.store.get(&id)? {
            if task.enabled {
                task.enabled = false;
                self.store.save(&task)?;
                info!(task_id = %id, "Task disabled");
                let _ = self.cmd_tx.send((id, chrono::DateTime::UNIX_EPOCH));
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn list_tasks(&self) -> anyhow::Result<Vec<Task>> {
        let tasks = self.store.list()?;
        info!(count = tasks.len(), "Tasks listed");
        for task in &tasks {
            info!(
                task_id = %task.id,
                name = %task.name,
                cron = %task.cron,
                enabled = task.enabled,
                command = %task.command
            );
        }
        Ok(tasks)
    }

    /// 启动 scheduler 和 runner
    pub fn start(&mut self) {
        if self.running {
            return;
        }
        self.running = true;

        let shutdown_rx = self.shutdown_rx.resubscribe();
        self.scheduler.start(shutdown_rx);

        let shutdown_rx = self.shutdown_rx.resubscribe();
        self.runner.start(shutdown_rx);

        // 加载已有任务
        self.load_tasks();

        info!("Core started");
    }

    /// 关闭 core
    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        if !self.running {
            return Ok(());
        }
        self.running = false;

        info!("Shutting down...");
        let _ = self.shutdown_tx.send(());

        self.scheduler.shutdown().await?;
        self.runner.shutdown().await?;

        info!("Core shutdown complete");
        Ok(())
    }
}