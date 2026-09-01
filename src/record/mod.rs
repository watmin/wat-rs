//! Arc 109 Stone — `src/record/`: the aggregate family's home.
//!
//! **Builder, `DESIGN-STONE-the-record-home.md`:** *"the edge has been waiting, and
//! it says so in writing"* — `src/intrinsic/record.rs:12` bumped all seven of its
//! delegate targets to `pub(crate)` "so this module can reach them — no body
//! [moves]", anticipating a home that was never built until this stone.
//! `src/intrinsic/record.rs` is this home's EDGE — it already existed, registers
//! all ten aggregate verbs, and delegates every one back here.
//!
//! ## The one contract decision: split by ROLE, never by declaration FORM
//!
//! ```text
//! src/record/construct.rs   struct-new · variant · aggregate-new · construct_aggregate · kwargs-construct
//! src/record/access.rs      struct-field · Record/field-at · record? · List?
//! src/record/project.rs     project_surface_attrs · parse_projection_args · to-record
//! src/record/update.rs      record_field_map · record->map · Record/same-data? · record_assoc_inner · Record/assoc
//! ```
//!
//! A helper is not a separate concern from the verb it exists for —
//! `construct_aggregate` lands beside `eval_aggregate_new`/`eval_kwargs_construct`
//! (both its only callers), `record_assoc_inner` beside `eval_record_assoc`,
//! `parse_projection_args`/`project_surface_attrs` beside `eval_to_core_record` —
//! a helper's reason to change is its verb's, never a `helpers.rs`/`util.rs`.
//!
//! ## What this stone shipped, and what it did not
//!
//! 17 items (~1,087 lines) relocate out of `src/runtime.rs` into this module,
//! split by role as above. Behaviour is unchanged — every aggregate verb
//! resolves identically; only the location moved.
//!
//! **`eval_retag_op` did NOT move here.** It sat between `eval_variant` and
//! `eval_struct_field` in `runtime.rs` and reads like a record verb (it retags a
//! variant), but its sole caller was `src/intrinsic/kernel/serve.rs` —
//! `kernel::serve`'s business, not this home's. That call was correct: arc 109
//! Stone B (the seven kernel sub-modules) has since homed it in
//! `src/kernel/serve.rs`, exactly the module this note predicted. The eighth
//! intruder this campaign found sitting inside a proposed module range.
//!
//! ## EDGE vs IMPL — the architecture this module is one instance of
//!
//! `src/intrinsic/<domain>` is the EDGE — registration and delegation, the kernel's
//! rim. `src/<domain>/` is the IMPL — the actual work. Already built for
//! `collection`, `declare`, `edn`, `holon`, `kernel`, `numeric`, `reflect`, `rete`,
//! `stream`, `string`. This module's edge, `src/intrinsic/record.rs`, already
//! existed before this stone — the one real boundary this module must respect is
//! that it never reaches back out to its own edge (the `intrinsic` registry module).

pub(crate) mod access;
pub(crate) mod construct;
pub(crate) mod project;
pub(crate) mod update;
