//! tests/reflection/ integration test group — build.rs auto-generates the module list from sibling
//! *.rs into OUT_DIR; this mod.rs is a thin include! stub. Add a test: drop a .rs here.
include!(concat!(env!("OUT_DIR"), "/reflection_mods.rs"));
