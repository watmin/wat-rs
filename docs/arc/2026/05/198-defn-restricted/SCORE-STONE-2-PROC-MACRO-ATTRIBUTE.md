# Arc 198 Slice 2 Stone 2 SCORE — `#[restricted_to(...)]` proc-macro attribute

**BRIEF:** `BRIEF-STONE-2-PROC-MACRO-ATTRIBUTE.md`
**EXPECTATIONS:** `EXPECTATIONS-STONE-2-PROC-MACRO-ATTRIBUTE.md`
**Predecessor:** Stone 1 — `RestrictionEntry` struct + `inventory::collect!` + setup-time iteration shipped.

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `#[restricted_to(...)]` proc-macro attribute defined in `crates/wat-macros/` | **YES** | `crates/wat-macros/src/lib.rs` adds `pub(crate) struct RestrictedToAttr { wat_name: LitStr, prefixes: Vec<LitStr> }` + `impl Parse for RestrictedToAttr` (positional variadic `LitStr` via `Punctuated::parse_terminated`) + `#[proc_macro_attribute] pub fn restricted_to(attr, item) -> TokenStream`. Grep: `grep -n "restricted_to\|RestrictedToAttr" crates/wat-macros/src/lib.rs` shows the registration site (`#[proc_macro_attribute]`), the parser (`impl Parse for RestrictedToAttr`), and the parse error message. |
| B | Codegen emits `inventory::submit!` block with `RestrictionEntry { wat_name, prefixes }` | **YES** | Same file, codegen `quote!` block: `::inventory::submit! { ::wat::restriction_entry::RestrictionEntry { wat_name: #wat_name, prefixes: &[ #(#prefix_lits),* ] } }`. Generated code lands at module scope (sibling to the annotated fn) — `inventory::submit!` requires module scope per its docs; the macro passes the annotated item through unchanged and appends the submit. Path syntax is absolute (`::wat::...`, `::inventory::...`) so consumer-crate `use` graphs can't shadow the resolution. |
| C | Variadic prefix arg parsing works (1+ and 2+ prefixes both succeed) | **YES** | Tests 1 (single prefix) and 2 (three prefixes) both green. `Punctuated::parse_terminated` accepts any non-zero count of comma-separated `LitStr`s with optional trailing comma. |
| D | 3 new tests pass — single-prefix + multi-prefix + exact-FQDN | **YES** | `tests/wat_arc198_slice2_stone_2_attribute.rs` declares three probe fns (`probe_single`, `probe_multi`, `probe_exact_fqdn`), each annotated with `#[restricted_to(...)]`. Three tests run `startup_from_source` against a minimal valid wat source and assert `frozen.symbols.defined_value_restrictions.get(<probe wat_name>)` returns the expected prefix vec. Run: `cargo test --release -p wat --test wat_arc198_slice2_stone_2_attribute` → `test result: ok. 3 passed; 0 failed`. |
| E | Workspace test failure count ≤ baseline (3 stable + lifeline flake variance) | **YES** | `cargo test --release --workspace --no-fail-fast` → `error: 4 targets failed: -p wat --test probe_lifeline_pipe_proof, -p wat --test test, -p wat --test wat_arc170_program_contracts, -p wat-cli --test wat_cli`. Individual failures: `lifeline_pipe_zero_orphans_across_100_trials` (known flake, 2/100 trials this run — within rotation band), `deftest_wat_tests_tmp_totally_bogus` (pre-existing), `t6_spawn_process_factory_with_capture_round_trips` (pre-existing), `startup_error_bubbles_up_as_exit_3` (pre-existing). 3 stable + 1 flake = baseline + 0 new failures. **Zero new failures introduced.** |

**5/5 PASS.**

## Honest deltas

### Sub-decision (a) positional vs (b) named — chose (a)

**Chose (a) positional variadic.** First positional arg is the wat FQDN; remaining args are the variadic prefix whitelist. Matches the `#[derive(...)]` / `#[cfg(any(unix, target_os = "macos"))]` precedent in stable Rust. Rationale:

