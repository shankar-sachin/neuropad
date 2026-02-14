use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub version: String,
    pub metadata: NotebookMetadata,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookMetadata {
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub kernel_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    Markdown,
    Code,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CellOutputKind {
    Stdout,
    Stderr,
    Result,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CellStatus {
    Idle,
    Running,
    Ok,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellExecution {
    pub count: u32,
    pub status: CellStatus,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellOutput {
    pub kind: CellOutputKind,
    pub mime: String,
    pub data: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub cell_type: CellType,
    #[serde(default)]
    pub language: Option<String>,
    pub source: String,
    #[serde(default)]
    pub outputs: Vec<CellOutput>,
    pub execution: CellExecution,
}

impl Notebook {
    pub fn new(title: &str) -> Self {
        let now = Utc::now();
        Self {
            version: "1.0".to_string(),
            metadata: NotebookMetadata {
                title: title.to_string(),
                created_at: now,
                updated_at: now,
                kernel_policy: "per_notebook".to_string(),
            },
            cells: vec![],
        }
    }

    pub fn add_markdown_cell(&mut self, source: impl Into<String>) -> Uuid {
        let cell = Cell::new_markdown(source);
        let id = cell.id;
        self.cells.push(cell);
        self.touch();
        id
    }

    pub fn add_code_cell(&mut self, language: impl Into<String>, source: impl Into<String>) -> Uuid {
        let cell = Cell::new_code(language, source);
        let id = cell.id;
        self.cells.push(cell);
        self.touch();
        id
    }

    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    pub fn validate(&self) -> CoreResult<()> {
        if self.version.trim().is_empty() {
            return Err(CoreError::Validation("version cannot be empty".to_string()));
        }
        for cell in &self.cells {
            match cell.cell_type {
                CellType::Markdown => {
                    if cell.language.is_some() {
                        return Err(CoreError::Validation(format!(
                            "markdown cell {} must not have a language",
                            cell.id
                        )));
                    }
                }
                CellType::Code => {
                    let lang = cell.language.as_deref().unwrap_or_default();
                    if lang != "go" && lang != "ruby" && lang != "python" {
                        return Err(CoreError::Validation(format!(
                            "code cell {} has unsupported language '{}'",
                            cell.id, lang
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_npad<P: AsRef<Path>>(&mut self, path: P) -> CoreResult<()> {
        self.touch();
        self.validate()?;
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn load_npad<P: AsRef<Path>>(path: P) -> CoreResult<Self> {
        let data = fs::read_to_string(path)?;
        let notebook: Self = serde_json::from_str(&data)?;
        notebook.validate()?;
        Ok(notebook)
    }
}

impl Cell {
    pub fn new_markdown(source: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            cell_type: CellType::Markdown,
            language: None,
            source: source.into(),
            outputs: vec![],
            execution: CellExecution {
                count: 0,
                status: CellStatus::Idle,
                duration_ms: 0,
            },
        }
    }

    pub fn new_code(language: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            cell_type: CellType::Code,
            language: Some(language.into()),
            source: source.into(),
            outputs: vec![],
            execution: CellExecution {
                count: 0,
                status: CellStatus::Idle,
                duration_ms: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_supported_languages() {
        let mut nb = Notebook::new("test");
        nb.add_code_cell("go", "fmt.Println(\"hello\")");
        nb.add_code_cell("ruby", "puts 'hello'");
        nb.add_code_cell("python", "print('hello')");
        assert!(nb.validate().is_ok());
    }

    #[test]
    fn rejects_unsupported_language() {
        let mut nb = Notebook::new("test");
        nb.add_code_cell("javascript", "console.log('no')");
        assert!(nb.validate().is_err());
    }
}
