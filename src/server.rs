use std::{
    env, fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

use anyhow::Result;
use ssh2::Session;
use tokio::time;

pub struct GpuServer {
    pub username: String,
    pub password: String,
    pub job_name: String,
    pub resource_dir: String,
    pub code_file_name: String,
    pub remote_dir: String,
    pub compile_script: String,
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
                    "/lustre/home/{}/{}/{}",
                    env::var("ACCOUNT").unwrap_or_else(|_| "acct-stu".to_string()),
                    env::var("USERNAME").unwrap(),
                    env::var("JOB_NAME").unwrap()
                )
            }),
            compile_script: env::var("COMPILE_SCRIPT").unwrap_or_else(|_| "make".to_string()),
        }
    }

    pub async fn run(&self, login: &Session, data: &Session) -> Result<String> {
        self.exec(login, "ls").await?;
        // upload
        self.upload_resources(data).await?;
        // unzip
        self.exec(
            login,
            format!(
                "unzip -o -d {remote} {remote}/{zip}",
                remote = self.remote_dir,
                zip = self.code_file_name
            )
            .as_str(),
        )
        .await?;
        // compile
        self.exec(
            login,
            format!("cd {} && {}", self.remote_dir, self.compile_script).as_str(),
        )
        .await?;
        // submit job
        let job_id = self
            .exec(
                login,
                format!("cd {} && sbatch {}.slurm", self.remote_dir, self.job_name).as_str(),
            )
            .await?;
        let job_id = scan_fmt!(job_id.as_str(), "Submitted batch job {}", String)?;
        // wait for the result
        let res = self.get_result(login, data, job_id.as_str()).await?;

        Ok(res)
    }

    pub async fn connect(&self, login_addr: &str, data_addr: &str) -> (Session, Session) {
        let login = self.connect_session(login_addr).await;
        let data = self.connect_session(data_addr).await;
        (login, data)
    }

    pub async fn exec(&self, sess: &Session, cmd: &str) -> Result<String> {
        println!("exec: {}", cmd);
        let mut channel = sess.channel_session().unwrap();
        channel.exec(cmd)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        println!("{}", s);
        channel.wait_close()?;
        Ok(s)
    }

    pub async fn upload_file(&self, sess: &Session, local: &Path, remote: &Path) -> Result<()> {
        println!(
            "uploading file: {} --> {}",
            local.to_str().unwrap(),
            remote.to_str().unwrap()
        );
        let result = fs::read(local)?;
        let mut remote_file = sess.scp_send(remote, 0o644, result.len() as u64, None)?;
        remote_file.write_all(&result)?;
        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(())
    }

    pub async fn download_file(&self, sess: &Session, local: &Path, remote: &Path) -> Result<()> {
        println!(
            "downloading file: {} --> {}",
            remote.to_str().unwrap(),
            local.to_str().unwrap()
        );

        let (mut remote_file, _) = sess.scp_recv(remote)?;
        let mut buf = Vec::new();
        remote_file.read_to_end(&mut buf).unwrap();

        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        let mut file = std::fs::File::create(local)?;
        file.write_all(buf.as_slice())?;
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

    async fn get_result(&self, login: &Session, data: &Session, job_id: &str) -> Result<String> {
        self.wait_for_completion(login, job_id).await?;

        // print
        let res = self
            .exec(
                login,
                format!("cat {}/{}.out", self.remote_dir, job_id).as_str(),
            )
            .await?;

        // download
        self.download_file(
            data,
            &PathBuf::from(format!("{}/{}.out", self.remote_dir, job_id)),
            &PathBuf::from(format!("{}/output.out", self.resource_dir)),
        )
        .await?;

        self.download_file(
            data,
            &PathBuf::from(format!("{}/{}.err", self.remote_dir, job_id)),
            &PathBuf::from(format!("{}/output.err", self.resource_dir)),
        )
        .await?;

        Ok(res)
    }

    async fn wait_for_completion(&self, sess: &Session, job_id: &str) -> Result<()> {
        let mut interval = time::interval(time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if self.check_completion(sess, job_id).await? {
                break;
            }
        }
        Ok(())
    }

    async fn check_completion(&self, sess: &Session, job_id: &str) -> Result<bool> {
        let result = self
            .exec(sess, format!("squeue -o %T -j {}", job_id).as_str())
            .await?;
        Ok(!result.contains("PENDING")
            && !result.contains("RUNNING")
            && !result.contains("CONFIGURING")
            && !result.contains("COMPLETING"))
    }
}