1. The prefix list is genuinely variadic — `(b) named` would force a bracketed-array syntax `from = [":wat::", ":my::"]` that's more typing for callers and more parser code on our side (`syn::bracketed!` + nested `Punctuated`).
2. The wat-side `(:wat::core::def-restricted :name [prefixes] body)` form from slice 1 is itself positional (name first, prefix vec second). Mirroring positionally makes the Rust↔wat surfaces visually symmetric for readers comparing the two declaration sites.
3. Future extension (e.g. an optional `reason = "..."` doc string) can still graft on as a named-only param after the variadic positionals — `syn::parse_terminated` over a Punctuated<NameOrPositional, Comma>` is the well-trodden upgrade path if needed. The decision isn't a one-way door.

(b) named was not surfaced as a STOP-trigger — (a) parsed cleanly and read naturally at the call site.

### Codegen path syntax — `::wat::restriction_entry::RestrictionEntry`

Used the **absolute crate path** `::wat::restriction_entry::RestrictionEntry` (and `::inventory::submit!`) in the emitted code. The generated tokens land in the CONSUMER crate's module scope (the test crate, the eventual `src/runtime.rs` for Stone 3, etc.), and absolute paths avoid every form of `use`-graph shadowing or rename collision. The `#[wat_dispatch]` codegen (`crates/wat-macros/src/codegen.rs`) uses the same convention (`::wat::runtime::*`, `::wat::rust_deps::*`) — Stone 2 mirrors the existing house style.

The `wat` crate already re-exports `restriction_entry` as a public module (`src/lib.rs:82`), so `::wat::restriction_entry::RestrictionEntry` is the canonical path — no need to add a top-level re-export. `wat_macros` is a regular `[dependencies]` member of the wat crate (Cargo.toml:58), so consumer crates that depend on `wat` (or anything that re-exports `wat_macros::restricted_to`) can use the macro without adding a separate `wat-macros` dep — same access pattern as `wat_macros::wat_dispatch` used by `tests/wat_dispatch_*` test files.

### `RestrictionEntry` field types — `&'static str` literal-friendly (verified)

Stone 1 declared `pub wat_name: &'static str` + `pub prefixes: &'static [&'static str]`. String literals in proc-macro attribute parsing are `syn::LitStr`, which `quote!` renders back to the source as a `&'static str` literal. The slice literal `&[#(#prefix_lits),*]` similarly renders to a `&'static [&'static str]`. No explicit lifetime annotation or `Box::leak` machinery needed — string + slice literals satisfy `'static` naturally. Same observation as Stone 1's SCORE.

### Hygiene for static names

No unique hygienic identifier needed. `inventory::submit! { ... }` internally generates an anonymous `static` item with its own unique mangled name; the macro accepts multiple submits from the same module (the test file has three probe fns, each emitting its own submit, and they coexist without collision). Stone 1's manual `inventory::submit!` already proved this — Stone 2 just generates the same call mechanically.

The annotated fn itself is unchanged — the macro emits `#item_ts` first (the original fn tokens) and the submit second, so the consumer's `pub` / `pub(crate)` / module placement is preserved verbatim.

### Pass-through robustness — `TokenStream` not `ItemFn`

