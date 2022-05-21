use anyhow::Result;

use crate::server::GpuServer;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let server = GpuServer::from_env();

    let (login_sess, data_sess) = server
        .connect("login.hpc.sjtu.edu.cn:22", "data.hpc.sjtu.edu.cn:22")
        .await;

    server.run(&login_sess, &data_sess).await?;

    Ok(())
}
