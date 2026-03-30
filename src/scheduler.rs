use crate::heap::TaskHeap;
use std::time::Duration;
use tokio::sync::broadcast::{Receiver as BroadcastReceiver, Sender as BroadcastSender};
use tokio::time::sleep;
use tracing::{error, info};
use uuid::Uuid;

/// 任务调度消息：Core -> Scheduler
pub type ScheduleCmd = (Uuid, chrono::DateTime<chrono::Utc>);
/// 任务执行请求：Scheduler -> Runner
pub type ExecuteReq = Uuid;

pub struct Scheduler {
    handle: Option<tokio::task::JoinHandle<()>>,
    /// 接收 Core 发来的调度命令
    cmd_rx: BroadcastReceiver<ScheduleCmd>,
    /// 向 Runner 发送执行请求
    exec_tx: BroadcastSender<ExecuteReq>,
}

impl Scheduler {
    pub fn new(cmd_rx: BroadcastReceiver<ScheduleCmd>, exec_tx: BroadcastSender<ExecuteReq>) -> Self {
        Self {
            handle: None,
            cmd_rx,
            exec_tx,
        }
    }

    pub fn start(&mut self, shutdown_rx: BroadcastReceiver<()>) {
        if self.handle.is_some() {
            return;
        }

        let cmd_rx = self.cmd_rx.resubscribe();
        let exec_tx = self.exec_tx.clone();

        let handle = tokio::spawn(async move {
            let mut heap = TaskHeap::new();
            let mut cmd_rx = cmd_rx;
            let mut shutdown_rx = shutdown_rx;

            loop {
                let now = chrono::Utc::now();

                for task_id in heap.pop_due(now) {
                    if exec_tx.send(task_id).is_err() {
                        error!("Runner closed");
                        return;
                    }
                }

                let deadline = heap.next_deadline();
                let sleep_duration = match deadline {
                    Some(d) if d > now => (d - now).to_std().unwrap(),
                    Some(_) => Duration::ZERO,
                    None => Duration::from_secs(3600),
                };

                tokio::select! {
                    _ = sleep(sleep_duration) => {
                        continue;
                    }
                    res = cmd_rx.recv() => {
                        match res {
                            Ok((task_id, next_run)) => {
                                if next_run == chrono::DateTime::UNIX_EPOCH {
                                    heap.remove(&task_id);
                                } else {
                                    heap.push(task_id, next_run);
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Scheduler shutting down");
                        break;
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.await?;
        }
        Ok(())
    }
}
