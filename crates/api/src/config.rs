use cortex_chain_monad::model::{MonadLocation, MonadNetwork};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_address: String,
    pub database_url: String,
    pub monad: MonadConfig,
}

#[derive(Debug, Clone)]
pub struct MonadConfig {
    pub mainnet_staking_contract: String,
    pub testnet_staking_contract: String,
    pub mainnet_default_validator: Option<String>,
    pub testnet_default_validator: Option<String>,
    pub default_location: MonadLocation,
    pub mainnet_rpc_url: Option<String>,
    pub testnet_rpc_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            bind_address: std::env::var("BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),

            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://user:password@localhost/dbname".to_string()
            }),

            monad: MonadConfig::from_env(),
        }
    }
}

impl MonadConfig {
    pub fn from_env() -> Self {
        Self {
            mainnet_staking_contract: std::env::var("MONAD_MAINNET_STAKING_CONTRACT")
                .expect("MONAD_MAINNET_STAKING_CONTRACT is required"),

            testnet_staking_contract: std::env::var("MONAD_TESTNET_STAKING_CONTRACT")
                .expect("MONAD_TESTNET_STAKING_CONTRACT is required"),

            mainnet_default_validator: std::env::var("MONAD_MAINNET_DEFAULT_VALIDATOR")
                .ok()
                .filter(|value| !value.is_empty()),

            testnet_default_validator: std::env::var("MONAD_TESTNET_DEFAULT_VALIDATOR")
                .ok()
                .filter(|value| !value.is_empty()),

            default_location: parse_monad_location(
                &std::env::var("MONAD_DEFAULT_LOCATION")
                    .unwrap_or_else(|_| "us-dallas".to_string()),
            ),
            mainnet_rpc_url: std::env::var("MONAD_MAINNET_RPC_URL").ok().filter(|value| !value.is_empty()),
            testnet_rpc_url: std::env::var("MONAD_TESTNET_RPC_URL").ok().filter(|value| !value.is_empty()),
        }
    }

    pub fn staking_contract(&self, network: &MonadNetwork) -> &str {
        match network {
            MonadNetwork::Mainnet => &self.mainnet_staking_contract,
            MonadNetwork::Testnet => &self.testnet_staking_contract,
        }
    }

    pub fn default_validator(&self, network: &MonadNetwork) -> Option<&str> {
        match network {
            MonadNetwork::Mainnet => self.mainnet_default_validator.as_deref(),
            MonadNetwork::Testnet => self.testnet_default_validator.as_deref(),
        }
    }

    pub fn default_location(&self) -> MonadLocation {
        self.default_location.clone()
    }

    pub fn rpc_url(&self, network: &MonadNetwork) -> Option<&str> {
        match network {
            MonadNetwork::Mainnet => self.mainnet_rpc_url.as_deref(),
            MonadNetwork::Testnet => self.testnet_rpc_url.as_deref(),
        }
    }
}

fn parse_monad_location(value: &str) -> MonadLocation {
    match value {
        "us-dallas" => MonadLocation::UsDallas,
        other => panic!("Unsupported MONAD_DEFAULT_LOCATION '{}'", other),
    }
}