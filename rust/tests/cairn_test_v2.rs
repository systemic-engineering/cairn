use cairn::commit::{self, Worktree};
use cairn::encoding::{Decode, Encode};
use cairn::spec::Actor;
use cairn::File;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn init_repo() -> (tempfile::TempDir, git2::Repository) {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();
    (dir, repo)
}

fn test_actor() -> Actor<String> {
    Actor::new("mara")
}

fn test_file(name: &str) -> File<String> {
    use fragmentation::ref_::Ref as FragRef;
    use fragmentation::sha;
    fragmentation::fragment::Fragment::shard(FragRef::new(sha::hash(name), name), name.to_string())
}

fn test_worktree() -> Worktree<String> {
    Worktree::new(test_file("CLAUDE.md"))
}

// ---------------------------------------------------------------------------
// init — genesis commit
// ---------------------------------------------------------------------------

#[test]
fn init_creates_cairn() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    assert!(!cairn.id().is_zero());
}

#[test]
fn init_commit_exists() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    assert!(commit.message().unwrap().contains("init"));
}

#[test]
fn init_preserves_actor() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    assert_eq!(cairn.actor().name(), "mara");
    assert_eq!(cairn.actor().email(), "mara@systemic.engineer");
}

#[test]
fn init_preserves_ref() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    assert_eq!(cairn.ref_(), "main");
}

#[test]
fn init_has_no_parent() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    assert_eq!(commit.parent_count(), 0);
}

#[test]
fn init_has_oda_step_trailer() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    let msg = commit.message().unwrap();
    assert!(msg.contains("ODA-Step: init"));
}

// ---------------------------------------------------------------------------
// observe — creates observation commit
// ---------------------------------------------------------------------------

#[test]
fn observe_creates_new_cairn() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let init_id = cairn.id();
    let cairn =
        commit::observe(cairn, &repo, "found a pattern".to_string(), "code-review").unwrap();
    assert_ne!(cairn.id(), init_id);
}

#[test]
fn observe_parents_previous() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let init_id = cairn.id();
    let cairn = commit::observe(cairn, &repo, "data".to_string(), "test").unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    assert_eq!(commit.parent_count(), 1);
    assert_eq!(commit.parent_id(0).unwrap(), init_id);
}

#[test]
fn observe_content_readable() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let cairn =
        commit::observe(cairn, &repo, "found a pattern".to_string(), "code-review").unwrap();
    let content: String = cairn.read(&repo).unwrap();
    assert_eq!(content, "found a pattern");
}

#[test]
fn observe_trailers() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let cairn = commit::observe(cairn, &repo, "data".to_string(), "boundary-violation").unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    let msg = commit.message().unwrap();
    assert!(msg.contains("Observation-Type: boundary-violation"));
    assert!(msg.contains("ODA-Step: observe"));
}

#[test]
fn observe_author_from_actor() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let cairn = commit::observe(cairn, &repo, "data".to_string(), "test").unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    assert_eq!(commit.author().name(), Some("mara"));
    assert_eq!(commit.author().email(), Some("mara@systemic.engineer"));
}

// ---------------------------------------------------------------------------
// decide — creates decision commit
// ---------------------------------------------------------------------------

#[test]
fn decide_parents_observation() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let cairn = commit::observe(cairn, &repo, "pattern".to_string(), "code-review").unwrap();
    let obs_id = cairn.id();
    let cairn = commit::decide(cairn, &repo, "apply fix".to_string()).unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    assert_eq!(commit.parent_id(0).unwrap(), obs_id);
}

#[test]
fn decide_content_readable() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let cairn = commit::observe(cairn, &repo, "pattern".to_string(), "test").unwrap();
    let cairn = commit::decide(cairn, &repo, "rationale here".to_string()).unwrap();
    let content: String = cairn.read(&repo).unwrap();
    assert_eq!(content, "rationale here");
}

#[test]
fn decide_trailers() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let cairn = commit::observe(cairn, &repo, "data".to_string(), "boundary-violation").unwrap();
    let cairn = commit::decide(cairn, &repo, "rationale".to_string()).unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    let msg = commit.message().unwrap();
    assert!(msg.contains("ODA-Step: decide"));
    assert!(msg.contains("Observation-Type: boundary-violation"));
}

// ---------------------------------------------------------------------------
// act — creates action commit
// ---------------------------------------------------------------------------

