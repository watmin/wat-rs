# PURGARE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## What I swept

Read-only, `grep`/`find`/`wc`/`sed`/`git log` only. Every number below was re-derived this session against HEAD (branch `grok-rete`), not taken from the brief.

**Inventory** — matched the brief exactly: 306 files / 38,058 lines; `tests/rete/` 144 `.wat`, 100 `.rs`, 23 `.edn`, 19 `.wat.bad`; `src/rete/kernel/tests/` 20 `.rs`; 613 `#[test]` fns; 0 `rune:purgare`.

**Sampling rule** — full structural sweep (100% coverage) for: fixture-reach mechanisms (name-derivation, explicit-path, CLI, `.wat.bad` walk), `struct` field liveness (all 12 struct definitions in-target), `#[allow(dead_code)]`/`#[allow(unused_*)]` annotations, bool-closure-parameter constant-value check, and mod.rs central test-helper usage. For ordinary "helper fn never called" dead code I ran a full mechanical single-vs-multi-occurrence scan across all 100+20 `.rs` files (not a sample), then hand-verified every candidate it surfaced. I did **not** read every line of the 14,425-line `src/rete/kernel/tests/` bodies or all ~38k lines of `tests/rete/` prose/assertions — I read struct/fn/field definition sites and their usage sites, not full test bodies line-by-line, so a defect purely inside an assertion's logic (not a defined-but-unused *thing*) could be missed; that is out of scope for `purgare` anyway (that's `peragrare`'s / `complectens`'s territory).

**Deltas from the brief's own numbers:** the brief's "40 call sites" for `startup_beside(file!())` is high — I count **32** actual call sites in 7 files; the other 8 hits are `use wat::freeze::{…, startup_beside}` import lines and one doc comment, not calls. Everything else in the brief's mechanism list (62 files for `call_beside_value`, 5 for `call_beside`, 0 `rune:purgare`, 613 tests, 306/38058) re-derived exact.

## The fixture-liveness answer

**`.wat.bad` (19/19 — fully alive, doubly reached.)** Every one is (a) walked and driven through `startup_from_file` by `tests/lint/every_wat_bad_fixture_actually_fails.rs` (`ROOTS = ["tests","wat-scripts","docs"]`, recursive — this alone reaches all 19 regardless of naming), **and** (b) named explicitly by string literal in its own probe for a located-error assertion (`comm -12` between the on-disk `.wat.bad` list and the corpus-wide explicit-path-literal extraction returned all 19 — zero walk-only stragglers).

**`.edn` (23/23 — fully alive.)** The brief was right that a same-stem check finds zero — the `__variant` suffix defeats it — but every one of the 23 is named by an explicit string literal (`include_str!` or a `"…__variant.edn"` literal) inside its owning `.rs` file: 2+3+4+1+4+4+4+1 = 23, verified file-by-file (`probe_arc278_enum_variant_typo.rs`, `_D6_constraint_omission.rs`, `_field_span.rs`, `_rete_edn.rs`, `_D10_/_D11_then_field_types.rs`, `_nested_wall.rs`, `_match_arm_is_not_a_call.rs`). `datamancer.rete.edn` is the one non-`__variant` case, read via `Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/rete/datamancer.rete.edn")` in `probe_arc278_rete_edn.rs:19`.

**`.wat` (143/144 — one unreached.)** Union of: 69 files reached by co-location (`startup_beside`/`call_beside`/`call_beside_value(file!(…))`, each resolving exactly one sibling `.wat` — verified every stem has a file on disk), plus 76 distinct `.wat` filenames named by an explicit string literal (`startup_from_file`, `Command::new` against the compiled binary with a `CARGO_MANIFEST_DIR`-joined path, including shared fixtures like `probe_arc278_6b_ii_a_where_oracle_impure.wat`, read by two different probes). Two files using `Command::new` (`wat_scripts_grid_axes_live.rs`, `wat_scripts_grid_port_check.rs`) I ruled OUT of this count — their `grid_dir()` is `wat-scripts/perf/grid`, not `tests/rete`, so they drive a different corpus entirely (a new mechanism, not one of the 7, but out of target scope). Union = 143 of 144. The one gap: **`tests/rete/datamancer.src.wat`**.

