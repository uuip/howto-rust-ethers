#[derive(Debug)]
pub struct Setting {
    pub rpc: String,
    pub token: String,
}

pub fn get_str_env(key: &str) -> String {
    dotenvy::var(key).unwrap_or_else(|_| panic!("lost {key}"))
}

impl Setting {
    pub fn init() -> Self {
        let rpc = get_str_env("RPC");
        let token = get_str_env("TOKEN_JPY");
        Self { rpc, token }
    }
}
