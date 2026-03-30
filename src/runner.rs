use crate::executor::Executor;
use crate::scheduler::{ScheduleCmd, ExecuteReq};
use crate::store::Store;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver as BroadcastReceiver, Sender as BroadcastSender};
use tracing::info;

pub struct Runner {
    handle: Option<tokio::task::JoinHandle<()>>,
    store: Arc<Store>,
    /// 接收 Scheduler 发来的执行请求
    exec_rx: BroadcastReceiver<ExecuteReq>,
    /// 向 Scheduler 发送调度命令（任务执行完后重新调度）
    cmd_tx: BroadcastSender<ScheduleCmd>,
}

impl Runner {
    pub fn new(
        store: Arc<Store>,
        exec_rx: BroadcastReceiver<ExecuteReq>,
        cmd_tx: BroadcastSender<ScheduleCmd>,
    ) -> Self {
        Self {
            handle: None,
            store,
            exec_rx,
            cmd_tx,
        }
    }

    pub fn start(&mut self, shutdown_rx: BroadcastReceiver<()>) {
        if self.handle.is_some() {
            return;
        }

        let store = self.store.clone();
        let exec_rx = self.exec_rx.resubscribe();
        let cmd_tx = self.cmd_tx.clone();

        let handle = tokio::spawn(async move {
            let mut exec_rx = exec_rx;
            let mut shutdown_rx = shutdown_rx;

            loop {
                tokio::select! {
                    res = exec_rx.recv() => {
                        let task_id = match res {
                            Ok(id) => id,
                            Err(_) => continue,
                        };

                        let task = match store.get(&task_id) {
                            Ok(Some(t)) => t,
                            _ => continue,
                        };

                        if !task.enabled {
                            continue;
                        }

                        info!(task_id = %task_id, command = %task.command, "Executing");
                        Executor::run(&task.command).await;

                        // 任务执行完，重新添加
                        if let Some(next_run) = task.next_tick() {
                            let _ = cmd_tx.send((task_id, next_run));
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Runner shutting down");
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
