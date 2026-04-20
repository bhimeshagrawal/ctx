use anyhow::Result;

use crate::{
    cli::UpdateArgs,
    output,
    services::{system, types::UpdateRequest},
};

pub async fn run(args: UpdateArgs) -> Result<()> {
    let result = system::update(
        UpdateRequest {
            version: args.version,
            force: args.force,
        },
        std::env::var("CTX_REPO").unwrap_or_else(|_| "bhimeshagrawal/ctx".to_string()),
    );
    output::render(&result, args.json)
}
