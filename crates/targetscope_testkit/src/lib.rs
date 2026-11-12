// doc/testing/overview.md#test-kit

pub mod event_probe;
pub mod failpoints;
pub mod fake_clock;
pub mod fake_program;
pub mod fixtures;
pub mod frame_feeder;
pub mod process;
pub mod redis_server;
pub mod temp_data_dir;
pub mod test_bus;

pub use event_probe::EventProbe;
pub use fake_clock::FakeClock;
pub use fake_program::{Answer, FakeProgram};
pub use process::ProgramUnderTest;
pub use redis_server::RedisTestServer;
pub use temp_data_dir::TempDataDir;
pub use test_bus::TestBus;

// doc/testing/overview.md#reference-data
pub fn testdata_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata")
}
