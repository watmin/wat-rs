# BRIEF — 296 N3: per-phase error tag namespaces (single-source, refactorable)

> **Executor: one sonnet, MAIN tree** (the `../holon-rs` path dep breaks worktree builds — do NOT use a worktree).
> Orchestrator drew this + `DESIGN-296-N3-per-phase-tag-namespaces.md`; weighs forced-clean by its OWN gate AND the
> emitted wire EDN. **Commit nothing.** Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`.
> Do NOT spawn subagents. This is a WIDE wire change (every error tag) — the cascade is EXPECTED; ride it to zero.

## The work (one paragraph)
Every top-level error FAMILY tags under its PHASE namespace instead of the uniform `#wat.kernel/`. The namespace strings
live in ONE new module `src/error_ns.rs` as `pub const`s — a single source of truth so a future rename (toward 1.0.0) is
one edit. The `#[derive(ToEdn)]` gains an enum-level `#[to_edn(namespace = <path>)]` sub-key that emits a REFERENCE to
the const (never a baked literal). Annotate the 7 derived families; point the 2 hand-written families (Parse/Resolve) and
the shared-infra wrappers at the consts. Then ride the test cascade: every golden/CLI/probe asserting
`#wat.kernel/<ErrorVariant>` updates to its phase ns — EXACTLY, never weakened.

## Read first (in order)
- **`docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-296-N3-per-phase-tag-namespaces.md`** — the mapping, the
  single-source mechanism, the shared-infra line, the out-of-scope carve. THIS BRIEF IS THE BUILD ORDER.
- **`crates/wat-macros/src/to_edn_derive.rs:780,847,864,930`** — the 4 spots emitting
  `::wat_edn::Tag::ns("wat.kernel", #variant_name_str)`. The `"wat.kernel"` literal is what becomes `#namespace_tokens`.
  Grep `parse_enum_attrs` (or the enum-level attribute parse; if none exists, add one alongside the variant-attr parse).
- **`src/to_edn.rs:290`** (`edn_tag`), **`src/check/error_edn.rs:105`**, **`src/runtime_error_edn.rs:210`**,
  **`src/macros/error_edn.rs:195`** — the hand wrappers hardcoding `Tag::ns("wat.kernel", variant)`.
