use serde::{Deserialize, Serialize};

fn def_auth_server() -> String {
    "https://mal-cli.dogfetus.no".to_string()
}

fn def_callback_port() -> u16 {
    53400
}

fn def_max_port_retries() -> u16 {
    10
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Network {
    // the backend used for oauth and refreshing of token
    #[serde(default = "def_auth_server")]
    pub auth_server: String,

    // this is the port on the local machine that receives the oauth callback
    // can be set to whatever as long as its not taken
    #[serde(default = "def_callback_port")]
    pub callback_port: u16,

    // if the port is taken and the binding fails it will retry with the next port:
    // (callback_port + 1). This determines how far that will go 
    #[serde(default = "def_max_port_retries")]
    pub max_port_retries: u16,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            auth_server: def_auth_server(),
            callback_port: def_callback_port(),
            max_port_retries: def_max_port_retries(),
        }
    }
}
