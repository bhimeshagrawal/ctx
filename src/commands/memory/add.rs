use anyhow::{anyhow, Result};

use crate::{
    cli::MemoryAddArgs,
    input, output,
    services::{
        memory::{self, MemoryAddRequest, MemorySource},
        runtime::ServiceRuntime,
    },
};

pub async fn run(args: MemoryAddArgs) -> Result<()> {
    let runtime = ServiceRuntime::bootstrap(None, None, !args.json).await?;
    let source = match (args.file, args.text, args.stdin) {
        (Some(path), None, false) => MemorySource::File { path },
        (None, Some(text), false) => MemorySource::Text { text },
        (None, None, true) => {
            let input = input::read_input(None, None, true).await?;
            MemorySource::Stdin {
                text: input.content,
            }
        }
        _ => return Err(anyhow!("exactly one input source is required: --file, --text, or --stdin")),
    };
    let result = memory::add(
        &runtime,
        MemoryAddRequest {
            source,
            title: args.title,
            tags: args.tags,
            chunk_size: args.chunk_size,
            chunk_overlap: args.chunk_overlap,
        },
    )
    .await?;

    output::render(
        &serde_json::json!({
            "ok": result.ok,
            "documentId": result.document_id,
            "chunkCount": result.chunk_count,
            "title": result.title,
        }),
        args.json,
    )
}
