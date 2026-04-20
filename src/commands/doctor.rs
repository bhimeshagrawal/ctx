use anyhow::Result;

use crate::{
    cli::DoctorArgs, output,
    services::{runtime::ServiceRuntime, system},
};

pub async fn run(args: DoctorArgs) -> Result<()> {
    let runtime = ServiceRuntime::bootstrap(None, None, false).await?;
    let result = system::doctor(&runtime).await?;
    output::render(&result, args.json)
}