#[test]
fn act_parents_decision() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let cairn = commit::observe(cairn, &repo, "pattern".to_string(), "code-review").unwrap();
    let cairn = commit::decide(cairn, &repo, "fix it".to_string()).unwrap();
    let dec_id = cairn.id();
    let cairn = commit::act(cairn, &repo, "patch applied".to_string()).unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    assert_eq!(commit.parent_id(0).unwrap(), dec_id);
}

#[test]
fn act_content_readable() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let cairn = commit::observe(cairn, &repo, "pattern".to_string(), "test").unwrap();
    let cairn = commit::decide(cairn, &repo, "rationale".to_string()).unwrap();
    let cairn = commit::act(cairn, &repo, "the action".to_string()).unwrap();
    let content: String = cairn.read(&repo).unwrap();
    assert_eq!(content, "the action");
}

#[test]
fn act_trailers() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let cairn = commit::observe(cairn, &repo, "data".to_string(), "boundary-violation").unwrap();
    let cairn = commit::decide(cairn, &repo, "rationale".to_string()).unwrap();
    let cairn = commit::act(cairn, &repo, "action".to_string()).unwrap();
    let commit = repo.find_commit(cairn.id()).unwrap();
    let msg = commit.message().unwrap();
    assert!(msg.contains("ODA-Step: act"));
    assert!(msg.contains("Observation-Type: boundary-violation"));
}

// ---------------------------------------------------------------------------
// full ODA chain — init → observe → decide → act
// ---------------------------------------------------------------------------

#[test]
fn full_oda_chain() {
    let (_dir, repo) = init_repo();
    let cairn = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let init_id = cairn.id();

    let cairn =
        commit::observe(cairn, &repo, "code has no tests".to_string(), "code-review").unwrap();
    let obs_id = cairn.id();
    let obs_content: String = cairn.read(&repo).unwrap();
    assert_eq!(obs_content, "code has no tests");

    let cairn = commit::decide(cairn, &repo, "add tests first".to_string()).unwrap();
    let dec_id = cairn.id();
    let dec_content: String = cairn.read(&repo).unwrap();
    assert_eq!(dec_content, "add tests first");

    let cairn = commit::act(cairn, &repo, "wrote 5 test cases".to_string()).unwrap();
    let act_content: String = cairn.read(&repo).unwrap();
    assert_eq!(act_content, "wrote 5 test cases");

    // Verify parent chain: act → dec → obs → init
    let act_commit = repo.find_commit(cairn.id()).unwrap();
    assert_eq!(act_commit.parent_id(0).unwrap(), dec_id);
    let dec_commit = repo.find_commit(dec_id).unwrap();
    assert_eq!(dec_commit.parent_id(0).unwrap(), obs_id);
    let obs_commit = repo.find_commit(obs_id).unwrap();
    assert_eq!(obs_commit.parent_id(0).unwrap(), init_id);
    let init_commit = repo.find_commit(init_id).unwrap();
    assert_eq!(init_commit.parent_count(), 0);
}

// ---------------------------------------------------------------------------
// worktree — accessible from cairn, carries through transforms
// ---------------------------------------------------------------------------

#[test]
fn worktree_accessible_from_cairn() {
    let (_dir, repo) = init_repo();
    let mut wt = Worktree::new(test_file("CLAUDE.md"));
    wt.allow_read(test_file("src/lib.rs"));
    wt.allow_write(test_file("output.md"));
    let cairn = commit::init(&repo, test_actor(), "main", wt).unwrap();
    assert_eq!(cairn.worktree().self_().data(), "CLAUDE.md");
    assert_eq!(cairn.worktree().read_list().len(), 1);
    assert_eq!(cairn.worktree().write_list().len(), 1);
}

#[test]
fn worktree_carries_through_oda() {
    let (_dir, repo) = init_repo();
    let mut wt = Worktree::new(test_file("CLAUDE.md"));
    wt.allow_read(test_file("src/lib.rs"));
    let cairn = commit::init(&repo, test_actor(), "main", wt).unwrap();
    let cairn = commit::observe(cairn, &repo, "data".to_string(), "test").unwrap();
    let cairn = commit::decide(cairn, &repo, "rationale".to_string()).unwrap();
    let cairn = commit::act(cairn, &repo, "action".to_string()).unwrap();
    // Worktree survives the full ODA chain
    assert_eq!(cairn.worktree().self_().data(), "CLAUDE.md");
    assert_eq!(cairn.worktree().read_list().len(), 1);
}

