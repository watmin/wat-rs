# BRIEF — 296 derive Strike 3b: single-field tuple variants + derive `LoadError`

> **Executor: one sonnet, MAIN tree (the `../holon-rs` path dep breaks worktree builds — do NOT use a worktree).**
> Orchestrator drew this + `DESIGN-296-derive.md § STRIKE 3b`; weighs the kill forced-clean by its own gate AND the
> emitted diff. **Commit nothing.** Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject any
> `.claude/worktrees/` path. Do NOT spawn subagents.

## The work (one paragraph)
Teach `#[derive(ToEdn)]` **single-field tuple variants**, then apply the derive to `LoadErrorKind` and DELETE
`LoadError`'s hand-written `to_edn` match body — byte-identical. A single-field tuple variant `Foo(T)` emits
`#wat.kernel/Foo {:<key> <field.to_edn()>}` where `<key>` is REQUIRED via a variant-level `#[to_edn(key = "…")]`;
a multi-field tuple and a keyless single tuple both stay `compile_error!`. This is the exact sibling of Strikes 1/2a
(ConfigError) + 2b (CheckError) + 3a (TypeError/StdlibError) — mirror those.

## Read first (in order)
- **`docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-296-derive.md § STRIKE 3b`** — the full contract (the tuple
  rule, the per-variant LoadError mapping, the scope carve, the proof, the blast radius). THIS BRIEF IS THE BUILD ORDER.
- **`crates/wat-macros/src/to_edn_derive.rs:560-832`** — the code generator. `Fields::Named` (`:602`) + `Fields::Unit`
  (`:733`) are the two handled shapes; `Fields::Unnamed(_)` (`:804`) is the `compile_error!` you replace. Variant-level
  attrs are parsed by `parse_variant_attrs` (grep it); ADD a `key: Option<String>` slot for the tuple field's key.
- **`src/config.rs:260-267`** — the exemplar Pattern-A wrapper (`splice_span(self.kind.to_edn(), &self.span)`) LoadError
  copies. `src/types/error.rs:24` (3a) — the same pattern on a bigger family.
- **`src/load.rs:255-431`** — `LoadError` / `LoadErrorKind` def + the hand `impl ToEdn` (`:365-431`) you DELETE, and
  `impl WatError for LoadError` (`:336-363`) you LEAVE UNCHANGED. The exact current per-variant output is `:377-429`.
- **`tests/diagnostics/probe_arc296_derive_configerror_identical.rs`** + **`probe_arc296_3a_typeerror_derive_identical.rs`**
  — the byte-identical probe SHAPE to mirror for LoadError (snapshot the HEAD wire strings; assert the derived output
  equals them).
- **`crates/wat-macros/tests/ui/ui_to_edn_rejects_tuple_variant.rs`** (+ `.stderr`) — the KEYLESS tuple `Tuple(String)`;
  it STAYS a reject (now "requires `#[to_edn(key=…)]`"); regenerate its `.stderr`.

