use cairn::encoding::{Decode, Encode};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn init_repo() -> (tempfile::TempDir, git2::Repository) {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();
    (dir, repo)
}

fn test_actor() -> cairn::spec::Actor {
    cairn::spec::Actor::new("mara")
}

// ---------------------------------------------------------------------------
// Actor
// ---------------------------------------------------------------------------

#[test]
fn actor_has_name() {
    let actor = cairn::spec::Actor::new("mara");
    assert_eq!(actor.name(), "mara");
}

#[test]
fn actor_has_email() {
    let actor = cairn::spec::Actor::new("mara");
    assert_eq!(actor.email(), "mara@systemic.engineer");
}

#[test]
fn actor_has_keypair() {
    let actor = cairn::spec::Actor::new("mara");
    let pubkey = actor.pubkey();
    assert_eq!(pubkey.len(), 32);
}

#[test]
fn actor_deterministic() {
    let a = cairn::spec::Actor::new("mara");
    let b = cairn::spec::Actor::new("mara");
    assert_eq!(a.pubkey(), b.pubkey());
}

#[test]
fn actor_different_name_different_key() {
    let a = cairn::spec::Actor::new("mara");
    let b = cairn::spec::Actor::new("reed");
    assert_ne!(a.pubkey(), b.pubkey());
}

// ---------------------------------------------------------------------------
// Worktree
// ---------------------------------------------------------------------------

#[test]
fn worktree_empty() {
    let wt: cairn::cairn::Worktree<String> = cairn::cairn::Worktree::new("identity".to_string());
    assert_eq!(wt.self_(), &"identity".to_string());
    assert!(wt.read_list().is_empty());
    assert!(wt.write_list().is_empty());
}

#[test]
fn worktree_with_read() {
    let mut wt: cairn::cairn::Worktree<String> =
        cairn::cairn::Worktree::new("identity".to_string());
    wt.allow_read("file1.md".to_string());
    wt.allow_read("file2.md".to_string());
    assert_eq!(wt.read_list().len(), 2);
}

#[test]
fn worktree_with_write() {
    let mut wt: cairn::cairn::Worktree<String> =
        cairn::cairn::Worktree::new("identity".to_string());
    wt.allow_write("output.md".to_string());
    assert_eq!(wt.write_list().len(), 1);
}

// ---------------------------------------------------------------------------
// Cairn<E> construction with Actor + Worktree
// ---------------------------------------------------------------------------

#[test]
fn cairn_open_with_actor() {
    let (_dir, repo) = init_repo();
    let actor = test_actor();
    let wt = cairn::cairn::Worktree::new("self".to_string());
    let cairn: cairn::cairn::Cairn<String> =
        cairn::cairn::Cairn::open(repo, actor, "main", wt);
    assert_eq!(cairn.actor().name(), "mara");
    assert_eq!(cairn.branch(), "main");
}

#[test]
fn cairn_actor_identity_in_commits() {
    let (_dir, repo) = init_repo();
    let actor = test_actor();
    let wt = cairn::cairn::Worktree::new("self".to_string());
    let cairn: cairn::cairn::Cairn<String> =
        cairn::cairn::Cairn::open(repo, actor, "main", wt);
    let oid = cairn
        .observe("data".to_string(), "test", None)
        .unwrap();
    let repo = cairn.repo();
    let commit = repo.find_commit(oid).unwrap();
    assert_eq!(commit.author().name(), Some("mara"));
    assert_eq!(
        commit.author().email(),
        Some("mara@systemic.engineer")
    );
}

// ---------------------------------------------------------------------------
// ODA still works through the new API
// ---------------------------------------------------------------------------

#[test]
fn cairn_oda_with_actor() {
    let (_dir, repo) = init_repo();
    let actor = test_actor();
    let wt = cairn::cairn::Worktree::new("self".to_string());
    let cairn: cairn::cairn::Cairn<String> =
        cairn::cairn::Cairn::open(repo, actor, "main", wt);

    let obs = cairn
        .observe("found issue".to_string(), "code-review", None)
        .unwrap();
    let dec = cairn.decide(obs, "fix it".to_string()).unwrap();
    let act = cairn.act(dec, "fixed".to_string()).unwrap();

    // Chain intact
    let repo = cairn.repo();
    let act_commit = repo.find_commit(act).unwrap();
    assert_eq!(act_commit.parent_id(0).unwrap(), dec);
    let dec_commit = repo.find_commit(dec).unwrap();
    assert_eq!(dec_commit.parent_id(0).unwrap(), obs);

    // Content round-trips
    assert_eq!(cairn.read::<String>(obs).unwrap(), "found issue");
    assert_eq!(cairn.read::<String>(dec).unwrap(), "fix it");
    assert_eq!(cairn.read::<String>(act).unwrap(), "fixed");
}

// ---------------------------------------------------------------------------
// Worktree accessible from Cairn
// ---------------------------------------------------------------------------

#[test]
fn cairn_worktree_accessible() {
    let (_dir, repo) = init_repo();
    let actor = test_actor();
    let mut wt = cairn::cairn::Worktree::new("identity".to_string());
    wt.allow_read("src/lib.rs".to_string());
    wt.allow_write("output.md".to_string());
    let cairn: cairn::cairn::Cairn<String> =
        cairn::cairn::Cairn::open(repo, actor, "main", wt);
    assert_eq!(cairn.worktree().self_(), &"identity".to_string());
    assert_eq!(cairn.worktree().read_list().len(), 1);
    assert_eq!(cairn.worktree().write_list().len(), 1);
}
