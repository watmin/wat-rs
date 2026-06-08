//! Integration test root for `src/process/` (clone/lifeline/pidfd/close_range/shutdown).
//!
//! Tests mirror the src/ layout via the {src,tests}/<namespace>/ convention.
//! build.rs auto-generates the module list from the sibling *.rs files into
//! OUT_DIR; this mod.rs is a thin include! stub. Add a test: drop a .rs here.
include!(concat!(env!("OUT_DIR"), "/process_mods.rs"));
