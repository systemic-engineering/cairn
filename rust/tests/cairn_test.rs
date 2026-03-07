use cairn::key;
use cairn::session::{Ref, Session, SessionConfig};
use cairn::spec::{Actor, Spec};
use cairn::state::State;
use cairn::store;
use fragmentation::fragment::{self, Fragment};
use fragmentation::ref_::Ref as FragRef;
use fragmentation::sha;
use fragmentation::witnessed::{Author, Committer, Message, Timestamp, Witnessed};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_spec() -> Spec {
    Spec {
        actor: "mara".to_string(),
        model: "claude-sonnet-4-6".to_string(),
        prompt: "documentation specialist".to_string(),
        repo: "/Users/alexwolf/dev/projects/cairn".to_string(),
        branch: "main".to_string(),
        max_turns: None,
    }
}

fn test_config() -> SessionConfig {
    SessionConfig {
        author: "mara@systemic.engineering".to_string(),
        name: "test-session".to_string(),
        timestamp: Some("1740000000".to_string()),
    }
}

// ---------------------------------------------------------------------------
// spec tests
// ---------------------------------------------------------------------------

#[test]
fn spec_to_actor_deterministic() {
    let a1: Actor = test_spec().into();
    let a2: Actor = test_spec().into();
    assert_eq!(a1.hash, a2.hash);
}

#[test]
fn spec_different_input_different_hash() {
    let a1: Actor = test_spec().into();
    let mut s2 = test_spec();
    s2.actor = "reed".to_string();
    let a2: Actor = s2.into();
    assert_ne!(a1.hash, a2.hash);
}

#[test]
fn spec_identity_format() {
    let actor: Actor = test_spec().into();
    assert_eq!(actor.identity, "mara@systemic.engineering");
}

#[test]
fn spec_all_fields_contribute() {
    let base: Actor = test_spec().into();

    let mut s = test_spec();
    s.model = "claude-opus-4-6".to_string();
    let changed_model: Actor = s.into();
    assert_ne!(base.hash, changed_model.hash);

    let mut s = test_spec();
    s.prompt = "different prompt".to_string();
    let changed_prompt: Actor = s.into();
    assert_ne!(base.hash, changed_prompt.hash);

    let mut s = test_spec();
    s.repo = "/other/repo".to_string();
    let changed_repo: Actor = s.into();
    assert_ne!(base.hash, changed_repo.hash);

    let mut s = test_spec();
    s.branch = "develop".to_string();
    let changed_branch: Actor = s.into();
    assert_ne!(base.hash, changed_branch.hash);

    let mut s = test_spec();
    s.max_turns = Some(10);
    let changed_turns: Actor = s.into();
    assert_ne!(base.hash, changed_turns.hash);
}

#[test]
fn spec_hash_is_sha256() {
    let actor: Actor = test_spec().into();
    assert_eq!(actor.hash.len(), 64);
    assert!(actor.hash.chars().all(|c| c.is_ascii_hexdigit()));
}

// ---------------------------------------------------------------------------
// session tests
// ---------------------------------------------------------------------------

#[test]
fn session_new_empty_head() {
    let s = Session::new(test_config());
    assert_eq!(s.head(), "");
}

#[test]
fn session_act_creates_shard() {
    let mut s = Session::new(test_config());
    let r = s.act("@work scan_corpus", "scope:src/cairn.gleam\nstate:scanning");
    match &r {
        Ref::Act(_) => {}
        _ => panic!("expected ActRef"),
    }
    let frags = s.fragments_for_ref(&r);
    assert_eq!(frags.len(), 1);
    assert!(frags[0].is_shard());
    assert_eq!(frags[0].self_ref().label, "act");
}

#[test]
fn session_act_witnessed_fields() {
    let mut s = Session::new(test_config());
    let r = s.act("@work scan_corpus", "scope:src/cairn.gleam");
    let frags = s.fragments_for_ref(&r);
    let w = frags[0].self_witnessed();
    assert_eq!(w.author, Author("mara@systemic.engineering".into()));
    assert_eq!(w.committer, Committer("cairn".into()));
    assert_eq!(w.message, Message("@work scan_corpus".into()));
    assert_eq!(w.timestamp, Timestamp("1740000000".into()));
}

#[test]
fn session_act_data_preserved() {
    let mut s = Session::new(test_config());
    let r = s.act("@work scan", "scope:src/cairn.gleam\nstate:scanning");
    let frags = s.fragments_for_ref(&r);
    assert_eq!(frags[0].data(), "scope:src/cairn.gleam\nstate:scanning");
}

