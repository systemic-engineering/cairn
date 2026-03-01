/// Session: ODA state for a witnessed AI session.
///
/// An agent runs with gall wired. Each call — observe, decide, act —
/// materializes a Fragment node. commit seals the session and returns
/// the root hash. Same inputs, same hash. Content-addressed.
///
/// ODA structure in Fragment terms:
///   Fragment(session_name)           ← root
///     Fragment(obs_data)             ← observation
///       Fragment(dec_data)           ← decision
///         Shard(act_data)            ← action
import fragmentation
import gleam/list

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

pub opaque type Session {
  Session(
    observations: List(fragmentation.Fragment),
    decisions: List(fragmentation.Fragment),
    actions: List(fragmentation.Fragment),
  )
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

pub fn new() -> Session {
  Session(observations: [], decisions: [], actions: [])
}

// ---------------------------------------------------------------------------
// ODA
// ---------------------------------------------------------------------------

/// Record an observation. Returns updated session and the observation's SHA.
/// ref: source location (e.g. "fragmentation.gleam:33")
/// data: what was observed
pub fn observe(
  session: Session,
  ref: String,
  data: String,
) -> #(Session, String) {
  let w = witnessed("observe", ref)
  let frag =
    fragmentation.shard(
      fragmentation.ref(fragmentation.hash(ref <> data), "obs"),
      w,
      data,
    )
  let sha = fragmentation.hash_fragment(frag)
  let updated =
    Session(..session, observations: list.append(session.observations, [frag]))
  #(updated, sha)
}

/// Record a decision linked to an observation (by SHA).
/// rule: the structural conclusion (e.g. "RequiredSection: fn:fragment")
pub fn decide(
  session: Session,
  obs_sha: String,
  rule: String,
) -> #(Session, String) {
  let w = witnessed("decide", obs_sha)
  let frag =
    fragmentation.shard(
      fragmentation.ref(fragmentation.hash(obs_sha <> rule), "dec"),
      w,
      rule,
    )
  let sha = fragmentation.hash_fragment(frag)
  let updated =
    Session(..session, decisions: list.append(session.decisions, [frag]))
  #(updated, sha)
}

/// Record an action linked to a decision (by SHA).
/// annotation: what was done (e.g. "annotate: fn:fragment is required")
pub fn act(
  session: Session,
  dec_sha: String,
  annotation: String,
) -> #(Session, String) {
  let w = witnessed("act", dec_sha)
  let frag =
    fragmentation.shard(
      fragmentation.ref(fragmentation.hash(dec_sha <> annotation), "act"),
      w,
      annotation,
    )
  let sha = fragmentation.hash_fragment(frag)
  let updated =
    Session(..session, actions: list.append(session.actions, [frag]))
  #(updated, sha)
}

/// Seal the session. Returns root Fragment SHA.
/// name: session name (e.g. "mara.witness: fragmentation")
pub fn commit(session: Session, name: String) -> #(Session, String) {
  let w = witnessed("commit", name)
  let children =
    list.flatten([session.observations, session.decisions, session.actions])
  let content = name <> children_hash(children)
  let root =
    fragmentation.fragment(
      fragmentation.ref(fragmentation.hash(content), "root"),
      w,
      name,
      children,
    )
  let sha = fragmentation.hash_fragment(root)
  #(session, sha)
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

pub fn observations(session: Session) -> List(fragmentation.Fragment) {
  session.observations
}

pub fn decisions(session: Session) -> List(fragmentation.Fragment) {
  session.decisions
}

pub fn actions(session: Session) -> List(fragmentation.Fragment) {
  session.actions
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn witnessed(phase: String, data: String) -> fragmentation.Witnessed {
  fragmentation.witnessed(
    fragmentation.Author("gall"),
    fragmentation.Committer("gall/session"),
    fragmentation.Timestamp(data),
    fragmentation.Message(phase),
  )
}

fn children_hash(children: List(fragmentation.Fragment)) -> String {
  list.map(children, fragmentation.hash_fragment)
  |> list.fold("", fn(acc, h) { acc <> h })
}
