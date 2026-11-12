//! Main loop: startup sequence, supervision, shutdown (doc/programs/launcher.md).

use std::collections::HashMap;

use targetscope_common::bus::Bus;

use crate::config::LauncherConfig;
use crate::program::ProgramHandle;

pub struct Launcher {
    pub config: LauncherConfig,
    pub programs: HashMap<String, ProgramHandle>,
    /// Available after Redis is up
    pub bus: Option<Bus>,
}

impl Launcher {
    pub fn new(config: LauncherConfig) -> Self {
        Self {
            config,
            programs: HashMap::new(),
            bus: None,
        }
    }

    /// 1. lock, 2. folders + cleanup_dead_nodes, 3. Redis, 4. programs in order,
    /// 5. serve launcher:requests; then supervise until SIGINT/SIGTERM.
    pub async fn run(self) -> Result<(), String> {
        todo!()
    }

    /// Stop restarting; SIGTERM in reverse order (10 s, then SIGKILL); Redis SHUTDOWN.
    async fn shutdown(&mut self) {
        todo!()
    }
}