- **`src/parser.rs`** (ParseError orphan `impl ToEdn`) + **`src/resolve/error.rs`** (ResolveError impl) — the 2 hand families.
- **`tests/diagnostics/probe_arc296_n3_per_phase_namespaces.rs`** — the committed RED probe (`#[ignore]`'d). UN-IGNORE it
  and make it GREEN (CheckError→#wat.check, TypeError→#wat.type, RuntimeError→#wat.runtime, LoadError→#wat.load, nested
  LoadFetchError stays #wat.kernel/NotFound).
- The byte-identical golden probes that WILL need their ns prefix updated (exact, not weakened):
  `probe_arc298_3_runtime_derive_identical.rs` (33 → `#wat.runtime/`), `probe_arc298_3_macro_derive_identical.rs`
  (13 → `#wat.macro/`), `probe_arc296_3a_typeerror_derive_identical.rs` (→ `#wat.type/`),
  `probe_arc296_derive_configerror_identical.rs` (→ `#wat.config/`), `probe_arc296_3b_loaderror_derive_identical.rs`
  (→ `#wat.load/`), plus any other `#wat.kernel/<ErrorVariant>` assertion (grep — see below).

## The mechanism (grounded — reproduce EXACTLY)
1. **`src/error_ns.rs` (NEW)** — the single source of truth:
   ```rust
   //! THE single source of truth for error tag namespaces. Rename HERE → every production
   //! emission site follows (one edit). Test-literal goldens carry the string by nature; a
   //! codemod/sed sweep is the refactor for those.
   pub const CONFIG:  &str = "wat.config";
   pub const CHECK:   &str = "wat.check";
   pub const TYPE:    &str = "wat.type";
   pub const STDLIB:  &str = "wat.stdlib";
   pub const LOAD:    &str = "wat.load";
   pub const RUNTIME: &str = "wat.runtime";
   pub const MACRO:   &str = "wat.macro";
   pub const PARSE:   &str = "wat.parse";
   pub const RESOLVE: &str = "wat.resolve";
   pub const KERNEL:  &str = "wat.kernel";
   ```
   Declare `pub mod error_ns;` in `src/lib.rs`.
2. **The derive** (`to_edn_derive.rs`) — parse an enum-level `#[to_edn(namespace = <path>)]` where `<path>` is a
   `syn::Path` (grammar-constrained — a path, NOT a string literal, NOT an arbitrary expr; mirror the `via` bare-ident
   constraint). Compute `namespace_tokens`: the path if present, else `quote! { "wat.kernel" }` (back-compat default).
   Replace the 4 hardcoded `"wat.kernel"` literals with `#namespace_tokens`. The generated code becomes
   `::wat_edn::Tag::ns(crate::error_ns::CHECK, #variant_name_str)` — a const REFERENCE, resolving in the `wat` crate.
3. **Annotate the 7 derived KIND enums** with their const path:
   `#[to_edn(namespace = crate::error_ns::CONFIG)]` on `ConfigErrorKind`; `CHECK` on `CheckErrorKind`; `TYPE` on
   `TypeErrorKind`; `STDLIB` on `StdlibErrorKind`; `LOAD` on `LoadErrorKind`; `RUNTIME` on `RuntimeErrorKind`; `MACRO` on
   `MacroErrorKind`.
4. **The hand wrappers + orphan impls** — replace `"wat.kernel"` with the const: `edn_tag` (to_edn.rs) →
   `crate::error_ns::KERNEL` (it serves the shared-infra blocks); `check/error_edn.rs` local tagger →
   `crate::error_ns::CHECK`; `runtime_error_edn.rs` → `crate::error_ns::RUNTIME`; `macros/error_edn.rs` →
   `crate::error_ns::MACRO`; `parser.rs` ParseError → `crate::error_ns::PARSE`; `resolve/error.rs` → `crate::error_ns::RESOLVE`.
   Shared-infra blocks (Remedy/LoadFetchError/HashError/ClauseAttempt/Span/Location/etc.) keep `KERNEL`.
5. **Un-ignore the RED probe** → GREEN. **Ride the golden cascade** — update every `#wat.kernel/<ErrorVariant>`
   assertion to its phase ns, EXACTLY (change only the ns prefix; the rest of the golden byte-for-byte unchanged).

## Proof
- The RED probe (un-ignored) → GREEN.
- **The refactor guarantee (grounded, mechanical):**
  `grep -rn '"wat\.\(kernel\|check\|runtime\|macro\|type\|parse\|config\|load\|resolve\|stdlib\)"' src/ crates/wat-macros/src/ --include=*.rs`
  → in PRODUCTION code (not tests) the ONLY hits are the 10 const definitions in `error_ns.rs` + the derive's back-compat
  default literal. No family's namespace is a scattered literal.
- FULL gate `cargo nextest run --release` = 0 failed. `cargo build --release` clean (warning delta ~0).

## Out of scope (do NOT touch — coupled to the de-stringify strike)
- **`Failure`, `ProcessDiedError`, `ThreadDiedError`** — registered wat types; their tag comes from the registered CLASS
  PATH, not the derive. Leave them `#wat.kernel/…`. Their re-namespacing rides the SEPARATE de-stringify strike.
- **`StartupError`** — transparent passthrough; the inner phase ns shows through. Nothing to do.

## STOP triggers (REJECTION criteria — ship nothing, report the gap; NOT permission to defer)
- **STOP-1:** if a family's namespace cannot come from a single `error_ns` const (something forces a scattered literal),
  STOP and report — the single-source guarantee is the point.
- **STOP-2:** if the cascade reaches NON-error code (a non-error type was tagged `#wat.kernel/` and now ambiguous), STOP
  and report — do NOT re-namespace a non-error.
- **STOP-3:** if a byte-identical golden cannot be updated to the new ns without also changing non-ns bytes, STOP and
  report the exact diff (it would mean the ns change touched more than the prefix).

## ⛔ THE ANTI-WEAKENING RULE (non-negotiable — PROBATIO FLEXA MENTITVR / IN TENEBRIS VIDEO)
The byte-identical goldens are the widest surface here and the easiest place to hide a weakening. NEVER downgrade an
`assert_eq!` to `assert!(contains)`, never invert, never `#[ignore]`. Updating a golden means changing ONLY the
`#wat.kernel/` → `#wat.<phase>/` prefix and NOTHING else — the rest of the string byte-for-byte identical. If a golden
won't go green with only the prefix changed, the CODE is wrong (or the ns leaked into a shared block) — STOP and report.
The orchestrator will `git diff` the goldens and confirm every change is a pure prefix swap.

## Report back
The `error_ns.rs` module; the derive diff (the `namespace` sub-key + the 4-literal → `#namespace_tokens` change); the 7
annotations; the wrapper/orphan diffs; the RED probe un-ignore + its GREEN output; the golden-cascade summary (which
probes, how many goldens, confirm pure-prefix-swap); the refactor-guarantee grep output; the FULL gate count; the
`cargo build --release` warning delta; any STOP; any deviation.
