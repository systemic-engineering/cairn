use fragmentation::fragment::Fragment;

pub struct SessionConfig {
    pub author: String,
    pub name: String,
    pub timestamp: Option<String>,
}

#[derive(Debug)]
pub enum Ref {
    Act(String),
    Dec(String),
    Obs(String),
}

impl Ref {
    pub fn sha(&self) -> &str {
        match self {
            Ref::Act(s) | Ref::Dec(s) | Ref::Obs(s) => s,
        }
    }
}

#[allow(dead_code)]
pub struct Session {
    config: SessionConfig,
    store: Vec<(String, Fragment)>,
    last_root: Option<(Fragment, String)>,
    head: String,
}

impl Session {
    pub fn new(_config: SessionConfig) -> Self {
        todo!()
    }

    pub fn head(&self) -> &str {
        todo!()
    }

    pub fn config(&self) -> &SessionConfig {
        todo!()
    }

    pub fn last_root(&self) -> Option<(&Fragment, &str)> {
        todo!()
    }

    pub fn fragments_for_ref(&self, _r: &Ref) -> Vec<&Fragment> {
        todo!()
    }

    pub fn act(&mut self, _annotation: &str, _data: &str) -> Ref {
        todo!()
    }

    pub fn decide(
        &mut self,
        _annotation: &str,
        _obs_ref: &Ref,
        _rule: &str,
        _acts: &[Fragment],
    ) -> Ref {
        todo!()
    }

    pub fn observe(
        &mut self,
        _annotation: &str,
        _ref_str: &str,
        _data: &str,
        _decisions: &[Fragment],
    ) -> Ref {
        todo!()
    }

    pub fn commit(&mut self, _annotation: &str, _observations: &[Fragment]) -> (Fragment, String) {
        todo!()
    }
}
