use std::marker::PhantomData;

use sha2::{Digest, Sha256};

use crate::key;

pub struct Spec {
    pub actor: String,
    pub model: String,
    pub prompt: String,
    pub repo: String,
    pub branch: String,
    pub max_turns: Option<u32>,
}

pub struct Actor<E = String> {
    name: String,
    email: String,
    keypair: key::Keypair,
    _spec: Option<Spec>,
    hash: Option<String>,
    _encoding: PhantomData<E>,
}

impl<E> Actor<E> {
    pub fn new(name: &str) -> Self {
        let keypair = key::derive(name);
        Actor {
            name: name.to_string(),
            email: format!("{}@systemic.engineer", name),
            keypair,
            _spec: None,
            hash: None,
            _encoding: PhantomData,
        }
    }

    pub fn from_spec(spec: Spec) -> Self {
        let canonical = format!(
            "actor:{}\nmodel:{}\nprompt:{}\nrepo:{}\nbranch:{}\nmax_turns:{}",
            spec.actor,
            spec.model,
            spec.prompt,
            spec.repo,
            spec.branch,
            spec.max_turns.map(|n| n.to_string()).unwrap_or_default(),
        );
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let hash = hex::encode(hasher.finalize());
        let keypair = key::derive(&spec.actor);
        Actor {
            name: spec.actor.clone(),
            email: format!("{}@systemic.engineer", spec.actor),
            keypair,
            _spec: Some(spec),
            hash: Some(hash),
            _encoding: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn pubkey(&self) -> [u8; 32] {
        self.keypair.signing_key.verifying_key().to_bytes()
    }

    pub fn hash(&self) -> Option<&str> {
        self.hash.as_deref()
    }
}
