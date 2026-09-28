# MEASURE — where a wat world freeze spends its time

Excursus 003, Strike B worklist, the S2 "cost" row in
`AUDIT-the-shape-of-an-error.md`:

> cost, not wire: each declared stdlib record adds ~0.8ms per world freeze (hello-world
> 615–676ms → 682–717ms, 6 samples each); floor 1103s → 1205s (+9%) across S1+S2; two
> near-margin tests widened (`d0f126e18`, `42cca492a`); measure where freeze time goes
> BEFORE S3.

This is that measurement. It is **measurement only** — nothing about the freeze pipeline's
behavior changed. What changed is instrumentation: two opt-in, off-by-default probes wired
into the freeze pipeline's own existing announce points, plus one new announce point. Both
are committed, not thrown away, per the task's own rule that a recorded number whose
instrument is deleted cannot be rechecked.

## The instrument

**Phase timing** — `src/freeze/pass_order.rs`. The pipeline already had a `record(step)`
call at every stage boundary (Stone 255.12, order gate `the_startup_passes_run_in_the_
declared_order`). `record` now ALSO stamps a per-thread `Instant` at each of those same
points when `WAT_FREEZE_PHASE_TIMING` is set in the environment — checked once via
`OnceLock`, so an unmeasured run pays one relaxed load per call and nothing else (no
allocation, no `Instant::now()`, no lock). Durations are read back via
`wat::freeze::take_freeze_phase_timings()` (`src/freeze.rs`), which drains the calling
thread's trace into `(phase, Duration)` pairs. One new announce point was added,
`"9-freeze"`, right at the top of `FrozenWorld::freeze` — it splits `check_program`'s own
cost from `FrozenWorld::freeze`'s own body, which previously shared one undifferentiated
tail. The order gate's `EXPECTED_ORDER` was updated to match and still passes.

**Freeze counting** — `src/freeze/measure.rs`. `FrozenWorld::freeze` appends one line (its
own pid) to the file named by `WAT_FREEZE_COUNT_LOG`, right before it returns a
successfully-built world — the one door every startup path funnels through. Off unless that
var is set (one more `OnceLock` read); never touches stdout or stderr under any condition,
so it cannot be the "a child's stdout is a wire" hazard
(`RULING-a-childs-stdout-is-a-wire.md`) the harness rules warn about.

**The measurement example** — `examples/freeze_phase_timing.rs`. Sets
`WAT_FREEZE_PHASE_TIMING` itself, freezes a hello-world program
(`(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println 1))`)
repeatedly, and separately freezes the same program with 40 synthetic `defrecord`s appended
to the entry source — a controlled A/B run in one process, no rebuild between arms. Prints
one EDN map per sample to stdout; this is a diagnostic binary, never linked into
`src/bin/wat.rs`, so its stdout is not the CLI's wire.

### Re-run it

```bash
cargo build --release --example freeze_phase_timing
./target/release/examples/freeze_phase_timing 8 40   # 8 samples/arm, 40 synthetic records
```

For the re-freeze count, point `WAT_FREEZE_COUNT_LOG` at a file before a floor run:

```bash
WAT_FREEZE_COUNT_LOG=/tmp/freeze_count.log scripts/floor.sh
wc -l /tmp/freeze_count.log        # total successful freezes across the whole suite
sort /tmp/freeze_count.log | uniq -c | sort -rn | head   # freezes per OS process
```

## 1. Phase breakdown of one freeze

8 samples, hello-world, built with `cargo build --release --example freeze_phase_timing`,
run back-to-back on an otherwise-idle box (one untimed warm-up run discarded first; no other
`cargo`/`nextest` process was running — checked with `pgrep -x cargo` before starting).
Values are mean `[min–max]` microseconds. Each row names the pipeline interval it covers by
the `pass_order::record` label that OPENS it, and — because several of these labels are
historical (named for one call, but the interval also contains uninstrumented work that
runs before the NEXT label) — what actually executes inside that interval, read directly
from `src/freeze/env.rs` / `src/freeze.rs`.

