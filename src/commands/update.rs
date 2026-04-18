use anyhow::Result;

use crate::{cli::UpdateArgs, output, update};

pub async fn run(args: UpdateArgs) -> Result<()> {
    let result = update::describe_update(
        args.version,
        std::env::var("CTX_REPO").unwrap_or_else(|_| "bhimeshagrawal/ctx".to_string()),
    );
    output::render(&result, args.json)
}
