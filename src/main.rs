use cron_core::Core;
use std::fs;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建日志目录
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cron-core");
    fs::create_dir_all(&data_dir)?;

    // 日志输出到文件
    let log_file = data_dir.join("cron.log");
    let file = fs::File::create(&log_file)?;
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .compact()
        .with_writer(file)
        .init();

    info!("Logging to: {:?}", log_file);

    // 创建 Core
    let mut core = Core::new(&data_dir)?;

    // 启动：scheduler 和 runner 启动，同时自动加载已有任务
    core.start();

    // 创建测试任务
    core.create_task("test", "*/2 * * * * *", "echo hello").await?;

    // 运行直到中断
    tokio::signal::ctrl_c().await?;

    // 关闭
    core.shutdown().await?;

    Ok(())
}