The macro takes `item: TokenStream` and passes it through as `TokenStream2` rather than `parse_macro_input!(item as ItemFn)`. This is deliberate — `#[restricted_to]` doesn't care about the annotated item's signature (it doesn't inspect args, generics, or return types). Accepting the raw token stream means any module-scope Rust item (fn / pub fn / pub(crate) fn / generic fn / async fn / ...) survives the macro without per-shape parser code. If a future shape needs signature inspection (it shouldn't — restriction is about the binding's FQDN at the wat layer, not the Rust shape), we can tighten the parse then; preserving the loose contract today keeps the macro from fighting evolving callers.

### Parser disambiguation — `<LitStr as Parse>::parse`

First implementation hit `error[E0308]: expected fn pointer, found fn item` on `LitStr::parse` — `LitStr` has a `Parse::parse` method (which is what `parse_terminated` wants) but `LitStr` itself also has other methods named `parse` from different traits, and Rust's resolution picked one with the wrong signature. Wrote `<LitStr as Parse>::parse` to disambiguate explicitly. One-line fix, one-line cause; no design implication.

### Workspace test count vs baseline

| Target | Baseline (Stone 1 end) | Post-Stone-2 | Delta |
|---|---|---|---|
| `wat::wat_arc198_slice2_stone_2_attribute` (NEW) | (did not exist) | **3 passed / 0 failed** | +3 passes |
| `wat::wat_arc198_slice2_stone_1_inventory_wiring` | 1 pass | 1 pass | unchanged |
| `wat::probe_lifeline_pipe_proof` | 0-1 fail (flake band) | 1 fail this run (flake hit) | within flake band |
| `wat::test` (deftest_wat_tests_tmp_totally_bogus) | 176 pass / 1 fail | 176 pass / 1 fail | unchanged |
| `wat::wat_arc170_program_contracts` (t6) | 23 pass / 1 fail | 23 pass / 1 fail | unchanged |
| `wat-cli::wat_cli` (startup_error_bubbles_up_as_exit_3) | 14 pass / 1 fail | 14 pass / 1 fail | unchanged |
| `wat-macros` internal | 50 passed | 50 passed | unchanged |
| Every other target | passes | passes | unchanged |

**Net: +3 new passes; 0 new failures; lifeline flake in rotation band.** Workspace failures = 3 stable + 1 flake = within baseline + flake variance per the BRIEF's "Row E" definition.

### Substrate-discovery surprises

**One.** Predicted 0-3 in EXPECTATIONS; actual: 1 (the `<LitStr as Parse>::parse` disambiguation noted above). Cost: one Edit, one rebuild. The surprise was a Rust trait-resolution quirk rather than a substrate-design issue — `LitStr::parse` is overloaded across traits and the compiler can't pick without explicit qualification. No design impact; no scope change.

Everything else composed cleanly:
- `Punctuated<LitStr, Comma>` parsed the variadic positional list in one line, with trailing-comma support free.
- `quote!` emitted absolute crate paths without escaping or interpolation gymnastics.
- The generated `inventory::submit!` landed at module scope automatically because the macro emits `#item_ts` (original fn) + sibling submit — both at the original annotation site's scope.
- Stone 1's `'static` field types absorbed the literal-derived tokens without any explicit lifetime ceremony.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 90 min | ~40 min |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | ≤ baseline (3 stable + flake variance) | 3 stable + 1 flake (in band) |
| New test count | 3 | 3 |
| Sub-decision chosen | (a) positional OR (b) named | **(a) positional** — matches `#[derive(...)]` and slice 1's wat-side form |
| Codegen path syntax | `::wat::RestrictionEntry` OR `wat::RestrictionEntry` | `::wat::restriction_entry::RestrictionEntry` (absolute; mirrors `#[wat_dispatch]` house style) |
| Substrate-discovery surprises | 0-3 | 1 (`<LitStr as Parse>::parse` disambiguation) |
| Mode | Additive (proc-macro attribute + 3 verification tests) | Additive |

## STOP triggers encountered

**None reached.**

- "Proc-macro infrastructure doesn't extend cleanly to variadic string args" — `Punctuated<LitStr, Comma>::parse_terminated` handled it in one line.
- "Codegen can't reference `wat::RestrictionEntry` from the consumer's generated code (path resolution issue)" — `::wat::restriction_entry::RestrictionEntry` (absolute path) resolves uniformly from any consumer module; the existing `pub mod restriction_entry;` in `src/lib.rs` is the canonical surface.
- "Sub-decision (a) positional runs into clear blocker" — no blocker; (a) read naturally at call sites and parsed in fewer lines than (b) would have.
- "Migration breaks existing tests" — purely additive; Stone 1's test still green, every other workspace target unchanged from baseline.
- ">3 unexpected substrate-finding surfaces" — 1 surface (trait-resolution disambiguation), well under threshold.

## What this enables

After Stone 2 ships:

- **Stone 3** applies `#[restricted_to(...)]` to the two real substrate fns `eval_kernel_thread_join_result` and `eval_kernel_process_join_result` (currently policed by arc 170 Stone B's hard-coded substrate-namespace exemption in `validate_def_restricted_caller_namespace`). The annotations carry the same prefix whitelist Stone B's ad-hoc rule encodes, but through the generic channel Stones 1+2 built.
- **Stone 4** deletes Stone B's ad-hoc walker rule + orphaned `JoinResultUserNamespace` `CheckError` variant + Stone B's 4 caller-migration tests now that the generic mechanism subsumes the ad-hoc one.

The substrate's restriction-declaration channel becomes uniform: wat source declarations and Rust substrate declarations land in the same `defined_value_restrictions` map via the same walker, regardless of where the declaration originated.

## Files touched

- `crates/wat-macros/src/lib.rs` — added `RestrictedToAttr` struct + `impl Parse` + `#[proc_macro_attribute] pub fn restricted_to` (~70 LOC including doc comment block and module-header rationale). Module-header `─── #[restricted_to(...)]  arc 198 slice 2 Stone 2 ───` separator placed between `wat_dispatch` and `wat::main!` for navigation parity with existing per-feature sections.
- `tests/wat_arc198_slice2_stone_2_attribute.rs` — NEW. Three probe fns (`probe_single`, `probe_multi`, `probe_exact_fqdn`) annotated with `#[restricted_to(...)]`, three tests asserting each entry lands in `defined_value_restrictions` after `startup_from_source`. Probe fn bodies are `unreachable!()` — the attribute's `inventory::submit!` emission happens at compile time / link time, independent of whether the fn is ever called.
- `docs/arc/2026/05/198-defn-restricted/SCORE-STONE-2-PROC-MACRO-ATTRIBUTE.md` — this file (NEW).