## Finding

**`tests/rete/datamancer.src.wat` — a `.wat` file exercised by nothing in the corpus, not even a parse check.**

- Evidence of death: not in the 69-file co-location set; not among the 76 explicit-literal filenames (`grep -rn 'datamancer' tests/rete src/rete/kernel/tests` shows only `probe_arc278_rete_edn.rs`/`.wat` referencing the *output* file `datamancer.rete.edn`, never `datamancer.src.wat`); not a `.wat.bad`, so the walk-gate doesn't reach it either. And critically — the general "every `.wat` must load or declare why not" gate (`tests/lint/docs_wat_loads_or_declares_why_not.rs:7`) states outright *"It walks `wat-scripts/` **only**"* — `tests/rete/` is not in its walk. So this file is checked by **nothing at all**, automated or otherwise.
- The file itself is self-aware of this: `datamancer.src.wat:135` reads `;; Writes tests/rete/datamancer.rete.edn. Invoked only by the wat CLI, never by probes.`, and its header (line 6) gives the manual regen command: `cargo run --release --bin wat -- tests/rete/datamancer.src.wat`. It is the declared *source* for the checked-in `datamancer.rete.edn` artifact that `probe_arc278_rete_edn.rs` DOES consume — so the artifact is alive, but its generator is not connected to anything that would catch the artifact going stale relative to the source (an edit to `datamancer.src.wat`, or a semantics change in the compiler, could silently desync it from the checked-in `.edn` and nothing would notice).
- This doesn't cleanly fit any of the four rune categories (`public-api`/`trait-contract`/`future-fixture`/`safety-margin`) — it's closest to a fifth, undeclared shape: "manual regenerator for a checked-in golden, no drift check." Per the ward's own rule (zero runes exist in this target, so nothing here is *declared* intentional), this needs one of: (a) a `rune:purgare` explaining the manual-CLI-only intent, or (b) — better — a small gate that regenerates and diffs against the checked-in `.edn` so a future edit can't silently desync producer and artifact.
- Recommendation: leaf addition (not deletion) — either rune the file or wire a regen-diff check; removal cost if deleted outright would be a cascade (the checked-in `datamancer.rete.edn` and its consuming test would need a new source of truth).

## What I looked for and did not find

- **Structs never imported / dead types:** none. All 12 struct definitions in-target (`Shot`×4, `Dispatch`/`BranchPair`/`Measured`/`UniformAxis` in `where_tree_branch_differential.rs`, `Gather`, `AlphaTreeFixture`, `D2Reading`, `Surface`) had every field read at a print/assert site — no repeat of the `child_tax` shape already rowed as `4X1` by `excusare`.
- **Params always passed the same constant:** the one bool-closure pattern (`shot: |with_query: bool| -> Shot`, 4 copies) is always called as both `shot(false)` and `shot(true)` at every site. `Surface.needs_expressions` genuinely varies (3 true, 1 false) across the const array.
- **`#[allow(dead_code)]`/`#[allow(unused_*)]`:** only the one instance already rowed by `excusare` (`fanout_cost.rs:841`) — no new sites.
- **Orphan helper functions** (defined, never called): a full mechanical scan (not sampled) flagged ~100 candidates; every one resolved to either (a) a `#[test]` fn (harness-discovered, correctly appearing once — my first-pass script falsely flagged 6 in `probe_arc278_export.rs` because a `// rune:vocare(...)` comment line between `#[test]` and the fn signature broke my attribute-tracking; corrected and confirmed all 6 are real tests), or (b) `mod.rs`'s central `pub(super)`/private helpers, each independently confirmed used in 2–8 sibling files. Zero true orphans.
- Did not read every assertion body line-by-line across the ~38k lines — see sampling rule above; a logic bug inside a live assertion is outside `purgare`'s remit.

## **FINDINGS**
