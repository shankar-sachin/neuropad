use crate::notebook::{Cell, CellOutput, CellOutputKind, CellStatus, CellType, Notebook};
use crate::{CoreError, CoreResult};
use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub fn import_ipynb<P: AsRef<Path>>(path: P) -> CoreResult<Notebook> {
    let raw = fs::read_to_string(path)?;
    let root: Value = serde_json::from_str(&raw)?;
    let cells = root
        .get("cells")
        .and_then(Value::as_array)
        .ok_or_else(|| CoreError::Validation("ipynb missing cells array".to_string()))?;

    let mut notebook = Notebook::new(
        root.get("metadata")
            .and_then(|m| m.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("Imported Notebook"),
    );

    for raw_cell in cells {
        let cell_type = raw_cell
            .get("cell_type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let source = source_from_ipynb(raw_cell.get("source"));
        if cell_type == "markdown" {
            notebook.cells.push(Cell::new_markdown(source));
            continue;
        }

        if cell_type == "code" {
            let language = detect_language(raw_cell).unwrap_or_else(|| "go".to_string());
            let mut cell = Cell::new_code(language, source);
            let outputs = raw_cell
                .get("outputs")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            cell.outputs = outputs
                .iter()
                .filter_map(map_output_from_ipynb)
                .collect::<Vec<_>>();
            cell.execution.count = raw_cell
                .get("execution_count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32;
            if cell.execution.count > 0 {
                cell.execution.status = CellStatus::Ok;
            }
            notebook.cells.push(cell);
        }
    }

    notebook.validate()?;
    Ok(notebook)
}

pub fn export_ipynb<P: AsRef<Path>>(notebook: &Notebook, path: P) -> CoreResult<()> {
    notebook.validate()?;
    let cells = notebook
        .cells
        .iter()
        .map(|cell| match cell.cell_type {
            CellType::Markdown => json!({
                "cell_type": "markdown",
                "metadata": {},
                "source": split_lines(&cell.source),
            }),
            CellType::Code => {
                let outputs = cell.outputs.iter().map(map_output_to_ipynb).collect::<Vec<_>>();
                json!({
                    "cell_type": "code",
                    "execution_count": cell.execution.count,
                    "metadata": {
                        "language": cell.language.clone().unwrap_or_else(|| "go".to_string())
                    },
                    "source": split_lines(&cell.source),
                    "outputs": outputs
                })
            }
        })
        .collect::<Vec<_>>();

    let root = json!({
        "cells": cells,
        "metadata": {
            "title": notebook.metadata.title,
            "language_info": {
                "name": "neuropad-polyglot"
            }
        },
        "nbformat": 4,
        "nbformat_minor": 5
    });
    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn detect_language(cell: &Value) -> Option<String> {
    let metadata = cell.get("metadata")?;
    let name = metadata.get("language")?.as_str()?;
    match name {
        "go" | "ruby" => Some(name.to_string()),
        _ => None,
    }
}

fn source_from_ipynb(source: Option<&Value>) -> String {
    match source {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(lines)) => lines
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}

fn split_lines(input: &str) -> Vec<String> {
    input
        .lines()
        .map(|line| format!("{line}\n"))
        .collect::<Vec<_>>()
}

fn map_output_from_ipynb(raw: &Value) -> Option<CellOutput> {
    let now = Utc::now();
    let output_type = raw.get("output_type").and_then(Value::as_str).unwrap_or("");
    match output_type {
        "stream" => {
            let name = raw.get("name").and_then(Value::as_str).unwrap_or("stdout");
            let text = source_from_ipynb(raw.get("text"));
            Some(CellOutput {
                kind: if name == "stderr" {
                    CellOutputKind::Stderr
                } else {
                    CellOutputKind::Stdout
                },
                mime: "text/plain".to_string(),
                data: text,
                created_at: now,
            })
        }
        "error" => {
            let text = raw
                .get("evalue")
                .and_then(Value::as_str)
                .unwrap_or("execution error")
                .to_string();
            Some(CellOutput {
                kind: CellOutputKind::Error,
                mime: "text/plain".to_string(),
                data: text,
                created_at: now,
            })
        }
        "execute_result" | "display_data" => {
            let text = raw
                .get("data")
                .and_then(|d| d.get("text/plain"))
                .map(|v| source_from_ipynb(Some(v)))
                .unwrap_or_default();
            Some(CellOutput {
                kind: CellOutputKind::Result,
                mime: "text/plain".to_string(),
                data: text,
                created_at: now,
            })
        }
        _ => None,
    }
}

fn map_output_to_ipynb(output: &CellOutput) -> Value {
    match output.kind {
        CellOutputKind::Stdout => json!({
            "output_type": "stream",
            "name": "stdout",
            "text": split_lines(&output.data),
        }),
        CellOutputKind::Stderr => json!({
            "output_type": "stream",
            "name": "stderr",
            "text": split_lines(&output.data),
        }),
        CellOutputKind::Result => json!({
            "output_type": "execute_result",
            "data": {
                "text/plain": split_lines(&output.data)
            },
            "metadata": {},
            "execution_count": 1
        }),
        CellOutputKind::Error => json!({
            "output_type": "error",
            "ename": "NeuroPadError",
            "evalue": output.data,
            "traceback": []
        }),
    }
}

pub fn new_code_cell_for_import(language: &str, source: &str) -> Cell {
    Cell {
        id: Uuid::new_v4(),
        cell_type: CellType::Code,
        language: Some(language.to_string()),
        source: source.to_string(),
        outputs: vec![],
        execution: crate::notebook::CellExecution {
            count: 0,
            status: CellStatus::Idle,
            duration_ms: 0,
        },
    }
}
