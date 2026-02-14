use anyhow::{anyhow, Context, Result};
use neuropad_ipc::{IpcEnvelope, IpcRequest};
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Clone)]
pub struct KernelLaunch {
    pub executable: PathBuf,
    pub args: Vec<String>,
}

struct KernelProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl KernelProcess {
    fn spawn(launch: &KernelLaunch) -> Result<Self> {
        let mut cmd = Command::new(&launch.executable);
        if !launch.args.is_empty() {
            cmd.args(&launch.args);
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("failed to spawn kernel process '{}'", launch.executable.display()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("kernel stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("kernel stdout unavailable"))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    fn call(&mut self, request: &IpcRequest) -> Result<IpcEnvelope> {
        let payload = serde_json::to_string(request)?;
        writeln!(self.stdin, "{payload}")?;
        self.stdin.flush()?;

        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        if line.trim().is_empty() {
            return Err(anyhow!("kernel returned empty response"));
        }
        let envelope = serde_json::from_str::<IpcEnvelope>(line.trim())?;
        Ok(envelope)
    }
}

pub struct NotebookRuntimes {
    go: KernelProcess,
    ruby: KernelProcess,
    python: KernelProcess,
}

impl NotebookRuntimes {
    fn for_language_mut(&mut self, language: &str) -> Result<&mut KernelProcess> {
        match language {
            "go" => Ok(&mut self.go),
            "ruby" => Ok(&mut self.ruby),
            "python" => Ok(&mut self.python),
            _ => Err(anyhow!("unsupported language '{language}'")),
        }
    }
}

pub struct KernelManager {
    notebooks: HashMap<String, NotebookRuntimes>,
    go_kernel: KernelLaunch,
    ruby_kernel: KernelLaunch,
    python_kernel: KernelLaunch,
}

impl KernelManager {
    pub fn new(go_kernel: KernelLaunch, ruby_kernel: KernelLaunch, python_kernel: KernelLaunch) -> Self {
        Self {
            notebooks: HashMap::new(),
            go_kernel,
            ruby_kernel,
            python_kernel,
        }
    }

    fn ensure_notebook(&mut self, notebook_id: &str) -> Result<&mut NotebookRuntimes> {
        if !self.notebooks.contains_key(notebook_id) {
            let go = KernelProcess::spawn(&self.go_kernel)?;
            let ruby = KernelProcess::spawn(&self.ruby_kernel)?;
            let python = KernelProcess::spawn(&self.python_kernel)?;
            self.notebooks
                .insert(notebook_id.to_string(), NotebookRuntimes { go, ruby, python });
        }
        self.notebooks
            .get_mut(notebook_id)
            .ok_or_else(|| anyhow!("notebook runtime not found"))
    }

    pub fn execute(&mut self, notebook_id: &str, language: &str, code: &str) -> Result<IpcEnvelope> {
        let runtime = self.ensure_notebook(notebook_id)?;
        let kernel = runtime.for_language_mut(language)?;
        let req = IpcRequest {
            id: uuid::Uuid::new_v4().to_string(),
            method: "execute".to_string(),
            params: json!({ "code": code }),
        };
        kernel.call(&req)
    }

    pub fn interrupt(&mut self, notebook_id: &str, language: &str) -> Result<IpcEnvelope> {
        let runtime = self.ensure_notebook(notebook_id)?;
        let kernel = runtime.for_language_mut(language)?;
        let req = IpcRequest {
            id: uuid::Uuid::new_v4().to_string(),
            method: "interrupt".to_string(),
            params: json!({}),
        };
        kernel.call(&req)
    }

    pub fn restart(&mut self, notebook_id: &str, language: &str) -> Result<IpcEnvelope> {
        let runtime = self.ensure_notebook(notebook_id)?;
        let kernel = runtime.for_language_mut(language)?;
        let req = IpcRequest {
            id: uuid::Uuid::new_v4().to_string(),
            method: "restart".to_string(),
            params: json!({}),
        };
        kernel.call(&req)
    }

    pub fn shutdown_notebook(&mut self, notebook_id: &str) {
        if let Some(mut rt) = self.notebooks.remove(notebook_id) {
            let _ = rt.go.child.kill();
            let _ = rt.ruby.child.kill();
            let _ = rt.python.child.kill();
        }
    }
}
