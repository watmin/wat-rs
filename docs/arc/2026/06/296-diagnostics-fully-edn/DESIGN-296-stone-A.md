# Stone A — the real `ToEdn` move: trait → wat-edn, derive → wat-to-edn-derive

**Arc 296 closing strike, stone A (completion).** The probe-proven foundation
landed at `7877d215` (the `wat-to-edn-derive` crate wired, the `derive`
re-export proven from wat-reader with no cycle, uuid workspace-dep'd, `mint`
killed). That commit uses a **throwaway `ProbeToEdn` stub**. This stone replaces
the stub with the **real `ToEdn`** — moving the trait down to wat-edn and the
derive body into the new crate — so a wat type is EDN structurally, from the
foundational crate up.

## Why

The `ToEdn` trait lives in the `wat` crate (`src/to_edn.rs:52`); the derive
lives in `wat-macros` (`src/to_edn_derive.rs`), which deps `wat-reader` — the
circle that forbids wat-reader from ever deriving. Option 3 (already decided):
the trait's home is wat-edn (the root — it speaks only `OwnedValue`), and the
derive rides in `wat-to-edn-derive` (deps nothing of ours), re-exported by
wat-edn under the default-on `derive` feature. Then wat-reader — and every
crate — can `#[derive(wat_edn::ToEdn)]`.

## The one contract decision (pinned)

The moved derive emits **absolute** paths to the trait's new home:
`crate::to_edn::ToEdn` → `::wat_edn::ToEdn` (~6 generated sites). This is not a
new choice — it is what relocating the trait *means* (crate-relative paths
resolve in the consumer crate; only `wat` has `crate::to_edn`, so absolute is
required for cross-crate derivation).

**Out of scope = REJECTED for this stone:**
- Restructuring `Span` into `#wat.core/Span {file line col end:Option<Pos>}` — that is **stone B**.
- The `crate::to_edn::push_span_field` path (derive line ~846) stays **crate-relative**. It references `Span` (wat-reader) + `crate::panic_hook` (wat) → it cannot live in wat-edn (cycle). It is emitted ONLY for a field whose type is `Span` (the `is_span_type` gate) — all such types are wat-crate error types that *have* `crate::to_edn::push_span_field`. No wat-reader type triggers it. It deletes itself in **stone B** when `Span: ToEdn` makes a span field a normal `span.to_edn()` field. Bounded, named, next-stone — not a deferral.

## The rooms (read in order)

