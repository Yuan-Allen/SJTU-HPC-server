use std::{
    env, fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

use anyhow::Result;
use ssh2::Session;

pub struct GpuServer {
    pub username: String,
    pub password: String,
    pub job_name: String,
    pub resource_dir: String,
    pub code_file_name: String,
    pub remote_dir: String,
}

impl GpuServer {
    pub fn from_env() -> GpuServer {
        GpuServer {
            username: env::var("USERNAME").unwrap(),
            password: env::var("PASSWORD").unwrap(),
            job_name: env::var("JOB_NAME").unwrap(),
            resource_dir: env::var("RESOURCE_DIR").unwrap_or_else(|_| "/code".to_string()),
            code_file_name: env::var("CODE_FILE_NAME").unwrap_or_else(|_| "gpu.zip".to_string()),
            remote_dir: env::var("REMOTE_DIR").unwrap_or_else(|_| {
                format!(
                    "/lustre/home/acct-stu/{}/{}",
                    env::var("USERNAME").unwrap(),
                    env::var("JOB_NAME").unwrap()
                )
            }),
        }
    }

    pub async fn connect(&self, login_addr: &str, data_addr: &str) -> (Session, Session) {
        let login = self.connect_session(login_addr).await;
        let data = self.connect_session(data_addr).await;
        (login, data)
    }

    pub async fn exec(&self, sess: &Session, cmd: String) -> Result<String> {
        let mut channel = sess.channel_session().unwrap();
        channel.exec(cmd.as_str())?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        println!("{}", s);
        channel.wait_close()?;
        Ok(s)
    }

    pub async fn upload_file(&self, sess: &Session, local: &Path, remote: &Path) -> Result<()> {
        let result = fs::read(local)?;
        let mut remote_file = sess.scp_send(remote, 0o644, result.len() as u64, None)?;
        remote_file.write_all(&result).unwrap();
        // Close the channel and wait for the whole content to be tranferred
        remote_file.send_eof().unwrap();
        remote_file.wait_eof().unwrap();
        remote_file.close().unwrap();
        remote_file.wait_close().unwrap();
        Ok(())
    }

    pub async fn upload_resources(&self, sess: &Session) -> Result<()> {
        let remote = PathBuf::from(self.remote_dir.clone());
        let fstp = sess.sftp()?;
        if fstp.lstat(&remote).is_err() {
            fstp.mkdir(&remote, 0o777)?;
        }

        self.upload_file(
            sess,
            &PathBuf::from(format!("{}/{}", self.resource_dir, self.code_file_name)),
            &PathBuf::from(format!("{}/{}", self.remote_dir, self.code_file_name)),
        )
        .await?;

        self.upload_file(
            sess,
            &PathBuf::from(format!("{}/{}.slurm", self.resource_dir, self.job_name)),
            &PathBuf::from(format!("{}/{}.slurm", self.remote_dir, self.job_name)),
        )
        .await?;

        Ok(())
    }

    async fn connect_session(&self, address: &str) -> Session {
        let tcp = TcpStream::connect(address).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        sess.userauth_password(self.username.as_str(), self.password.as_str())
            .unwrap();
        assert!(sess.authenticated());
        sess
    }
}
