# Arc 240 — runtime-rot remediation (consumer `.wat` drift + substrate gaps)

**Spawned 2026-05-27** from arc 239's STOP-and-report. Arc 239 (span-arity
*compile*-rot sweep) made the workspace test-build compile for the first time
since the `--lib`-only green-metric let it rot. The first full
`cargo test --release --workspace --no-fail-fast` then revealed **69 runtime
failures** — none caused by recent work (237.7a/length). Arc 239's own BRIEF
says STOP-and-report any failure that isn't span-arity / records-destructure;
this is that report, promoted to its own arc.

**Authority:** the workspace run captured at `/tmp/arc239_workspace_test.log`
(2026-05-27). All classifications below are from that ground truth, not the
breadcrumb's guesses (which under-counted: it named 5 suspects; the field is ~6
root causes / 69 tests).

## The ledger — 69 failures, ~6 root causes

| # | Root cause | Tests | Owning arc | Verdict |
|---|---|---|---|---|
| **A** | **Consumer `.wat` drift.** `(:wat::holon::Atom <non-holon>)` — `Atom` narrowed to `HolonAST→HolonAST` (arc 225); keyword/WatAST args must use `to-holon`. Plus 1-arg `:wat::core::HashMap` — the `(K,V)`-tuple-alias shorthand died when arc 215 minted the 2-type-arg constructor. In telemetry + telemetry-sqlite **production AND test** `.wat`. `WorkUnitLog.wat` is bundled into CLI startup → cascades into ~5 `wat_cli` failures. | 36 tel + 11 sqlite + ~5 cli-cascade | arcs 91/96 **CLOSED** | **FIX** (240.3) — pure drift on closed arcs. |
| **B** | `:wat::core::first`/`rest` have **no `List` arm** (`check.rs:12147` `infer_positional_accessor` → "tuple or Vec<T>"). Both arc220 failures are this one gap (the conj test calls `first` too). | 2 | arc 220 (closure #449 pending; substrate built) | **FIX** (240.1) — clean additive gap; finishes arc 220's intended first/rest/conj contract. |
| **C** | `:wat::holon::Bundle` doesn't **unfold the `:wat::holon::Holons` alias** (`= Vec<HolonAST>`, arc 033) in its param check — violates the "aliases resolve structurally at call sites" rule. | 1 | — (holon machinery) | **FIX** (240.1) — small substrate alias-unfold. |
| **D** | `probe_arc216_stone5c::probe_12`: `to-holon` of a HashMap is now a classifier-wrapped `Bind`, not a bare `Bundle` (arc 228/230) — so `Bundle/children` fails. **Substrate is MORE correct; the test asserts the old shape.** | 1 | arc 216 open, but broke from CLOSED 228/230 | **FIX test** (240.2) — substrate teaching us; don't lose good work. |
| **F** | `probe_lifeline_pipe_proof`: orphan-proof failed **1/100 trials** — the `spawn_lifelined` / Pidfd lifeline-pipe primitive (PDEATHSIG replacement, `src/fork.rs:154` "Arc 213 β") slipped its "100/100 zero-orphans, cannot race" guarantee once. | 1 | arc 213 **OPEN** (#373 pidfd cascade) | **DEFER → arc 213** (process-management-grade; user reclassified 2026-05-27 from "user review"). Marked on arc 213 KNOWN-BROKEN. Investigate residual-race-vs-env-flake when #373 lands; do NOT fold into routine gates (spawns processes). |
| **G** | 3× **ambient-stdio time-limit leaks** (`readln-echo` 15s timeout, `println`, `assert-stderr`) — stdio-trio thread-context machinery. (+2 `struct-to-form` likely A-class; confirm during 240.3.) | 3 (+2?) | arc 170 **IN-FLIGHT** (#296 stdio-trio) | **DEFER → arc 170** — "addressed when we unwind to it" (user). |

### The DEFER set — broken because their arc's dependencies are actively under construction (user rule)

- **lru / holon-lru `HolonKey.wat`** (3 tests: roundtrip / distinguishes / structural-equal). Immediate error is the same `Atom` drift as A — BUT these files live in the **wat-lru crate**, call `:wat::lru::LocalCache::*`, and the lru wat-tests are **arc 119's in-flight `#208` consumer-sweep** while **arc 130 (`#226`) is actively reshaping the LocalCache substrate**. Fixing them now would do arc 119/130's in-flight work piecemeal and risk re-breaking on the reshape. → **DEFER; marked on arc 119 + arc 130.**
- **`wat_cli` fork/sigterm/exit-code residual** (`sigterm_to_cli_cascades`, `check_mode_exits_zero`, `missing_user_main_rejected` exit-code deltas, etc., *after* the A-cascade clears). These exercise the spawn/fork/exit machinery **arc 170 is actively deleting/reshaping** (#309 wat-cli Stone B, #310 spawn.rs deletion). → **DEFER; marked on arc 170.** (Re-run after 240.3 to separate cascade-cleared from genuinely-170.)
- **ambient-stdio (G)** → arc 170 (#296). Per user.

## Partition summary

- **FIX (this arc):** A (240.3 consumer `.wat` sweep) · B + C (240.1 substrate gaps) · D (240.2 stale-test).
- **DEFER + marked:** lru/holon-lru → arc 119 + 130 · wat_cli-fork + ambient-stdio → arc 170.
- **USER REVIEW:** lifeline 1/100 flaky → arc 213.

## Stones

- **240.1** — substrate gaps: `first`/`rest` `List` arm (B) + `Bundle` `Holons` alias-unfold (C). `src/check.rs` (+ `src/runtime.rs` for first/rest eval if needed). Clears wat_arc220_list (2) + wat_bundle_capacity (1).
- **240.2** — stale-test: `probe_arc216_stone5c::probe_12` → classifier-wrap holon shape (D). Clears 1.
- **240.3** — consumer `.wat` drift sweep (A): telemetry + telemetry-sqlite production + test `.wat`. **TWO-BRIDGE recipe** (per-site by the arg's type, which the substrate error names in `got:`; prove on `WorkUnitLog.wat` first, FM 2-bis):
  - `(:wat::holon::Atom <value>)` where value is a keyword/String/i64/etc. → `(:wat::holon::to-holon <value>)` (∀T-over-values bridge).
  - `(:wat::holon::Atom <watast>)` where the arg is `:wat::WatAST` (e.g. `data-holon` in WorkUnitLog.wat:100, or `(:wat::core::quote ...)`) → `(:wat::holon::from-wat <watast>)` (the WatAST→HolonAST bridge; arc-225 rename of `from-watast`, check.rs:16259). **A uniform Atom→to-holon sweep would break the WatAST sites — classify each by `got:`.**
  - `(:wat::core::HashMap :Tag)` (1-arg, `Tag = (HolonAST,HolonAST)`) → `(:wat::core::HashMap :wat::holon::HolonAST :wat::holon::HolonAST)` (arc-215 2-type-arg constructor; expand the `(K,V)` alias).
  - Clears ~47 + the wat_cli A-cascade. ~28 Atom sites (telemetry WorkUnit/WorkUnitLog prod+test, sqlite hashmap-field/edn-newtypes) + ~5 HashMap sites.
- **240.4** — INSCRIPTION + reconcile the DEFER ledger + close. **Verification is TARGETED, not the full suite.** Per user direction 2026-05-27: the routine gate is `cargo test --lib` + `cargo build --tests --workspace` (compile-only); **never routinely run `cargo test --workspace`** — it leaks processes (ambient-stdio/fork/lifeline) that arc 170 fixes. Confirm the FIX set via the specific non-process-spawning binaries (`wat_arc220_list`, `wat_bundle_capacity`, `probe_arc216_stone5c`, telemetry deftests). The wat-cli A-cascade clearing is asserted-by-construction (WorkUnitLog.wat fixed → no longer in startup); do NOT run the leaky wat-cli fork suite to "confirm." See `feedback_green_gate_lib_and_build`.

## Discipline notes

- DEFER markers are NOT FM-11 deferral-language-in-an-INSCRIPTION; they are honest in-flight-arc attributions per user direction 2026-05-27: *"defer any FIX calls who are pending arc closures — if they are broken because we are actively building their arc's dependencies then leave them (mark those arcs as having broken tests so we know)."*
- holon-rs frozen (STOP-5). Write only in wat-rs. Sonnet writes substrate; orchestrator briefs/scores/commits.
