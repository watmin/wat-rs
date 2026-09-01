//! Arc 109 Stone — `src/reflect/`: the introspection surface's home.
//!
//! **Builder, `DESIGN-STONE-the-reflect-home.md`:** the home for the reflection API's
//! IMPL — `:wat::runtime::lookup-define`/`signature-of-defn`/`signature-of-fn`/
//! `return-type-of`/`body-of`/`rename-callable-name`/`extract-arg-names`/
//! `extract-arg-types`/`field-names-of`/`field-types-of`, `:wat::form::matches?`, and
//! `:wat::core::macroexpand`/`macroexpand-1`. `src/intrinsic/reflect.rs` is this home's
//! EDGE — it already existed; this stone gives its implementations somewhere to live
//! other than `src/runtime.rs`.
//!
//! ## The one contract decision: split by ROLE, never by declaration FORM
//!
//! ```text
//! src/reflect/render.rs   internal state → AST — Function/TypeScheme/MacroDef/TypeDef builders
//! src/reflect/lookup.rs   find a binding — the uniform Binding enum + lookup_form
//! src/reflect/verbs.rs    the `*-of` API surface — the bulk of the reflection verbs
//! src/reflect/match.rs    form matching — `:wat::form::matches?` + its clause walker
//! src/reflect/expand.rs   macroexpand — the two macroexpand special forms
//! ```
//!
//! `render.rs` is the emission layer every other file calls into; `lookup.rs` supplies
//! the uniform `Binding` both `render.rs`'s callers and `verbs.rs` dispatch on; `verbs.rs`
//! is the largest file because the `*-of` surface is the bulk of the API; `match.rs` and
//! `expand.rs` are unrelated to each other and to the other three, but each is too small
//! to be its own stone — DESIGN grouped them here as "the rest of the range."
//!
//! ## What this stone shipped, and what it did not
//!
//! 32 of DESIGN's listed 33 functions relocate out of `src/runtime.rs` into this module,
//! split by role as above. Behaviour is unchanged — every introspection verb resolves
//! identically; only the location moved.
//!
//! Two items in the moved range stay in `runtime.rs`, neither by this stone's own
//! authority — one flagged by name in the brief, the other caught only by applying a
//! general STOP this stone's brief stated in advance:
//!
//! - **`require_bundle`** (STOP-1) — a **holon** helper (both its callers are
//!   `src/intrinsic/holon/atom.rs`) sitting in this range by proximity, not membership.
//!   `src/holon/`'s business, not this stone's.
//! - **`eval_metadata_of`** (STOP-4, a finding this stone's own execution surfaced) — its
//!   body reads the top-level `intrinsic` module's own registry accessor and constructs
//!   that module's `ToEnumValue`/`DefinedIn`/`Layer`/`Arity` types throughout. Moving it
//!   would put a same-crate path to that module into `src/reflect/*.rs`, which this
//!   stone's acceptance gate (a textual grep for that exact path, over `src/reflect/*.rs`,
//!   must read 0) forbids categorically — no import shape dodges a textual grep. See
//!   `verbs.rs`'s module doc for the full account.
//!
//! ## EDGE vs IMPL — the architecture this module is one instance of
//!
//! `src/intrinsic/<domain>` is the EDGE — registration and delegation, the kernel's rim.
//! `src/<domain>/` is the IMPL — the actual work. Already built for `collection`, `declare`,
//! `edn`, `holon`, `kernel`, `numeric`, `rete`, `stream`, `string`. This module's edge,
//! `src/intrinsic/reflect.rs`, already existed before this stone — the one real boundary
//! this module must respect is that it never references its own edge module (verified by
//! the acceptance gate above).

pub(crate) mod expand;
pub(crate) mod lookup;
#[path = "match.rs"]
pub(crate) mod r#match;
pub(crate) mod render;
pub(crate) mod verbs;
