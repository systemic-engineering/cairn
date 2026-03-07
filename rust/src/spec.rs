pub struct Spec {
    pub actor: String,
    pub model: String,
    pub prompt: String,
    pub repo: String,
    pub branch: String,
    pub max_turns: Option<u32>,
}

pub struct Actor {
    pub spec: Spec,
    pub hash: String,
    pub identity: String,
}

impl From<Spec> for Actor {
    fn from(_spec: Spec) -> Self {
        todo!()
    }
}