#[test]
fn session_decide_creates_fragment() {
    let mut s = Session::new(test_config());
    let act_ref = s.act("@annotate fn:fragment", "target:fn:fragment");
    let act_frags: Vec<_> = s.fragments_for_ref(&act_ref).into_iter().cloned().collect();
    let obs_ref = Ref::Obs("placeholder-obs".to_string());
    let dec_ref = s.decide(
        "decide: placeholder-obs",
        &obs_ref,
        "RequiredSection: fn:fragment",
        &act_frags,
    );
    match &dec_ref {
        Ref::Dec(_) => {}
        _ => panic!("expected DecRef"),
    }
    let frags = s.fragments_for_ref(&dec_ref);
    assert_eq!(frags.len(), 1);
    assert!(frags[0].is_fragment());
    assert_eq!(frags[0].self_ref().label, "dec");
}

#[test]
fn session_decide_wraps_acts() {
    let mut s = Session::new(test_config());
    let act_ref = s.act("@annotate fn:fragment", "target:fn:fragment");
    let act_frags: Vec<_> = s.fragments_for_ref(&act_ref).into_iter().cloned().collect();
    let obs_ref = Ref::Obs("placeholder-obs".to_string());
    let dec_ref = s.decide("decide: obs", &obs_ref, "RequiredSection", &act_frags);
    let dec_frags = s.fragments_for_ref(&dec_ref);
    assert_eq!(dec_frags[0].children().len(), 1);
    assert_eq!(dec_frags[0].data(), "RequiredSection");
}

#[test]
fn session_observe_creates_fragment() {
    let mut s = Session::new(test_config());
    let obs_ref_input = Ref::Obs("placeholder-obs".to_string());
    let dec_ref = s.decide("decide: obs", &obs_ref_input, "Rule", &[]);
    let dec_frags: Vec<_> = s.fragments_for_ref(&dec_ref).into_iter().cloned().collect();
    let obs_ref = s.observe(
        "observe: concept:fn:fragment",
        "concept:fn:fragment",
        "fn:fragment present",
        &dec_frags,
    );
    match &obs_ref {
        Ref::Obs(_) => {}
        _ => panic!("expected ObsRef"),
    }
    let frags = s.fragments_for_ref(&obs_ref);
    assert_eq!(frags.len(), 1);
    assert!(frags[0].is_fragment());
    assert_eq!(frags[0].self_ref().label, "obs");
}

#[test]
fn session_observe_wraps_decisions() {
    let mut s = Session::new(test_config());
    let obs_ref_input = Ref::Obs("placeholder-obs".to_string());
    let dec_ref = s.decide("decide: obs", &obs_ref_input, "Rule", &[]);
    let dec_frags: Vec<_> = s.fragments_for_ref(&dec_ref).into_iter().cloned().collect();
    let obs_ref = s.observe("observe: concept", "concept:fn", "data", &dec_frags);
    let frags = s.fragments_for_ref(&obs_ref);
    assert_eq!(frags[0].children().len(), 1);
    assert_eq!(frags[0].data(), "data");
}

#[test]
fn session_commit_returns_root() {
    let mut s = Session::new(test_config());
    let (root, sha) = s.commit("commit: test-session", &[]);
    assert!(!sha.is_empty());
    assert!(root.is_fragment());
    assert_eq!(root.self_ref().label, "root");
    assert_eq!(root.data(), "test-session");
}

#[test]
fn session_commit_updates_head() {
    let mut s = Session::new(test_config());
    assert_eq!(s.head(), "");
    let (_, sha) = s.commit("commit: test-session", &[]);
    assert_eq!(s.head(), sha);
}

#[test]
fn session_deterministic() {
    let mut s1 = Session::new(test_config());
    let r1 = s1.act("@annotate fn:fragment", "target:fn:fragment");
    let act_frags1: Vec<_> = s1.fragments_for_ref(&r1).into_iter().cloned().collect();
    let obs_ref1 = Ref::Obs("obs1".to_string());
    let dec_ref1 = s1.decide("decide: obs1", &obs_ref1, "RequiredSection", &act_frags1);
    let dec_frags1: Vec<_> = s1
        .fragments_for_ref(&dec_ref1)
        .into_iter()
        .cloned()
        .collect();
    let obs_ref1b = s1.observe(
        "observe: concept",
        "concept:fn:fragment",
        "fn:fragment present",
        &dec_frags1,
    );
    let obs_frags1: Vec<_> = s1
        .fragments_for_ref(&obs_ref1b)
        .into_iter()
        .cloned()
        .collect();
    let (_, sha1) = s1.commit("commit: test-session", &obs_frags1);

    let mut s2 = Session::new(test_config());
    let r2 = s2.act("@annotate fn:fragment", "target:fn:fragment");
    let act_frags2: Vec<_> = s2.fragments_for_ref(&r2).into_iter().cloned().collect();
    let obs_ref2 = Ref::Obs("obs1".to_string());
    let dec_ref2 = s2.decide("decide: obs1", &obs_ref2, "RequiredSection", &act_frags2);
    let dec_frags2: Vec<_> = s2
        .fragments_for_ref(&dec_ref2)
        .into_iter()
        .cloned()
        .collect();
    let obs_ref2b = s2.observe(
        "observe: concept",
        "concept:fn:fragment",
        "fn:fragment present",
        &dec_frags2,
    );
    let obs_frags2: Vec<_> = s2
        .fragments_for_ref(&obs_ref2b)
        .into_iter()
        .cloned()
        .collect();
    let (_, sha2) = s2.commit("commit: test-session", &obs_frags2);

    assert_eq!(sha1, sha2);
}

