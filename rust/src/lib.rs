pub mod commit;
pub mod encoding;
pub mod key;
pub mod session;
pub mod spec;
pub mod state;
pub mod store;

pub type Id = git2::Oid;
pub type Ref = String;
pub type File<E = String> = fragmentation::fragment::Fragment<E>;
