use russh::client;
use russh::keys::key;


// struct ClientHandler;
//
// #[async_trait::async_trait]
// impl client::Handler for ClientHandler {
//     type Error = anyhow::Error;
//     async fn check_server_key(
//         self,
//         _server_public_key: &key::PublicKey,
//     ) -> Result<(Self, bool), Self::Error> {
//         Ok((self, true))
//     }
// }
pub struct HostInfo {
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key: Option<String>,
}

impl HostInfo {
    pub fn new(host: &str, port: u16, username: &str, password: Option<&str>, private_key: Option<&str>) -> Self {
        Self {
            host: host.to_string(),
            port,
            username: username.to_string(),
            password: password.map(|s| s.to_string()),
            private_key: private_key.map(|s| s.to_string()),
        }
    }
    // async fn connect(&self) -> Result<SshConn, Box<dyn std::error::Error>> {
    //
    // }
    // pub async fn connect_with_jumper(&self, target_host: &HostInfo)
    //     ->
    //     anyhow::Result<client::Handle<ClientHandler>> {
    // }
}


pub struct SshConn {
}
