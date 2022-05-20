use std::{env, io::Read, net::TcpStream};

use anyhow::Result;
use ssh2::Session;

pub struct GpuServer {
    pub username: String,
    pub password: String,
    pub code_path: String,
}

impl GpuServer {
    pub fn from_env() -> GpuServer {
        GpuServer {
            username: env::var("USERNAME").unwrap(),
            password: env::var("PASSWORD").unwrap(),
            code_path: env::var("CODE_PATH").unwrap_or_else(|_| "./resources".to_string()),
        }
    }

    pub async fn connect(&self, address: &str) -> Session {
        let tcp = TcpStream::connect(address).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        sess.userauth_password(self.username.as_str(), self.password.as_str())
            .unwrap();
        assert!(sess.authenticated());
        sess
    }

    pub async fn exec(&self, sess: Session, cmd: String) -> Result<String> {
        let mut channel = sess.channel_session().unwrap();
        channel.exec(cmd.as_str())?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        println!("{}", s);
        channel.wait_close()?;
        Ok(s)
    }
}
