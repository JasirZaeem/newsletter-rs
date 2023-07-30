use anyhow::Result;
use newsletter::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}