use ed25519_dalek::SigningKey;

pub struct Keypair {
    pub signing_key: SigningKey,
}

pub fn derive(_nickname: &str) -> Keypair {
    todo!()
}

pub fn openssh_private_key(_keypair: &Keypair, _comment: &str) -> String {
    todo!()
}

pub fn openssh_public_line(_keypair: &Keypair, _comment: &str) -> String {
    todo!()
}