#[test]
fn session_ado_full_cascade() {
    let mut s = Session::new(test_config());
    let act_ref = s.act("@work scan", "scope:src/cairn.gleam");
    let act_frags: Vec<_> = s.fragments_for_ref(&act_ref).into_iter().cloned().collect();
    let obs_input = Ref::Obs("obs-sha".to_string());
    let dec_ref = s.decide("decide: obs", &obs_input, "RequiredSection", &act_frags);
    let dec_frags: Vec<_> = s.fragments_for_ref(&dec_ref).into_iter().cloned().collect();
    let obs_ref = s.observe(
        "observe: file:cairn.gleam",
        "file:cairn.gleam",
        "found",
        &dec_frags,
    );
    let obs_frags: Vec<_> = s.fragments_for_ref(&obs_ref).into_iter().cloned().collect();
    let (root, _) = s.commit("commit: test-session", &obs_frags);

    // root -> obs -> dec -> act (depth 4)
    assert_eq!(root.children().len(), 1);
    let obs = &root.children()[0];
    assert_eq!(obs.children().len(), 1);
    let dec = &obs.children()[0];
    assert_eq!(dec.children().len(), 1);
    let act = &dec.children()[0];
    assert!(act.is_shard());
}

#[test]
fn session_head_updates_after_each_op() {
    let mut s = Session::new(test_config());
    assert_eq!(s.head(), "");

    let r = s.act("@work scan", "data");
    let head_after_act = s.head().to_string();
    assert!(!head_after_act.is_empty());
    assert_eq!(s.head(), r.sha());

    let act_frags: Vec<_> = s.fragments_for_ref(&r).into_iter().cloned().collect();
    let obs_input = Ref::Obs("obs".to_string());
    let dr = s.decide("decide", &obs_input, "rule", &act_frags);
    let head_after_decide = s.head().to_string();
    assert_ne!(head_after_act, head_after_decide);
    assert_eq!(s.head(), dr.sha());
}

#[test]
fn session_last_root_after_commit() {
    let mut s = Session::new(test_config());
    assert!(s.last_root().is_none());
    let (root, sha) = s.commit("commit: test-session", &[]);
    let lr = s.last_root().unwrap();
    assert_eq!(lr.1, sha);
    assert_eq!(
        fragment::hash_fragment(lr.0),
        fragment::hash_fragment(&root)
    );
}

// ---------------------------------------------------------------------------
// state tests
// ---------------------------------------------------------------------------

#[test]
fn state_empty() {
    let actor: Actor = test_spec().into();
    let state = State::new(actor);
    assert!(state.sessions.is_empty());
}

#[test]
fn state_single_session() {
    let actor: Actor = test_spec().into();
    let mut state = State::new(actor);
    state.append("abc123".to_string(), "1740000000".to_string());
    assert_eq!(state.sessions.len(), 1);
    assert_eq!(state.sessions[0].root_sha, "abc123");
    assert!(state.sessions[0].previous.is_none());
}

#[test]
fn state_chain_links_via_hash() {
    let actor: Actor = test_spec().into();
    let mut state = State::new(actor);
    state.append("sha-first".to_string(), "1740000000".to_string());
    state.append("sha-second".to_string(), "1740000001".to_string());
    assert_eq!(state.sessions[1].previous.as_deref(), Some("sha-first"));
}

#[test]
fn state_append_preserves_previous() {
    let actor: Actor = test_spec().into();
    let mut state = State::new(actor);
    state.append("sha-1".to_string(), "1".to_string());
    state.append("sha-2".to_string(), "2".to_string());
    state.append("sha-3".to_string(), "3".to_string());
    assert_eq!(state.sessions.len(), 3);
    assert_eq!(state.sessions[0].root_sha, "sha-1");
    assert_eq!(state.sessions[1].root_sha, "sha-2");
    assert_eq!(state.sessions[2].root_sha, "sha-3");
    assert_eq!(state.sessions[2].previous.as_deref(), Some("sha-2"));
}

