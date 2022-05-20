use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;

use crate::config::CONFIG;

mod config;

fn main() {
    let tcp = TcpStream::connect("login.hpc.sjtu.edu.cn:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password(CONFIG.username.as_str(), CONFIG.password.as_str())
        .unwrap();
    assert!(sess.authenticated());
    let mut channel = sess.channel_session().unwrap();
    channel.exec("ls").unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    println!("{}", s);
    channel
        .wait_close()
        .map_err(|err| println!("{:?}", err))
        .ok();

    println!("{}", channel.exit_status().unwrap());
    println!("Hello, world!");
}
