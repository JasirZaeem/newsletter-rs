use anyhow::Result;
use newsletter::run;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = std::net::TcpListener::bind("127.0.0.1:8000")?;
    run(listener)?.await?;
    Ok(())
}