1. `crates/wat-edn/src/lib.rs` ~98–113 — the `ProbeToEdn` stub trait + `pub use wat_to_edn_derive::ProbeToEdn`. **Replace** with the real `ToEdn` trait + `pub use wat_to_edn_derive::ToEdn`.
2. `src/to_edn.rs:52–54` — the real `ToEdn` trait def. **Move** its body to wat-edn (room 1); here leave `pub use wat_edn::ToEdn;` so `crate::to_edn::ToEdn` STILL RESOLVES (this is what keeps the 40 `impl ToEdn` blocks + all `use crate::to_edn::ToEdn` sites UNTOUCHED — do not sweep them).
3. `src/to_edn.rs:169` — `impl ToEdn for OwnedValue`. **Move** into wat-edn (both types are wat-edn's now; the impl belongs there).
4. `crates/wat-macros/src/to_edn_derive.rs` (1124 lines) — the real derive body. **Move** into `crates/wat-to-edn-derive/src/lib.rs` (replacing the stub). It is self-contained: imports only `proc_macro2`, `quote`, `syn`; its helpers (`snake_to_kebab`, `is_span_type`, `parse_enum_attrs`, `parse_field_attrs`) are in-file. In the moved body, change the ~6 `crate::to_edn::ToEdn` sites → `::wat_edn::ToEdn` (lines ~745, 750, 907, 916, 984 + the doc comments). LEAVE `crate::to_edn::push_span_field` (~846) crate-relative (out of scope, above).
5. `crates/wat-macros/src/lib.rs:30` (`mod to_edn_derive;`) + `:99–106` (the `#[proc_macro_derive(ToEdn, attributes(to_edn))] pub fn derive_to_edn`). **Delete** both; the `#[proc_macro_derive]` wrapper moves to the new crate's lib.rs.
6. `crates/wat-macros/tests/ui/ui_to_edn_*` (10 files: 5 `.rs` + 5 `.stderr`). **Move** to `crates/wat-to-edn-derive/tests/ui/` (the derive they exercise moved). Wire a trybuild harness in the new crate if needed.
7. `src/to_edn_derive_tests.rs` — the S1 test uses `#[derive(wat_macros::ToEdn)]`. **Redirect** to `wat_edn::ToEdn`.
8. `crates/wat-reader/src/lib.rs` — the `probe_arc296_stone_a` module (ProbeToEdn consumer). **Delete** it. **Add** a small `#[cfg(test)]` proof that the REAL derive works cross-crate: a throwaway struct `#[derive(wat_edn::ToEdn)]` (no namespace attr → defaults to `wat.kernel`) whose `.to_edn()` returns a `#wat.kernel/…` tagged record. This persists as the cycle-break guard until stone B makes Span do it for real.

## Implementation sketch

- wat-edn/src/lib.rs: `pub trait ToEdn { fn to_edn(&self) -> OwnedValue; }` (+ its doc), `impl ToEdn for OwnedValue { fn to_edn(&self) -> OwnedValue { self.clone() } }` (match the existing impl at src/to_edn.rs:169), and `#[cfg(feature = "derive")] pub use wat_to_edn_derive::ToEdn;`.
- src/to_edn.rs: replace the `pub trait ToEdn {...}` block with `pub use wat_edn::ToEdn;`. Delete the `impl ToEdn for OwnedValue` block (moved). Keep WatError, push_span_field, splice_span, edn_span, everything else.
- wat-to-edn-derive/src/lib.rs: the derive body from to_edn_derive.rs + the `#[proc_macro_derive(ToEdn, attributes(to_edn))] pub fn derive_to_edn(...)` wrapper from wat-macros/lib.rs, with `crate::to_edn::ToEdn` → `::wat_edn::ToEdn`.

## Blast radius

wat-edn/src/lib.rs · src/to_edn.rs · crates/wat-to-edn-derive/ · wat-macros (strip) · the 10 UI test files (move) · src/to_edn_derive_tests.rs (1 attr) · wat-reader/src/lib.rs (swap probe→real test). **The 40 `impl ToEdn` blocks and ~30 `use crate::to_edn::ToEdn` sites are NOT touched** — the `pub use` re-export keeps `crate::to_edn::ToEdn` resolving. If you find yourself editing those, STOP — the re-export is missing or wrong.

## STOP triggers (rejection criteria — ship nothing, report the gap)

- **STOP-1:** if moving the `ToEdn` trait to wat-edn requires wat-edn to reference any `wat`/`wat-reader` type (a cycle), STOP and report. (The trait is `fn to_edn(&self) -> OwnedValue` — pure wat-edn vocab — so it should move clean.)
- **STOP-2:** if `pub use wat_edn::ToEdn;` in src/to_edn.rs does NOT keep `crate::to_edn::ToEdn` resolving — i.e., the 40 impl sites go red — STOP and report. Do NOT sweep 40 files.
- **STOP-3:** if the moved derive body needs anything from wat-macros beyond syn/quote/proc-macro2 + its in-file helpers, STOP and report.
- **STOP-4 (trybuild goldens):** the UI `.stderr` files may shift because the trait path in error messages changed (`crate::to_edn::ToEdn` → `wat_edn::ToEdn`). Regenerate with `TRYBUILD=overwrite` ONLY after confirming by eye the diff is *just the path* — never a behavior change. If a golden diff shows a behavior change (e.g. `ui_to_edn_rejects_struct` now contradicts S1's struct support), STOP and report it as a finding.

## Expectations (scorecard — fixed before the strike)

| # | what | command | expected |
|---|---|---|---|
| 1 | workspace builds | `cargo build` | Finished, 0 errors |
| 2 | full test suite green | `cargo test` (or the nextest gate) | 0 failed |
| 3 | the real derive works cross-crate | `cargo test -p wat-reader` (the new real-derive test) | passes — a wat-reader struct derives `wat_edn::ToEdn` |
| 4 | S1 test still green via new path | `cargo test -p wat --test … struct_derive_emits_namespaced_tagged_record_with_optional_nested` (or its home) | passes with `#[derive(wat_edn::ToEdn)]` |
| 5 | 40 impl sites untouched | `git diff --stat` | no changes to the ~17 files holding `impl ToEdn for` beyond path-neutral; the diff is trait-move + derive-move only |
| 6 | wat-macros no longer exports ToEdn | `grep -rn "proc_macro_derive(ToEdn" crates/wat-macros` | empty |
| 7 | derive emits absolute trait path | `grep -n "crate::to_edn::ToEdn" crates/wat-to-edn-derive/src/lib.rs` | empty (only `::wat_edn::ToEdn`); `push_span_field` may remain `crate::to_edn::` |

**Runtime prediction:** 20–35 min. **Trap-doors:** the trybuild `.stderr` path shift (STOP-4); a hidden `crate::to_edn::` reference in the derive beyond ToEdn/push_span_field; the `impl ToEdn for OwnedValue` body (confirm it's `self.clone()` or whatever src/to_edn.rs:169 does, moved verbatim).

## Reference

- The proven pattern: probe commit `7877d215` (the `ProbeToEdn` stub → this stone swaps in the real `ToEdn`; the re-export + helper-attr + cross-crate resolution are already proven).
- The derive-on-struct shape: the S1 test `struct_derive_emits_namespaced_tagged_record_with_optional_nested`.
