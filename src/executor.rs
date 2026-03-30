use std::process::Output;

use tokio::process::Command;

pub struct Executor;

impl Executor {
    pub async fn run(command: &str) -> ExecutionResult {
        #[cfg(target_os = "windows")]
        let result = Self::run_windows(command).await;

        #[cfg(not(target_os = "windows"))]
        let result = Self::run_unix(command).await;

        result
    }

    #[cfg(target_os = "windows")]
    async fn run_windows(command: &str) -> ExecutionResult {
        match Command::new("cmd").args(&["/C", command]).output().await {
            Ok(output) => ExecutionResult::from_output(output),
            Err(e) => ExecutionResult::from_error(e),
        }
    }

    #[cfg(not(target_os = "windows"))]
    async fn run_unix(command: &str) -> ExecutionResult {
        match Command::new("sh").arg("-c").arg(command).output().await {
            Ok(output) => ExecutionResult::from_output(output),
            Err(e) => ExecutionResult::from_error(e),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl ExecutionResult {
    fn from_output(output: Output) -> Self {
        Self {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        }
    }

    fn from_error(e: std::io::Error) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr: e.to_string(),
            exit_code: None,
        }
    }
}