// ---------------------------------------------------------------------------
// store tests
// ---------------------------------------------------------------------------

#[test]
fn store_write_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let frag = Fragment::shard(
        FragRef::new(sha::hash("test"), "test"),
        Witnessed::new(
            Author("test".into()),
            Committer("cairn".into()),
            Timestamp("0".into()),
            Message("test".into()),
        ),
        "test data",
    );
    store::write(&frag, dir.path().to_str().unwrap()).unwrap();
    let sha = fragment::hash_fragment(&frag);
    let path = dir.path().join(&sha);
    assert!(path.exists());
}

#[test]
fn store_verify_passes() {
    let dir = tempfile::tempdir().unwrap();
    let frag = Fragment::shard(
        FragRef::new(sha::hash("verify-test"), "test"),
        Witnessed::new(
            Author("test".into()),
            Committer("cairn".into()),
            Timestamp("0".into()),
            Message("test".into()),
        ),
        "verify data",
    );
    store::write(&frag, dir.path().to_str().unwrap()).unwrap();
    assert!(store::verify(&frag, dir.path().to_str().unwrap()).is_ok());
}

#[test]
fn store_verify_missing() {
    let dir = tempfile::tempdir().unwrap();
    let frag = Fragment::shard(
        FragRef::new(sha::hash("missing-test"), "test"),
        Witnessed::new(
            Author("test".into()),
            Committer("cairn".into()),
            Timestamp("0".into()),
            Message("test".into()),
        ),
        "missing data",
    );
    // Don't write — verify should fail
    let result = store::verify(&frag, dir.path().to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("missing:"));
}

#[test]
fn store_verify_tampered() {
    let dir = tempfile::tempdir().unwrap();
    let frag = Fragment::shard(
        FragRef::new(sha::hash("tamper-test"), "test"),
        Witnessed::new(
            Author("test".into()),
            Committer("cairn".into()),
            Timestamp("0".into()),
            Message("test".into()),
        ),
        "original data",
    );
    store::write(&frag, dir.path().to_str().unwrap()).unwrap();
    // Tamper with the file
    let sha = fragment::hash_fragment(&frag);
    let path = dir.path().join(&sha);
    std::fs::write(&path, "tampered content").unwrap();
    let result = store::verify(&frag, dir.path().to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("tampered:"));
}

#[test]
fn store_verify_deep_tree() {
    let dir = tempfile::tempdir().unwrap();
    let dir_str = dir.path().to_str().unwrap();
    let child = Fragment::shard(
        FragRef::new(sha::hash("child"), "act"),
        Witnessed::new(
            Author("test".into()),
            Committer("cairn".into()),
            Timestamp("0".into()),
            Message("test".into()),
        ),
        "child data",
    );
    let parent = Fragment::new_fragment(
        FragRef::new(sha::hash("parent"), "root"),
        Witnessed::new(
            Author("test".into()),
            Committer("cairn".into()),
            Timestamp("0".into()),
            Message("test".into()),
        ),
        "parent data",
        vec![child.clone()],
    );
    // Write parent but not child
    fragmentation::git::write(&parent, dir_str).unwrap();
    let result = store::verify(&parent, dir_str);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("missing:"));
}

// ---------------------------------------------------------------------------
// key tests
// ---------------------------------------------------------------------------

#[test]
fn key_derivation_deterministic() {
    let k1 = key::derive("mara");
    let k2 = key::derive("mara");
    assert_eq!(
        k1.signing_key.verifying_key().to_bytes(),
        k2.signing_key.verifying_key().to_bytes(),
    );
}

#[test]
fn key_different_nickname_different_key() {
    let k1 = key::derive("mara");
    let k2 = key::derive("reed");
    assert_ne!(
        k1.signing_key.verifying_key().to_bytes(),
        k2.signing_key.verifying_key().to_bytes(),
    );
}

#[test]
fn key_pubkey_is_32_bytes() {
    let k = key::derive("mara");
    assert_eq!(k.signing_key.verifying_key().to_bytes().len(), 32);
}

#[test]
fn key_seed_is_32_bytes() {
    let k = key::derive("mara");
    assert_eq!(k.signing_key.to_bytes().len(), 32);
}

#[test]
fn key_openssh_format() {
    let k = key::derive("mara");
    let pem = key::openssh_private_key(&k, "cairn/mara");
    assert!(pem.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----\n"));
    assert!(pem.ends_with("-----END OPENSSH PRIVATE KEY-----\n"));
}
