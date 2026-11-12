//! Tools used by all Rust tests (doc/testing/overview.md#test-kit).
//! Every test starts from a fresh state: its own Redis, new folders, a new database.

pub mod event_probe;
pub mod failpoints;
pub mod fake_clock;
pub mod fake_program;
pub mod fixtures;
pub mod frame_feeder;
pub mod redis_server;
pub mod temp_data_dir;
pub mod test_bus;

pub use event_probe::EventProbe;
pub use fake_clock::FakeClock;
pub use fake_program::FakeProgram;
pub use frame_feeder::FrameFeeder;
pub use redis_server::RedisTestServer;
pub use temp_data_dir::TempDataDir;
pub use test_bus::TestBus;

/// Root of the repository's reference data.
pub fn testdata_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata")
}
