//! The fire round's passes, one module each.
//!
//! `fire_fixpoint_delta_armed` was 1774 lines and 12 levels deep, with nine
//! passes braided into one body (`DESIGN-STONE-partire-fire-loop`). The seams
//! were already drawn by its own `// ── N.` section comments; these modules are
//! those sections, moved out whole.
//!
//! Each extraction is behaviour-identical by contract: no clone removed, no
//! name improved, no comment rewritten in the same commit as a move, so the
//! diff can be reviewed by reading it. The gate is the oracle differential —
//! a second implementation of the same semantics — plus the floor.

mod root_join;
pub(crate) use root_join::*;

#[cfg(test)]
mod round_census;
#[cfg(test)]
pub(crate) use round_census::*;
