use lazy_static::lazy_static;
use std::env;

pub struct GpuServerConfig {
    pub username: String,
    pub password: String,
}

impl GpuServerConfig {
    fn from_env() -> GpuServerConfig {
        GpuServerConfig {
            username: env::var("USERNAME").unwrap(),
            password: env::var("PASSWORD").unwrap(),
        }
    }
}

lazy_static! {
    pub static ref CONFIG: GpuServerConfig = GpuServerConfig::from_env();
}
