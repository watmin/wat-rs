//! Integration test root for the **scratch-pad probe manifest** — the runner that
//! executes named `wat-scripts/scratch-pad/*.wat` probes on the floor and asserts on
//! their output.
//!
//! Why this group exists (excursus 001 `the-probes-run-in-the-floor`): every probe
//! under `wat-scripts/scratch-pad/` is PARSED and type-checked by
//! `every_wat_scripts_file_loads_on_the_current_runtime`, and the overwhelming
//! majority are never RUN by anything. A lifecycle invariant proved once by hand, on
//! one box, and then left unguarded is a claim, not a proof. This group is where the
//! driven ones come back every floor.
//!
//! It is a group of its own rather than a file in `tests/services/` or
//! `tests/process/` because a row here is neither: it is a whole `wat` PROCESS run
//! from the outside, and the group name is what nextest prints in the Summary.
//!
//! Tests mirror the src/ layout via the {src,tests}/<namespace>/ convention; this
//! group has no `src/` sibling, which is itself the point — it tests programs, not
//! modules. build.rs auto-generates the module list from the sibling *.rs files into
//! OUT_DIR; this mod.rs is a thin include! stub. Add a test: drop a .rs here.
include!(concat!(env!("OUT_DIR"), "/probes_mods.rs"));
