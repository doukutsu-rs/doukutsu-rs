use tokio::net::UdpSocket;
use crate::framework::error::GameResult;
use crate::netplay::server_config::ServerConfiguration;

pub struct Server {
}

impl Server {
    pub fn start(config: ServerConfiguration) -> GameResult<Server> {
        let context = ServerContext::new(config);


        Ok(Server {

        })
    }
}

struct ServerContext {
    config: ServerConfiguration,
}

impl ServerContext {
    pub fn new(config: ServerConfiguration) -> ServerContext {
        ServerContext {
            config
        }
    }

    pub async fn run(self) {
        let socket = UdpSocket::bind(&self.config.bind_to).await.unwrap();

    }
}