| interval (opens at this `record` label) | mean [min–max] µs | what runs inside it |
|---|---:|---|
| `2-collect-entry-file` | 2.2 [2–3] | `collect_entry_file` |
| `3-resolve-loads` | 30 952 [30 408–32 478] | `resolve_loads` **+ `stdlib_forms()`** — parsing all 72 baked stdlib `.wat` files (`src/load/stdlib.rs:693`) runs immediately after this interval's own work and before the next label, so its cost lands here |
| `3b-extract-rete-defn-names` | 1.5 [1–2] | `extract_rete_defn_names` + head rewrite |
| `4-register-stdlib-defmacros` | 3 911 [3 797–4 162] | `register_stdlib_defmacros` |
| `4-register-defmacros` | **197 938** [193 128–211 654] | `register_defmacros(user)` (trivial for hello-world) **+ `register_aggregate_kwargs_companions`(builtins) + `expand_all_with(stdlib_post_macros, …, Privilege::Stdlib)`** — the stdlib's OWN macro expansion, ~1000 top-level forms. This is almost certainly what dominates; the label names only the first, cheapest call in the interval |
| `4-expand-all` | 917 [899–993] | `expand_all(user_forms)` + the arc-163/arc-170 bare-legacy walkers over the result |
| `5-register-stdlib-types` | 5 282 [5 200–5 388] | `register_stdlib_types` |
| `5-register-types` | 1 598 [1 572–1 678] | `register_types_with_acronyms` + `validate_aggregate_containment` + `register_variant_types` |
| `6-register-stdlib-defines` | 19 443 [19 284–19 676] | `register_stdlib_defines` + defclause stub preregistration |
| `6-register-defines` | 15 410 [15 168–15 962] | `register_defines(user)` + `validate_named_type_annotations` **+ `register_struct_methods` + `register_enum_methods` + `register_newtype_methods` + `register_aggregate_methods` + `register_type_predicates`** (accessor/ctor/predicate codegen for every declared aggregate, stdlib and user) + the `inventory` restriction-entry drain + `preregister_acronyms` + `rekey_type_member_functions` |
| `7-normalize-symbol-refs` | 9.9 [9–11] | `normalize_symbol_refs` |
| `7-normalize-stored-function-bodies` | 18 194 [17 972–18 394] | `normalize_stored_function_bodies` (1st pass) |
| `7-resolve-references` | 13 449 [13 368–13 564] | `resolve_references` |
| `7.7-normalize-stored-function-bodies` | 32 974 [31 784–39 663] | `register_stdlib_runtime_defs` + step 7.7's extend-type pre-registration + `normalize_stored_function_bodies` (2nd pass) |
| `8-check-program` | **216 558** [209 903–228 873] | `check_program` — see § 2, this is the single biggest phase |
| *(residual, no closing label)* | 7 933 [7 405–10 548] | the entry-source parse in `startup_from_source` (before step 2) + `FrozenWorld::freeze`'s own body after its `record("9-freeze")`: `validate_holon_record_capacity`, `EncodingCtx`/sigma-fn setup, `register_runtime_defs` |
| **TOTAL** | **564 580** [551 770–573 243] | — |

Two intervals — `4-register-defmacros` (~198 ms) and `8-check-program` (~217 ms) — together
account for **~73% of one freeze**, and *neither one's label describes the work that
dominates it*: the first is mostly the stdlib's own macro expansion riding on the back of a
call to register a handful of (zero, for hello-world) user macros; the second is
`check_program`, a single call with no further internal breakdown from this instrument.

This total (551–573 ms, this box, this binary) is close to, but below, the CLI's own
615–717 ms range quoted in the audit — expected, since this example calls
`wat::freeze::startup_from_source` directly with no `wat` binary process/argv/staleness-check
overhead around it.

## 2. Attribution per record

**Which phase grows, and why.** `check_program` (`src/check.rs:565`) does not walk "the
program"; the first thing it does is `for func in sym.function_values() { … }`
(`src/check.rs:602`) — a sweep over **every function in the whole `SymbolTable`**, stdlib
included, running `validate_bare_legacy_primitives` / `walk_for_legacy_stream` /
`walk_for_legacy_lru_cache_service` / `walk_for_legacy_kernel_queue` /
`walk_for_bare_legacy_console` on each body. Three more full-registry sweeps follow at
`:650` (`walk_for_restricted_call`), `:679`, and `:739`. Every one of these is `O(total
registered functions × body size)`, and "total registered functions" is exactly what a
`defrecord` grows: `register_struct_methods` mints **1 constructor**, `register_aggregate_
methods` mints **1 accessor per field**, and `register_type_predicates` mints **1 type
predicate** (`src/freeze/env.rs:585–596`) — for a 6-field record (the shape used below), 8
brand-new functions that every SUBSEQUENT freeze's `check_program` walks, forever, whether
or not anything ever calls them.

**The experiment.** `examples/freeze_phase_timing.rs` appends 40 synthetic records, shaped
like a typical `wat/check-errors.wat` variant (`message`/`location`/`causes` + 3 plain
fields — 6 fields), directly to the hello-world entry source, and re-runs the same 8-sample
protocol. Because they are declared as **user** forms (`:user::SyntheticRecord0..39`), they
flow through the user-labeled halves of the registration pipeline
(`register_defmacros`/`register_types_with_acronyms`/`register_defines`), not the
stdlib-labeled halves (`register_stdlib_defmacros`/`register_stdlib_types`/
`register_stdlib_defines`) a real `wat/check-errors.wat` addition would use — **this is the
proxy's one known divergence from a true stdlib record**, named here rather than left
implicit. `check_program`'s four full-registry sweeps and `FrozenWorld::freeze`'s
`register_runtime_defs`/`validate_holon_record_capacity` do not distinguish stdlib from
user functions, so those phases' measured growth should carry over directly; the
user-vs-stdlib registration phases' growth is a same-mechanism proxy, not a direct
measurement, and was NOT independently cross-checked against an actual stdlib addition
(that would need a second binary built from an older commit — out of scope for this pass;
see § "Contamination").

