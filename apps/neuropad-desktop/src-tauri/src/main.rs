#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod kernel_manager;

use chrono::Utc;
use kernel_manager::{KernelLaunch, KernelManager};
use neuropad_core::ipynb;
use neuropad_core::{Cell, CellOutput, CellOutputKind, CellStatus, MetadataStore, Notebook};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};
use uuid::Uuid;

struct AppState {
    kernels: Mutex<KernelManager>,
    metadata: Mutex<MetadataStore>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveResult {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ack {
    ok: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExecutionTicket {
    notebook_id: String,
    cell_id: String,
    status: String,
}

#[tauri::command]
fn notebook_new(title: String) -> Result<Notebook, String> {
    Ok(Notebook::new(&title))
}

#[tauri::command]
fn notebook_open(path: String, state: State<AppState>) -> Result<Notebook, String> {
    let notebook = Notebook::load_npad(&path).map_err(|e| e.to_string())?;
    state
        .metadata
        .lock()
        .map_err(|_| "metadata lock poisoned".to_string())?
        .mark_recent_open(&path, &Utc::now().to_rfc3339())
        .map_err(|e| e.to_string())?;
    Ok(notebook)
}

#[tauri::command]
fn notebook_save(path: String, mut notebook: Notebook, state: State<AppState>) -> Result<SaveResult, String> {
    notebook.save_npad(&path).map_err(|e| e.to_string())?;
    state
        .metadata
        .lock()
        .map_err(|_| "metadata lock poisoned".to_string())?
        .upsert_notebook_index(
            &path,
            &notebook.metadata.title,
            &notebook.metadata.updated_at.to_rfc3339(),
        )
        .map_err(|e| e.to_string())?;
    Ok(SaveResult { path })
}

#[tauri::command]
fn cell_execute(
    notebook_id: String,
    cell_id: String,
    language: String,
    code: String,
    state: State<AppState>,
) -> Result<(ExecutionTicket, Vec<CellOutput>), String> {
    let started = std::time::Instant::now();
    let envelope = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_string())?
        .execute(&notebook_id, &language, &code)
        .map_err(|e| e.to_string())?;

    let mut outputs = vec![];
    if let Some(err) = envelope.error {
        outputs.push(CellOutput {
            kind: CellOutputKind::Error,
            mime: "text/plain".to_string(),
            data: format!("{}: {}", err.code, err.message),
            created_at: Utc::now(),
        });
    } else if let Some(result) = envelope.result {
        outputs.push(CellOutput {
            kind: CellOutputKind::Result,
            mime: "text/plain".to_string(),
            data: result.to_string(),
            created_at: Utc::now(),
        });
    }

    let ticket = ExecutionTicket {
        notebook_id,
        cell_id,
        status: "ok".to_string(),
    };

    let _duration = started.elapsed();
    Ok((ticket, outputs))
}

#[tauri::command]
fn kernel_interrupt(notebook_id: String, language: String, state: State<AppState>) -> Result<Ack, String> {
    state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_string())?
        .interrupt(&notebook_id, &language)
        .map_err(|e| e.to_string())?;
    Ok(Ack { ok: true })
}

#[tauri::command]
fn kernel_restart(notebook_id: String, language: String, state: State<AppState>) -> Result<Ack, String> {
    state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_string())?
        .restart(&notebook_id, &language)
        .map_err(|e| e.to_string())?;
    Ok(Ack { ok: true })
}

#[tauri::command]
fn import_ipynb(path: String) -> Result<Notebook, String> {
    ipynb::import_ipynb(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_ipynb(path: String, notebook: Notebook) -> Result<SaveResult, String> {
    ipynb::export_ipynb(&notebook, &path).map_err(|e| e.to_string())?;
    Ok(SaveResult { path })
}

#[tauri::command]
fn ai_generate_cell(prompt: String, language: String) -> Result<Cell, String> {
    let source = format!(
        "// Local AI provider integration placeholder.\n// Prompt: {}\n",
        prompt
    );
    let mut cell = Cell::new_code(language, source);
    cell.execution.status = CellStatus::Idle;
    cell.execution.count = 0;
    cell.outputs = vec![];
    Ok(cell)
}

fn resolve_resource(app: &tauri::AppHandle, relative: &str) -> PathBuf {
    match app.path().resource_dir() {
        Ok(resource_dir) => resource_dir.join(relative),
        Err(_) => PathBuf::from("..")
            .join("..")
            .join("..")
            .join(relative.replace('/', "\\")),
    }
}

fn resolve_first_existing_resource(app: &tauri::AppHandle, candidates: &[&str]) -> PathBuf {
    for candidate in candidates {
        let path = resolve_resource(app, candidate);
        if path.exists() {
            return path;
        }
    }
    resolve_resource(app, candidates[0])
}

fn pick_python_executable(app: &tauri::AppHandle) -> String {
    let embedded = resolve_first_existing_resource(
        app,
        &[
            "runtimes/python/python.exe",
            "services/python-portable/python.exe",
            "python-portable/python.exe",
        ],
    );
    if embedded.exists() {
        embedded.to_string_lossy().to_string()
    } else {
        "python".to_string()
    }
}

fn pick_ruby_launch(app: &tauri::AppHandle) -> KernelLaunch {
    let embedded_ruby = resolve_first_existing_resource(
        app,
        &[
            "runtimes/ruby/bin/ruby.exe",
            "services/ruby-portable/bin/ruby.exe",
            "ruby-portable/bin/ruby.exe",
        ],
    );
    let ruby_script = resolve_first_existing_resource(
        app,
        &["kernels/ruby_kernel.rb", "services/ruby-kernel/ruby_kernel.rb", "ruby_kernel.rb"],
    );
    if embedded_ruby.exists() {
        KernelLaunch {
            executable: embedded_ruby,
            args: vec![ruby_script.to_string_lossy().to_string()],
        }
    } else {
        KernelLaunch {
            executable: PathBuf::from("ruby"),
            args: vec![ruby_script.to_string_lossy().to_string()],
        }
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let go_kernel_path = resolve_first_existing_resource(
                app.handle(),
                &["kernels/go-kernel.exe", "services/go-kernel/go-kernel.exe", "go-kernel.exe"],
            );
            let python_kernel_script = resolve_first_existing_resource(
                app.handle(),
                &[
                    "kernels/python_kernel.py",
                    "services/python-kernel/python_kernel.py",
                    "python_kernel.py",
                ],
            );
            let python_executable = pick_python_executable(app.handle());
            let metadata = MetadataStore::open("neuropad.sqlite").expect("failed to initialize metadata store");

            let state = AppState {
                kernels: Mutex::new(KernelManager::new(
                    KernelLaunch {
                        executable: go_kernel_path,
                        args: vec![],
                    },
                    pick_ruby_launch(app.handle()),
                    KernelLaunch {
                        executable: PathBuf::from(python_executable),
                        args: vec![python_kernel_script.to_string_lossy().to_string()],
                    },
                )),
                metadata: Mutex::new(metadata),
            };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            notebook_new,
            notebook_open,
            notebook_save,
            cell_execute,
            kernel_interrupt,
            kernel_restart,
            import_ipynb,
            export_ipynb,
            ai_generate_cell
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
