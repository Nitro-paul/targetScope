//! Startup order: topological sort of `depends_on`. Cycles and unknown names are configuration
//! errors; dependencies on disabled programs are ignored.

use crate::config::ProgramSpec;

/// Names of the enabled programs in start order.
pub fn start_order(programs: &[ProgramSpec]) -> Result<Vec<String>, String> {
    let _ = programs;
    todo!()
}
