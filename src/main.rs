use anyhow::Result;

use crate::server::GpuServer;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let server = GpuServer::from_env();
    let sess = server.connect("login.hpc.sjtu.edu.cn:22").await;

    server.exec(sess, "ls".to_string()).await?;

    Ok(())
}