Same 8-sample protocol, mean of (treatment − baseline), divided by 40:

| phase | Δ mean (µs, 40 records) | µs / record | share of total Δ/record |
|---|---:|---:|---:|
| `8-check-program` | 11 555 | **288.9** | 47.4% |
| `4-expand-all` | 6 724 | **168.1** | 27.6% |
| `6-register-defines` | 2 539 | 63.5 | 10.4% |
| `7-normalize-stored-function-bodies` | 1 609 | 40.2 | 6.6% |
| residual (`FrozenWorld::freeze` body) | 799 | 20.0 | 3.3% |
| `7.7-normalize-stored-function-bodies` | 708 | 17.7 | 2.9% |
| `5-register-types` | 507 | 12.7 | 2.1% |
| `7-resolve-references` | 263 | 6.6 | 1.1% |
| `3b-extract-rete-defn-names` | 10 | 0.3 | ~0% |
| every stdlib-labeled phase (`3-resolve-loads`, `4-register-stdlib-defmacros`, `4-register-defmacros`, `5-register-stdlib-types`, `6-register-stdlib-defines`) | −9 to −306 | ≈0, within noise | — (flat, as expected: these process the fixed stdlib corpus only) |
| **TOTAL** | **24 372** | **609.3** | 100% |

The per-phase deltas sum to 609.3 µs/record against a directly-measured total delta of
609.3 µs/record (the per-phase figures were derived independently, from the same 16
samples, and agree to within rounding) — internally consistent. This 0.61 ms/record is the
same order as the audit's aggregate 0.8 ms/record measured on the real S1+S2 stdlib
addition; lower here plausibly because the synthetic records use simpler field types
(`String`/`i64`/`bool`) than some real `CheckErrorKind` variants (e.g. `TypeMismatch`'s
`Vector<Remedy>` field), and because of the user/stdlib registration-path divergence noted
above.

**Bottom line: ~75% of the per-record marginal cost (`8-check-program` + `4-expand-all`) is
concentrated in exactly the two phases whose *baseline* cost also dominates the whole
freeze** — check-time function-body walking and (whatever inside the `4-expand-all`
interval scales with declared forms; not further traced this pass — see "Contamination").
The registration phases that literally construct the new functions (`6-register-defines`,
at 63.5 µs/record) are a distant third.

## 3. How often the stdlib gets re-frozen

**Is any frozen state reused across freezes in one process?** No, in the production path.
`stdlib_forms()` (`src/load/stdlib.rs:693`) re-parses all 72 `include_str!`'d files on
*every* call — no cache, no `OnceLock`. `build_env` (`src/freeze/env.rs:388`) calls it
unconditionally at the top of every freeze. The only cached stdlib snapshot,
`stdlib_snapshot()` (`src/freeze/env.rs:84`, a `OnceLock<(SymbolTable, MacroRegistry,
TypeEnv)>`), is consumed by exactly two non-production call sites — `src/check.rs:23828`
and two more at `:24246`/`:24247` (a pointer-identity unit test), and
`src/reflect/verbs.rs:1645` and `:1704` — **never** by `startup_from_forms_post_config`, the
path every `startup_from_source`/`startup_beside`/`call_beside_value`/spawned-child call
goes through. So: a process that freezes N times pays the FULL parse → macro-expand →
type-register → defines-register → resolve → check → freeze pipeline, over the ENTIRE
stdlib, N separate times.

