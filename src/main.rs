use anyhow::Result;

use crate::server::GpuServer;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let server = GpuServer::from_env();
    let (login_sess, data_sess) = server
        .connect("login.hpc.sjtu.edu.cn:22", "data.hpc.sjtu.edu.cn:22")
        .await;

    server.exec(&login_sess, "ls".to_string()).await?;
    server.upload_resources(&data_sess).await?;
    server.exec(&login_sess, "ls".to_string()).await?;
    // server.exec(&login_sess, "ls cuda".to_string()).await?;

    Ok(())
}
