use anyhow::Result;

use crate::{
    cli::MemorySearchArgs,
    output,
    services::{
        memory::{self, MemorySearchRequest},
        runtime::ServiceRuntime,
    },
};

pub async fn run(args: MemorySearchArgs) -> Result<()> {
    let runtime = ServiceRuntime::bootstrap(None, None, false).await?;
    let result = memory::search(
        &runtime,
        MemorySearchRequest {
            query: args.query.join(" "),
            top_k: args.top_k,
            tags: args.tags,
        },
    )
    .await?;
    output::render(&result, args.json)
}
