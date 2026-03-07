use crate::spec::Actor;

pub struct SessionRecord {
    pub root_sha: String,
    pub previous: Option<String>,
    pub timestamp: String,
}

pub struct State {
    pub actor: Actor,
    pub sessions: Vec<SessionRecord>,
}

impl State {
    pub fn new(_actor: Actor) -> Self {
        todo!()
    }

    pub fn append(&mut self, _root_sha: String, _timestamp: String) {
        todo!()
    }
}
