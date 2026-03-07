use cairn::spec::{Actor, Spec};

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

// ---------------------------------------------------------------------------
// Actor::new
// ---------------------------------------------------------------------------

#[test]
fn actor_new_has_name() {
    let actor: Actor = Actor::new("mara");
    assert_eq!(actor.name(), "mara");
}

#[test]
fn actor_new_has_email() {
    let actor: Actor = Actor::new("mara");
    assert_eq!(actor.email(), "mara@systemic.engineer");
}

#[test]
fn actor_new_has_keypair() {
    let actor: Actor = Actor::new("mara");
    assert_eq!(actor.pubkey().len(), 32);
}

#[test]
fn actor_new_no_hash() {
    let actor: Actor = Actor::new("mara");
    assert!(actor.hash().is_none());
}

#[test]
fn actor_deterministic() {
    let a: Actor = Actor::new("mara");
    let b: Actor = Actor::new("mara");
    assert_eq!(a.pubkey(), b.pubkey());
}

#[test]
fn actor_different_name_different_key() {
    let a: Actor = Actor::new("mara");
    let b: Actor = Actor::new("reed");
    assert_ne!(a.pubkey(), b.pubkey());
}

// ---------------------------------------------------------------------------
// Actor::from_spec
// ---------------------------------------------------------------------------

#[test]
fn actor_from_spec_has_hash() {
    let actor: Actor = Actor::from_spec(test_spec());
    assert!(actor.hash().is_some());
}

#[test]
fn actor_from_spec_hash_is_sha256() {
    let actor: Actor = Actor::from_spec(test_spec());
    let hash = actor.hash().unwrap();
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn actor_from_spec_deterministic() {
    let a: Actor = Actor::from_spec(test_spec());
    let b: Actor = Actor::from_spec(test_spec());
    assert_eq!(a.hash(), b.hash());
}

#[test]
fn actor_from_spec_name_from_actor_field() {
    let actor: Actor = Actor::from_spec(test_spec());
    assert_eq!(actor.name(), "mara");
}

#[test]
fn actor_from_spec_email_from_actor_field() {
    let actor: Actor = Actor::from_spec(test_spec());
    assert_eq!(actor.email(), "mara@systemic.engineer");
}

// ---------------------------------------------------------------------------
// Encoding phantom — Actor<E> is generic over encoding type
// ---------------------------------------------------------------------------

#[test]
fn actor_encoding_phantom_string() {
    let _: Actor<String> = Actor::new("mara");
}

#[test]
fn actor_encoding_phantom_bytes() {
    let _: Actor<Vec<u8>> = Actor::new("mara");
}