// ---------------------------------------------------------------------------
// determinism — same content → same blob OID
// ---------------------------------------------------------------------------

#[test]
fn deterministic_content_oid() {
    let (_dir, repo) = init_repo();
    let c1 = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let c1 = commit::observe(c1, &repo, "same content".to_string(), "test").unwrap();

    let c2 = commit::init(&repo, test_actor(), "main", test_worktree()).unwrap();
    let c2 = commit::observe(c2, &repo, "same content".to_string(), "test").unwrap();

    let tree1 = repo.find_commit(c1.id()).unwrap().tree().unwrap();
    let tree2 = repo.find_commit(c2.id()).unwrap().tree().unwrap();
    let data1 = tree1.get_name(".data").unwrap().id();
    let data2 = tree2.get_name(".data").unwrap().id();
    assert_eq!(data1, data2);
}

// ---------------------------------------------------------------------------
// Vec<u8> encoding
// ---------------------------------------------------------------------------

#[test]
fn bytes_encoding_roundtrip() {
    let (_dir, repo) = init_repo();
    let actor: Actor<Vec<u8>> = Actor::new("mara");
    let wt: Worktree<Vec<u8>> = Worktree::new(fragmentation::fragment::Fragment::shard_typed(
        fragmentation::ref_::Ref::new(fragmentation::sha::hash("self"), "self"),
        vec![0x01, 0x02],
    ));
    let cairn = commit::init(&repo, actor, "main", wt).unwrap();
    let data = vec![0xde, 0xad, 0xbe, 0xef];
    let cairn = commit::observe(cairn, &repo, data.clone(), "binary-test").unwrap();
    let content: Vec<u8> = cairn.read(&repo).unwrap();
    assert_eq!(content, data);
}

// ---------------------------------------------------------------------------
// Custom encoding
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum TestDialect {
    Text(String),
    Finding {
        file: String,
        line: u32,
        msg: String,
    },
}

impl Encode for TestDialect {
    fn encode(&self) -> Vec<u8> {
        match self {
            TestDialect::Text(s) => format!("T:{}", s).into_bytes(),
            TestDialect::Finding { file, line, msg } => {
                format!("F:{}:{}:{}", file, line, msg).into_bytes()
            }
        }
    }
}

impl Decode for TestDialect {
    type Error = String;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
        if let Some(rest) = s.strip_prefix("T:") {
            Ok(TestDialect::Text(rest.to_string()))
        } else if let Some(rest) = s.strip_prefix("F:") {
            let parts: Vec<&str> = rest.splitn(3, ':').collect();
            if parts.len() != 3 {
                return Err("invalid finding format".to_string());
            }
            Ok(TestDialect::Finding {
                file: parts[0].to_string(),
                line: parts[1]
                    .parse()
                    .map_err(|e: std::num::ParseIntError| e.to_string())?,
                msg: parts[2].to_string(),
            })
        } else {
            Err(format!("unknown prefix: {}", s))
        }
    }
}

#[test]
fn custom_encoding_full_oda() {
    let (_dir, repo) = init_repo();
    let actor: Actor<TestDialect> = Actor::new("mara");
    let wt: Worktree<TestDialect> = Worktree::new(fragmentation::fragment::Fragment::shard_typed(
        fragmentation::ref_::Ref::new(fragmentation::sha::hash("self"), "self"),
        TestDialect::Text("identity".to_string()),
    ));
    let cairn = commit::init(&repo, actor, "main", wt).unwrap();

    let cairn = commit::observe(
        cairn,
        &repo,
        TestDialect::Finding {
            file: "lib.rs".to_string(),
            line: 10,
            msg: "no tests".to_string(),
        },
        "code-review",
    )
    .unwrap();
    let obs_content: TestDialect = cairn.read(&repo).unwrap();
    assert_eq!(
        obs_content,
        TestDialect::Finding {
            file: "lib.rs".to_string(),
            line: 10,
            msg: "no tests".to_string(),
        }
    );

    let cairn = commit::decide(cairn, &repo, TestDialect::Text("write tests".to_string())).unwrap();
    let dec_content: TestDialect = cairn.read(&repo).unwrap();
    assert_eq!(dec_content, TestDialect::Text("write tests".to_string()));

    let cairn = commit::act(cairn, &repo, TestDialect::Text("added 3 tests".to_string())).unwrap();
    let act_content: TestDialect = cairn.read(&repo).unwrap();
    assert_eq!(act_content, TestDialect::Text("added 3 tests".to_string()));
}
