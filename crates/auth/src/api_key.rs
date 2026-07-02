use rand::RngCore;

#[derive(Debug, Clone)]
pub struct GeneratedApiKey {
    pub token: String,
    pub key_prefix: String,
}

pub fn generate_api_key(prefix: &str) -> GeneratedApiKey {
    let mut secret_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret_bytes);

    let secret = hex::encode(secret_bytes);

    let key_prefix = format!("ctx_{}_{}", prefix, &secret[..8]);
    let token = format!("{}.{}", key_prefix, secret);

    GeneratedApiKey { token, key_prefix }
}