pub mod client;
pub mod host_id;
pub mod identity;
pub mod manager;
pub mod protocol;
pub mod repo_metadata_proto;
pub mod setup;
pub mod ssh;
pub mod transport;

pub use host_id::HostId;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/remote_server.rs"));
}
