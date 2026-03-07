use crate::encoding::{Decode, Encode};
use crate::spec::Actor;
use crate::{File, Id};

#[derive(Debug)]
pub enum CairnError {
    Git(git2::Error),
    Decode(String),
    Missing(String),
}

impl std::fmt::Display for CairnError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CairnError::Git(e) => write!(f, "git: {}", e),
            CairnError::Decode(e) => write!(f, "decode: {}", e),
            CairnError::Missing(e) => write!(f, "missing: {}", e),
        }
    }
}

impl std::error::Error for CairnError {}

impl From<git2::Error> for CairnError {
    fn from(e: git2::Error) -> Self {
        CairnError::Git(e)
    }
}

pub struct Worktree<E = String> {
    self_: File<E>,
    read: Vec<File<E>>,
    write: Vec<File<E>>,
}

impl<E> Worktree<E> {
    pub fn new(self_: File<E>) -> Self {
        Worktree {
            self_,
            read: Vec::new(),
            write: Vec::new(),
        }
    }

    pub fn self_(&self) -> &File<E> {
        &self.self_
    }

    pub fn read_list(&self) -> &[File<E>] {
        &self.read
    }

    pub fn write_list(&self) -> &[File<E>] {
        &self.write
    }

    pub fn allow_read(&mut self, file: File<E>) {
        self.read.push(file);
    }

    pub fn allow_write(&mut self, file: File<E>) {
        self.write.push(file);
    }
}

pub struct Cairn<E = String> {
    id: Id,
    actor: Actor<E>,
    ref_: String,
    worktree: Worktree<E>,
}

impl<E> Cairn<E> {
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn actor(&self) -> &Actor<E> {
        &self.actor
    }

    pub fn ref_(&self) -> &str {
        &self.ref_
    }

    pub fn worktree(&self) -> &Worktree<E> {
        &self.worktree
    }

    pub fn read<D: Decode>(&self, repo: &git2::Repository) -> Result<D, CairnError> {
        let commit = repo.find_commit(self.id)?;
        let tree = commit.tree()?;
        let data_entry = tree
            .get_name(".data")
            .ok_or_else(|| CairnError::Missing(".data entry".to_string()))?;
        let blob = repo.find_blob(data_entry.id())?;
        D::decode(blob.content()).map_err(|e| CairnError::Decode(format!("{}", e)))
    }
}

/// Genesis — first commit. No parent.
pub fn init<E>(
    repo: &git2::Repository,
    actor: Actor<E>,
    ref_: &str,
    worktree: Worktree<E>,
) -> Result<Cairn<E>, CairnError> {
    let message = format!("init: {}\n\nODA-Step: init", actor.name());
    let id = write_commit(repo, &actor, actor.name().as_bytes(), &message, None)?;
    Ok(Cairn {
        id,
        actor,
        ref_: ref_.to_string(),
        worktree,
    })
}

/// Observe — creates observation commit parented on previous cairn.
pub fn observe<E: Encode>(
    cairn: Cairn<E>,
    repo: &git2::Repository,
    content: E,
    obs_type: &str,
) -> Result<Cairn<E>, CairnError> {
    let bytes = content.encode();
    let message = format!(
        "observe: {}\n\nObservation-Type: {}\nODA-Step: observe",
        obs_type, obs_type
    );
    let id = write_commit(repo, &cairn.actor, &bytes, &message, Some(cairn.id))?;
    Ok(Cairn {
        id,
        actor: cairn.actor,
        ref_: cairn.ref_,
        worktree: cairn.worktree,
    })
}

/// Decide — creates decision commit parented on observation.
pub fn decide<E: Encode>(
    cairn: Cairn<E>,
    repo: &git2::Repository,
    rationale: E,
) -> Result<Cairn<E>, CairnError> {
    let bytes = rationale.encode();
    let obs_type = observation_type(repo, cairn.id)?;
    let message = format!("decide\n\nObservation-Type: {}\nODA-Step: decide", obs_type);
    let id = write_commit(repo, &cairn.actor, &bytes, &message, Some(cairn.id))?;
    Ok(Cairn {
        id,
        actor: cairn.actor,
        ref_: cairn.ref_,
        worktree: cairn.worktree,
    })
}

/// Act — creates action commit parented on decision.
pub fn act<E: Encode>(
    cairn: Cairn<E>,
    repo: &git2::Repository,
    content: E,
) -> Result<Cairn<E>, CairnError> {
    let bytes = content.encode();
    let obs_type = observation_type(repo, cairn.id)?;
    let message = format!("act\n\nObservation-Type: {}\nODA-Step: act", obs_type);
    let id = write_commit(repo, &cairn.actor, &bytes, &message, Some(cairn.id))?;
    Ok(Cairn {
        id,
        actor: cairn.actor,
        ref_: cairn.ref_,
        worktree: cairn.worktree,
    })
}

fn observation_type(repo: &git2::Repository, oid: Id) -> Result<String, CairnError> {
    let commit = repo.find_commit(oid)?;
    let msg = commit.message().unwrap_or("");
    for line in msg.lines() {
        if let Some(value) = line.strip_prefix("Observation-Type: ") {
            return Ok(value.to_string());
        }
    }
    Err(CairnError::Missing("Observation-Type trailer".to_string()))
}

fn write_commit<E>(
    repo: &git2::Repository,
    actor: &Actor<E>,
    bytes: &[u8],
    message: &str,
    parent: Option<Id>,
) -> Result<Id, CairnError> {
    let blob_oid = repo.blob(bytes)?;
    let mut builder = repo.treebuilder(None)?;
    builder.insert(".data", blob_oid, 0o100644)?;
    let tree_oid = builder.write()?;
    let tree = repo.find_tree(tree_oid)?;

    let sig = git2::Signature::now(actor.name(), actor.email())?;

    let parents: Vec<git2::Commit> = match parent {
        Some(oid) => vec![repo.find_commit(oid)?],
        None => vec![],
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    Ok(repo.commit(None, &sig, &sig, message, &tree, &parent_refs)?)
}