## The rooms (grounded 2026-07-01, HEAD fc1fdf3a — reproduce EXACTLY)
The derived `#[derive(ToEdn)]` on `LoadErrorKind` must produce, per variant (all + `:span` via the wrapper's `splice_span`):
- `MalformedLoadForm { reason }` → `{:reason …}` (snake→kebab default).
- `SetterInLoadedFile { loaded_path, setter_head }` → `{:loaded-path … :setter-head …}`.
- `DuplicateLoad { path }` → `{:path …}`.
- `CycleDetected { cycle: Vec<String> }` → `{:cycle [str …]}` (the `Vec<T: ToEdn>` building block).
- `Fetch(LoadFetchError)` → `#[to_edn(key = "cause")]` → `{:cause (inner.to_edn())}` — **THE NEW TUPLE RULE.**
- `Parse { path, err }` → `path` default `:path`; `err` → `#[to_edn(key = "cause", via = crate::to_edn::error_edn_of)]`
  → `{:cause (error_edn_of(err))}` (the RECURSIVE FLOOR — `error_edn_of` = `err.error_edn()`, NOT raw `to_edn`).
- `VerificationFailed { path, err }` → `path` `:path`; `err` → `#[to_edn(key = "cause")]` → `{:cause (err.to_edn())}`.

`LoadFetchError` + `HashError` KEEP their existing hand `impl ToEdn` (building-block leaves — the derived Fetch/
VerificationFailed arms just call `.to_edn()` on them; do NOT touch or derive them).

## Implementation sketch (the strike path — fill it, don't reinvent the shape)
1. **`to_edn_derive.rs`** — in `parse_variant_attrs`, accept `key = "…"` (LitStr, grammar-constrained like the
   field-level `key`). In `derive_variant`, replace the `Fields::Unnamed(_) => compile_error!` arm:
   ```rust
   Fields::Unnamed(f) if f.unnamed.len() == 1 => {
       let key = variant_attr.key.ok_or_else(|| compile_error!("single-field tuple variant requires #[to_edn(key = \"…\")]"))?;
       // Self::Foo(__0) => Tagged("wat.kernel"/"Foo", Map[(key, __0.to_edn())])   (+ literal/computed_via if present)
   }
   Fields::Unnamed(_) => compile_error!("ToEdn derive supports single-field tuple variants only (multi-field is ambiguous)"),
   ```
   Guard: `key` on a Named/Unit variant is a `compile_error!` (only valid on a single-field tuple).
2. **`src/load.rs`** — `#[derive(crate::to_edn::ToEdn)]` (or the crate's derive path, mirror config.rs) on
   `LoadErrorKind`; annotate `Fetch`/`Parse.err`/`VerificationFailed.err` per the rooms; replace the hand
   `impl ToEdn for LoadError` (`:365-431`) with the `splice_span` wrapper (mirror `src/config.rs:260-267`).
   `impl WatError for LoadError` UNCHANGED.
3. **Probe** — new co-located `tests/diagnostics/probe_arc296_3b_loaderror_derive_identical.rs` mirroring the
   ConfigError/TypeError identical probes: assert `wat_edn::write(&e.to_edn())` for a representative value of ALL 7
   `LoadErrorKind` variants (known + unknown span) equals the snapshotted HEAD string.
4. **UI fixtures** — regenerate `ui_to_edn_rejects_tuple_variant.stderr` (message changed); ADD a pass proof (a unit
   test in the derive's own `to_edn_derive_tests.rs`, or a `tests/pass/`-style fixture): an enum with
   `#[to_edn(key = "cause")] Wrap(String)` derives to `#wat.kernel/Wrap {:cause "…"}`.

## Blast radius (STOP + report if you exceed this)
`crates/wat-macros/src/to_edn_derive.rs` + its unit tests + `tests/ui/` fixtures · `src/load.rs` · the new probe.
NOTHING else. LoadFetchError/HashError untouched. No `Display`/`WatError` changes.

## STOP triggers (REJECTION criteria — ship nothing, report the gap; NOT permission to defer)
- **STOP-1:** if the derived LoadError output is NOT byte-identical to the HEAD snapshot for any variant, STOP and
  report the exact diff. Do NOT "fix" it by editing the snapshot to match your output.
- **STOP-2:** if `error_edn_of` cannot be used as a field-level `via` on `Parse.err` (signature mismatch), STOP and
  report — do NOT fall back to raw `to_edn` (it would drop the recursive floor).
- **STOP-3:** if the tuple-variant change ripples past the named blast radius, STOP and report what breaks.

## ⛔ THE ANTI-WEAKENING RULE (non-negotiable — PROBATIO FLEXA MENTITVR)
A probe is NEVER yours to weaken to reach green. Do not invert an assertion, relax a snapshot, `#[ignore]` a test, or
soften the byte-identical check to make the gate pass. If a probe goes red, the CODE is wrong — fix the code, or STOP
and report. The orchestrator weighs the **emitted diff**, not your report; a moved probe is the loudest possible tell.

## Report back (per the strike)
The derive diff (the new tuple arm + the `key` slot); the LoadError diff (derive annotations + deleted hand body +
wrapper); the new probe + its assertions (paste the snapshotted strings); the regenerated `.stderr`; the FULL gate
count (`cargo nextest run --release`); `cargo build --release` warning delta vs HEAD; any STOP; any deviation.
