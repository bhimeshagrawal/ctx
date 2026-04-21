use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum SourceType {
    File,
    Text,
    Stdin,
}

#[derive(Debug, Clone, Serialize)]
pub struct InputPayload {
    pub source_type: SourceType,
    pub source_path: Option<PathBuf>,
    pub title: Option<String>,
    pub content: String,
}

pub async fn read_input(
    file: Option<String>,
    text: Option<String>,
    stdin: bool,
) -> Result<InputPayload> {
    let enabled = [file.is_some(), text.is_some(), stdin]
        .into_iter()
        .filter(|flag| *flag)
        .count();

    if enabled != 1 {
        return Err(anyhow!(
            "exactly one input source is required: --file, --text, or --stdin"
        ));
    }

    if let Some(file) = file {
        let path = PathBuf::from(&file);
        return Ok(InputPayload {
            title: path
                .file_name()
                .map(|value| value.to_string_lossy().into_owned()),
            source_type: SourceType::File,
            source_path: Some(std::fs::canonicalize(path)?),
            content: tokio::fs::read_to_string(file).await?,
        });
    }

    if let Some(text) = text {
        return Ok(InputPayload {
            source_type: SourceType::Text,
            source_path: None,
            title: None,
            content: text,
        });
    }

    let mut input = String::new();
    use tokio::io::AsyncReadExt;
    tokio::io::stdin().read_to_string(&mut input).await?;
    Ok(InputPayload {
        source_type: SourceType::Stdin,
        source_path: None,
        title: None,
        content: input,
    })
}
