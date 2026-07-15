use futures::StreamExt;

use bollard::container::LogOutput;
use tracing::warn;

use super::{
    errors::{ContainerError, ContainerResult},
    manager::ContainerManager,
    types::ExecOutput,
};

pub fn sanitize_for_shell_arg(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\0' | '\n' | '\r' => {}
            _ => out.push(ch),
        }
    }
    format!("'{}'", out.replace('\'', "'\\''"))
}

impl ContainerManager {
    /// Execute a command inside a container.
    ///
    /// The `command` slices are passed as the argument vector directly
    /// to the container exec API without shell interpolation.
    pub async fn exec(&self, container_id: &str, command: &[&str]) -> ContainerResult<ExecOutput> {
        let cmd: Vec<String> = command.iter().map(|s| s.to_string()).collect();

        let create_options = bollard::exec::CreateExecOptions::<String> {
            cmd: Some(cmd),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec_response = self
            .docker
            .create_exec(container_id, create_options)
            .await
            .map_err(|e| ContainerError::ExecFailed {
                container_id: container_id.to_string(),
                message: format!("create_exec failed: {}", e),
            })?;

        let start_result = self
            .docker
            .start_exec(
                &exec_response.id,
                Some(bollard::exec::StartExecOptions {
                    detach: false,
                    ..Default::default()
                }),
            )
            .await
            .map_err(|e| ContainerError::ExecFailed {
                container_id: container_id.to_string(),
                message: format!("start_exec failed: {}", e),
            })?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        if let bollard::exec::StartExecResults::Attached { output, .. } = start_result {
            let mut output = output;
            while let Some(msg) = output.next().await {
                match msg {
                    Ok(LogOutput::StdOut { message }) => {
                        stdout.push_str(&String::from_utf8_lossy(&message));
                    }
                    Ok(LogOutput::StdErr { message }) => {
                        stderr.push_str(&String::from_utf8_lossy(&message));
                    }
                    Ok(_) => {}
                    Err(e) => {
                        stderr.push_str(&format!("exec stream error: {}", e));
                        break;
                    }
                }
            }
        }

        let inspect = match self.docker.inspect_exec(&exec_response.id).await {
            Ok(i) => Some(i),
            Err(e) => {
                warn!(
                    exec_id = %exec_response.id,
                    error = %e,
                    "exec inspect failed — exit code will be unavailable"
                );
                None
            }
        };

        Ok(ExecOutput {
            exit_code: inspect.and_then(|i| i.exit_code),
            stdout: stdout.trim_end().to_string(),
            stderr: stderr.trim_end().to_string(),
        })
    }
}