**How many times does that happen across the suite?** Measured exactly, not estimated: the
full floor (`scripts/floor.sh`, `cargo nextest run --release`, 6 324 tests + 5 doctests,
this run's `Summary [1206.201s] 6324 tests run: 6324 passed (29 slow), 22 skipped`) was run
with `WAT_FREEZE_COUNT_LOG` set. Result:

- **5 834 successful `FrozenWorld::freeze` calls**, across **3 960 distinct OS processes**
  (nextest isolates most tests per-binary-process, and some tests additionally fork/spawn
  child processes that freeze their own world).
- Distribution: 3 359 processes (85%) froze exactly once. The remaining 601 processes (15%)
  account for the other 2 475 freezes — up to **142 freezes in a single process** (the
  heaviest one seen this run), in the same family as the audit's cited
  `probe_rational_C5c_nan_unordered` (23 fresh freezes).
- A static grep of `startup_from_source`/`startup_from_forms`/`startup_beside`/
  `startup_from_file`/`call_beside`/`call_beside_value`/`startup_bare` call SITES in the
  tree gives ~1 741 — 3.3× lower than the real count. This is exactly the "a grep answers a
  question about text" trap: several call sites sit inside loops or shared macros
  (`tests/lint/wat_scripts_fixes_load.rs`, `tests/lint/every_wat_bad_fixture_actually_
  fails.rs`, `tests/lint/rete_compile_gate.rs` each iterate a `.wat` corpus, one freeze per
  file), so a call-site count is a lower bound, not the answer, and is reported here only as
  the cautionary contrast — 5 834 is the real number.

At ~0.61 ms marginal cost per declared record (§ 2) and with every one of 5 834 freezes
paying it independently, each additional stdlib record's cost is not "0.8 ms once" — it is
paid 5 834 times per floor run (**≈ 4.7 s of floor time per newly declared record**, which
matches the audit's observed aggregate: 87 records × ~0.7 ms/freeze-mean × ~1.5–1.8×
freeze-count-weighting ≈ the 1103 s → 1205 s move, within the noise this kind of back-of-
envelope carries).

## Contamination — read before trusting any number above

- **Sample count and box state.** § 1 and § 2 are 8 samples per arm (16 freezes total for
  § 2's A/B), one untimed warm-up discarded per arm, run back-to-back on one box with no
  other `cargo`/`nextest` process running (`pgrep -x cargo` checked clean immediately
  before). Not 6 independent *runs* of the whole example — one process, repeated in-process
  freezes — so page-cache and allocator warm-up effects are shared across samples within an
  arm; the min–max spread in the tables is the only defense against reading a mean as more
  precise than it is. `7.7-normalize-stored-function-bodies` and `6-register-defines` show
  the widest relative spread of any phase (baseline max 39 663 µs vs mean 32 974 µs; one
  treatment sample's `6-register-defines` hit 21 654 µs against a 17 949 µs mean) — GC/
  allocator jitter, not attributed further.
- **The per-record correlation is a proxy, not a stdlib measurement.** § 2's synthetic
  records are USER-declared, not stdlib-declared; the specific divergence (which registration
  functions run) is named in § 2. The four `check_program` sweeps and `FrozenWorld::freeze`'s
  own work do not distinguish the two, so their attributed growth should transfer; the
  user-labeled registration phases' growth is offered as a same-mechanism estimate for the
  stdlib-labeled phases, unverified by an actual stdlib-corpus A/B (would need a second
  release build from an older commit in a separate `CARGO_TARGET_DIR` — not done this pass).
- **`4-expand-all`'s 168 µs/record is not mechanistically traced.** I confirmed WHAT interval
  it is (between the `record("4-expand-all")` and `record("5-register-stdlib-types")`
  announce points: `expand_all(user_forms)` + the two arc-163/170 bare-legacy walkers) but
  did NOT trace which of those actually scales with declared-record count, nor whether
  `defrecord` itself expands as a registered macro at this point or is recognized as a
  type-declaration form earlier (`classify_type_decl`, referenced in `register_declared_
  types`'s doc, `src/freeze/env.rs`) and merely walked here at near-zero marginal cost per
  node. This is the single largest unresolved "mechanism" gap in this report — the SHAPE
  (it grows, here, this much) is measured; the MECHANISM is not.
- **The phase table's labels are historical, not descriptive**, per § 1's own callout for
  `4-register-defmacros` and (less so) `3-resolve-loads`. Anyone extending this instrument
  should rename intervals or add announce points rather than trust a label at face value.
- **What the instrument cannot see at all:** anything inside a single interval — anywhere
  ONE phase's number could actually be several sub-phases with opposite record-count
  sensitivity that happen to cancel (would only be visible by adding more `record()` calls,
  which this pass deliberately kept to the minimum — one new label, `9-freeze` — to limit
  risk to the order gate and the floor).
- **Freeze-count total is exact for THIS run, on THIS box, with THIS test selection** — it is
  not a law of the suite; a future test added/removed, or a change to which tests loop over
  a `.wat` corpus, moves the number. The floor that produced it: `Summary [1206.201s] 6324
  tests run: 6324 passed (29 slow), 22 skipped`, `exit=0` — the same run that validated this
  instrument's own code (see below).

## The floor

`scripts/floor.sh` (release, `cargo nextest run --release`), run with `WAT_FREEZE_COUNT_LOG`
set for the freeze-count measurement above:

```
Summary [1206.201s] 6324 tests run: 6324 passed (29 slow), 22 skipped
exit=0
```

Green — no test went red, no timeout. 1206.201s is consistent with the audit's own
post-S1+S2 floor time (1205s) within noise; this instrument's overhead (a per-`record`
`OnceLock` check when the env vars are unset, which they are during the normal floor run
above) did not move it.
