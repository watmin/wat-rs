# VIGILIA 2026-08-30 — the work list

> **Cast at HEAD `78b1fad56`.** 19 wards: 17 inward in parallel, then `experiri` (which drives),
> then `circumspicere` (which surveys what the rest turned their backs on). `mora` out per the
> standing orders in `VIGILIA-LOOP.md`. **41 L1 + 70 L2.**
>
> The prior full cast was 2026-08-24. Between them: 184 commits and +19,496/−11,996 lines in
> `src/rete` alone, including 19 files that did not exist then. **The gates were green throughout;
> every finding is on a surface the 28 lints cannot see.**

> ⛔ **STATUS IS EDITED HERE, IN PLACE. Never append a closure below a row.** A row's status
> living in two places IS the defect — `exigere` found exactly that in this arc's TRACKED
> DECISIONS this same day, in a section whose own header bans it. One row, one place.

> ⛔ **DO NOT WORK THIS LIST TOP-TO-BOTTOM AS 111 ITEMS.** Five wards independently found five
> instances of ONE class, and it is Class A below. Pulling that root is worth more than the rest
> of the list combined. A count is not a finding — `partire`'s row sat open a week in this arc
> because it recorded counts and no proposals.

---

## ⛔ THE CLASS ABOVE THE FINDINGS — an invariant proven at ONE door, assumed at ALL of them

**There are THREE doors into a Session:** `arm-session` (`compile-all`), `import_export`, and a
hand-assembled `Session` record. **The first proves things. The other two do not**, and almost
every instance below is the second door.

| # | invariant | proven at | assumed at | found by |
|---|---|---|---|---|
| A1 | node ids ascend (topological) | minting, on compile | the wire, unchecked | `circumspicere` |
| A2 | fold values are `i64` | `build_rete_arm` | `acc.rs`'s `panic!` | `circumspicere` |
| A3 | acc-form head is callable | the fence, via `RETE_OPS` | the executor, via `sym.get` | `experiri` (driven) |
| A4 | session byte ceiling | one thread-local origin | every session on that thread | `secare` + `sequi` |
| A5 | termination is verified | `arm-session` | *"the one door EVERY rule passes"* | `circumspicere` |

**The question to ask of every remaining invariant in this engine: which door proved this, and
how many doors are there?**

---

## ✅ CLASS A — the doors. **CLOSED 2026-08-31** (A1, A2, A3, A4, A5, A6, A7). This was the root.

`export.rs:15-17` states its own law: *"it consumes bytes some other process wrote, and every one
of them can be a lie."* `export.rs:2015` calls `import_export` *"the file's one place where
untrusted bytes become a runnable network."* The header counts **three walls** (range refusal,
slot bounds, three compat gates). **None is a graph wall.**

| id | site | what | fix shape |
|---|---|---|---|
| ~~**A1**~~ ✅ `788e5b66d` | `export.rs:2112-2128` | no structural validation of the imported graph: nothing checks a child id resolves, that a Negation/Exists/Accumulate `aid` names an **Alpha**, or that `child > parent`. `node.rs:192` and `arm.rs:592` both state the passes **require** ascending id order. | a fourth wall between phases 3 and 5, refusing with `malformed` like the other three — then state it in law 3 so the wall count and the walls agree |
| ~~**A2**~~ ✅ `c449cd24d` (acc.rs half; `fire/mod.rs` remains) | `acc.rs:64,72,76,83,129,139-142`; `fire/mod.rs:1400,1406,1415,1615-1628` | `panic!` licensed by a rune reading *"AccFold compile proved i64"* — a proof `import_export` never runs. `unpack_fold` (`export.rs:1382`) takes the fold key straight off the wire; `import_export` interns it at `:2199`. No `catch_unwind` on the program path. Rust panic, no span, no rule named. | return `Result<_, EvalBreak>` as `driver_of` (`fire/mod.rs:239`) already does for the same class — **a wire-reachable invariant may not be spelled `panic!`**. A rune here must name the DOOR, not the compiler. |
| ~~**A3**~~ ✅ `17fc5fb3e` | `expr_ir/mod.rs:947` / `arm.rs:429` | driven both halves: `PersistentVector/length` DIRECTLY as an acc-form head → `unknown rete-defn` **about a row of the very `RETE_OPS` table the fence consulted to admit it**; the SAME op behind a one-line user `defn`, SAME position → `"fired"`. The capability was real; only the ladder was missing. | ✅ `lower_named_rete_fn` gains the `rete_op_index` branch its sibling `lower_list` had. Tightening the fence instead was **rejected on the wrapped control** — it would delete a working capability to make two registries agree. Class gate in `reachability.rs` with the eligible set **computed from `RETE_OPS` by param shape** (1 of 79, never named), proven to fail three ways incl. a wrong-opcode differential. ⛔ **The banked `harness-experiri` recon was NOT appended** — ONE assertion across EIGHT tests, counted. ★ **And a regression my own brief authorized was caught and reversed**: an arity-2 acc head was refused at FIRE with a span into `fire/acc.rs`; a compile-time fence at `arm.rs:430` moves location, op name and timing, against `expr_ir/mod.rs:14-19`. See `strike-acc-head/`. |
| ~~**A4**~~ ✅ `42704d57b` | `alloc_counter.rs:118,133` / `session.rs:1404` | `SESSION_ORIGIN` is one `Cell` per THREAD, rebased unconditionally by every `compile-all` (`arm.rs:1205`). Second session re-bases the first; `saturating_sub` then floors the reading at **0 — no ceiling at all for the rest of that session's life**. `arm_lease.rs:141` is a GREEN test holding two live sessions on one thread. | ✅ origin keyed by network identity, the way `ARM_TABLE` keys its arms, and threaded into both doors. **Two corrections to this fix shape, both from contact:** (a) *"`origin > thread_bytes()` … is proof the origin belongs to another session, and must refuse loudly"* is now FALSE and was not implemented — with per-session origins that inequality is the ordinary consequence of a sibling session freeing on the same thread, and refusing loudly there would refuse the innocent. (b) keying is only HALF the fix: `mark_session_origin` must also refuse to overwrite an origin it already holds, or a re-`arm-session` on a live session re-bases it under its own key. See § A4 closure below. |
| ~~**A5**~~ ✅ `7e24c3257` | `arm.rs:1294`, `stratify.rs:833` *(line numbers corrected)* | **two halves, and the row understated the first.** `import_export` does not skip the analysis — it **never calls the verifier at all** (grep: no hit), and `rete_arm_get_or_build` is a **second** false door, so the "one door EVERY rule passes" sentence was false twice. And `Ok(())` came from **five** silent sites meaning three things; driven: a `Rule` with empty `:lhs`/`:rhs` — the imported shape — makes `compile-all` answer `"Compiled"`. | ✅ `TerminationVerdict{Proven, NotAnalysable{rules}, Refused}`, all three arms matched at its one caller; behaviour unchanged. ★ **The skip count TAINTS the proofs** — my stone said the two early exits were proven *unconditionally*, and the repro reaches one of them, so without the taint the split would have shipped and changed nothing. The two false doors are now **rows in `rete_header_claims_are_asserted`**, per that lint's own law. Six mutations, each with a predicted distinct red set. See `strike-termination-silence/`. |
| ~~**A6**~~ ✅ `bb0256e38` | `export.rs:746,296,275` | **not a SIGSEGV — a stack-guard ABORT** (`fatal runtime error: stack overflow`), which no `catch_unwind` reaches. And the real defect is sharper than "deep input crashes": the SAME 20,000-deep Export is **ACCEPTED on a 256 MiB thread and aborts on a 2 MiB one** (both driven), so import had no depth criterion at all — acceptance was a property of the importing THREAD. | ✅ wall 5: ONE budget (`MAX_IMPORT_DEPTH = 300`) threaded through **five** mutually recursive unpackers. **The finding named one tower and there were three** — `unpack_driver` is self-recursive and its doc stated the defect as a feature (*"round-trips without a depth parameter — the wire's nesting IS the recursion"*), `unpack_cond_op` is the third. Bound MEASURED (corpus max 3 × 100; abort window 3,000–5,000), both numbers at the constant. Mutation re-driven: budget in `unpack_expr` only leaves the `:and` probe GREEN and the other three RED. See `strike-import-depth/`. |
| ~~**A7**~~ ✅ `b0e3377e9` | `export.rs:2128` + `pmap.rs:148` | **worse than uncounted**: `session_bytes` does `entry(key).or_insert(now)`, so an unmarked session's origin is set at the FIRST CHECK and the whole build goes retroactively free. Driven, same 2 MB: marked-at-birth **2097268**, never-marked **0**. And the build is quadratic — per-pair cost doubles as N doubles (1.05→4.87 µs at 500→4000) — with no node cap. | ✅ origin captured as the door's FIRST statement and filed after the build (the key is the built network's identity, so the reading and the filing must split); ceiling checked; **wall six** refuses past `MAX_IMPORT_NODES = 10_000` — **measured**: corpus max 63 across 34 importing tests, ~122 ms worst case on the driven curve. `from_pairs` affirmatively CUT — the cap bounds N so the quadratic is bounded with it; before-curve recorded for a speed stone. ⛔⛔ **Mutation 3 indicted A4**: removing the non-clobber rule leaves A4's `rearm` arm GREEN, though its fixture says *"only this arm can see it"* — masked by `LAST_ORIGIN`, which shipped in the SAME commit. Struck at the site. See `strike-import-accounting/`. |


### ✅ A4 — THE SESSION CEILING'S ZERO POINT. CLOSED `42704d57b`.

`alloc_counter::SESSION_ORIGIN` was one `Cell` per THREAD. It is now an `FxHashMap` keyed by the
session's network identity (`arm::network_identity` — the same key `ARM_TABLE` uses), with a
`const`-initialised one-entry `Cell` cache in front of it, and `session_ceiling_breach` /
`check_insert_ceiling` carry the key down from all three call sites.

**What it bought, stated at its true size.** A thread-local byte counter still cannot separate two
sessions sharing a thread — `alloc_counter`'s own doc said so before the fix and still says so
after. Session A's reading includes session B's bytes, so A **over-counts and refuses EARLY**,
which the module already rules the safe direction. **The strike converted an unsafe silent failure
(a session with no ceiling at all) into a safe conservative one. A per-session origin is not a
per-session allocator.**

**Three things measured rather than assumed:**

1. **The prescribed mutation was INERT.** "Make `mark_session_origin` clobber regardless of id"
   (`or_insert` → `insert`) left every arm of the probe GREEN, because with distinct keys the two
   behave identically. The mutation that actually bites is dropping the key entirely. A third probe
   arm (`rearm`) was added to close the hole the inert mutation exposed: it hands the SAME session
   back to `arm-session` mid-life, and it is the only arm `insert` vs `or_insert` can move.
2. **The map cost is real on the insert hot path** — `+51 / +77 / +75` ns per fact, a consistent
   ~1.5%, measured with two binaries built from the same tree and run INTERLEAVED
   (`wat-scripts/scratch-pad/bench-arc278-session-origin-insert-door.wat`). The one-entry cache
   brings it to `-43 / -3 / -86` against the pre-strike binary, i.e. back inside the noise. The
   `const` init the old doc defended did not disappear; it moved to the slot read per fact.
3. **Two `compile-all`s of the same rule set do NOT share a network identity** (measured:
   `Some(17592186044421)`, `…428`, `…435` in one process), so the key distinguishes sessions in the
   corpus. STOP trigger 2 did not fire.

⚠ **NOT TAKEN — A7's import half, and it is one line.** `import_export` (`export.rs:2312`) still
never calls `mark_session_origin`; the patch is
`crate::alloc_counter::mark_session_origin(network_identity(&network));` beside the existing
`rete_arm_intern`. DESIGN admitted it "only if it costs one call" — it does — but taking it means a
SEVENTH file against a blast radius stated as six, and "a seventh is a STOP, not a delta" is the
harder rule. **Left for A7, and mostly harmless meanwhile:** an imported session now self-marks
under its OWN key on first ceiling check and nothing can clobber it, which is the property that was
missing before.

### ✅ A2b — THE SILENT ZERO. CLOSED `d081142a9`.

Surfaced by A2's own rider, driven and deliberately not committed then. `operand_slot`'s
`Option<usize>` carried TWO facts — `bucket.first()?` (empty bucket, where `Sum`'s identity
genuinely is 0) and `.position(…)` (the var names nothing). `Sum` read the second as the first and
returned `i64(0)`; `Min`/`Max`/`Mean` read it as absence and **silently dropped the derived fact**.
One `Option`, two consumers, two different wrong answers. Split into
`EmptyBucket` / `Slot(usize)` / `Unbound`, no `_ =>`; both `Unbound` arms refuse. Both arms
mutation-proven independently, each reddening only its own probe.

⚠ **AND THE ORCHESTRATOR'S COUNTER-PROOF ROW COULD NOT HAVE FAILED.** EXPECTATIONS named the
existing `empty_case` tests as the proof that an empty bucket still yields the identity, and called
that row "not optional". Both call `fold_i64s` DIRECTLY with `std::iter::empty()` — neither enters
`fold_bucket`, neither touches `operand_slot`, neither covers `Sum`. They would have stayed green
under exactly the over-correction the row existed to catch. The rider read them, said so, and BUILT
the real counter-proof (3 tests through `fold_bucket` and `operand_slot`). Recorded as
`[[a-named-counter-proof-is-still-a-claim]]`.

### ⏭ NEW — the refusal span carries no information

`acc_refusal` (`acc.rs:62`) expands `rust_caller_span!()` **in the helper body**, so all **11**
refusal call sites in that file report location `acc.rs:62`. Only the message text distinguishes
the arms — which is why A2b's two refusals had to spell out `:sum` vs `:min/:max/:mean` in prose.
Introduced by `c449cd24d` and missed by both that rider and this orchestrator. Same class as
Class E5 (`conformare`: `refuse_export_without_arm` gets a synthetic span at both call sites while
the real one is a frame up). If that span is load-bearing anywhere, it is a strike.

### ⏭ NEW — `docs/**` IS A GRAVEYARD BY CONSTRUCTION, and it already has a body

**Found 2026-08-31 by the builder asking one question of a file I had just banked: *"where does
this file live such that it does not run?"***

`wat-rs/CLAUDE.md` kills this class for scratch `.wat` — they go under `wat-scripts/` **because**
`every_wat_scripts_file_loads` parses and type-checks every file there, *"so a scratch program that
rots goes RED and cannot become a graveyard that reads like live code."*

**The escape hatch from that gate is a directory with no gate.** `docs/arc/2026/06/278-rules-engine/`
holds 8 orphaned `.wat` plus 7 `.rs.txt` / `.wat.txt`. Driven on the current runtime:

| file | verdict |
|---|---|
| `probes/enum-holds-record.wat` | runs, prints — alive |
| `probes/red-send-cause-is-not-matchable.wat` | runs, prints — alive |
| `probes/red-owner-signals-child.wat` | fails — **declares itself RED BY DESIGN in its header** |
| `probes/surface-field-dispatch.wat` | fails — **ROTTED, silently, ~8 WEEKS** |
| `harness-experiri/*.wat` (4) | 3 refuse by design, 1 fires — alive |

`surface-field-dispatch.wat` (2026-07-05) says in its own header *"PROVES the storage-abstraction
model — **prints 142**. Run: `cargo wat <this>`."* It now dies at startup: `defsurface` gained a
required `:nature` and the probe was never migrated. **Nothing has noticed since the grammar
changed.**

⚠ **AND `red-owner-signals-child.wat` STATES THE EXACT REASONING I USED**, months earlier: *"It
lives under `docs/…/probes/` (NOT `wat-scripts/`) precisely because `every_wat_scripts_file_loads`
walks `wat-scripts` only — a deliberately-failing probe parked there would break that gate."* The
reasoning is sound; the consequence is that deliberately-red and genuinely-rotted files now sit in
one directory, indistinguishable.

**FIX — the gate must be able to tell the two apart, which is the whole point.** Walk
`docs/arc/**/probes/` and `docs/arc/**/harness-*/`, and require every `.wat` to EITHER load on the
current runtime OR carry an explicit red-by-design marker in its header. Then
`surface-field-dispatch` reddens today, `red-owner-signals-child` passes on its declaration, and the
next grammar change cannot kill a probe in silence. **Do not simply move them under `wat-scripts/`
— that is what the prior hand correctly refused, and it would break the existing gate.**

⚠ My own 7 banked `.rs.txt` / `.wat.txt` are in the same unchecked place. They are short-lived by
design (a rider lands them within hours), but `harness-experiri/positions-3-4.rs.txt` has now sat
since 2026-08-30 and **its README claimed it was a working gate when it holds ONE assertion across
EIGHT tests** — see that file's own CORRECTION block. A `println!` of a correct matrix looks exactly
like a proof.

---

## CLASS B — resource lifetime

| id | site | what | fix shape |
|---|---|---|---|
| ~~**B1**~~ ✅ `7319c1ea4` | `wat/rete/syntax.wat:308` | release sat in a `do` AFTER the body, so **any unwind skipped it** — driven RED on BOTH paths (a wat error and a host panic; `assertion-failed!` PANICS, `runtime.rs:15922`, so they are separate mechanisms and needed separate probes). Table grew 0 → 1 on each. | ✅ **the root was NOT the `let`+`do` shape** — `with-open-file` has the identical shape and is safe because its resource is a Rust value whose `Drop` closes the fd. An `ArmLease` guard **adopts** `compile-all`'s lease; the `do` is DELETED, not supplemented. Two holes the design missed, both found by driving: `rete_arm_release` needed `try_with` (a guard dropping after `ARM_TABLE` is destroyed **ABORTS**, not panics — reproduced standalone; nothing in the suite reaches teardown, so this is a **coverage** finding), and `:rust::rete::ArmLease` is hand-minted so `is_pure_type`'s `None => true` arm judged **a live resource handle PURE** — driven with an unregistered positive control, closed with one impure-path row. See `strike-lease-unwind/`. |

---

## CLASS C — the instruments. Blocks trusting ANY recorded cost number in this arc.

⚠ **Two of these are my own, committed this morning and reported as finished.** Both have the
same shape: **the commit message asserted a general fix while the diff performed a specific one.**

| id | site | what |
|---|---|---|
| ~~**C1**~~ ✅ `119214aef` — 103 accumulators swept, mean → minimum. ⚠ SCOPED 2026-08-31 first: **96 divides, 7 files**, not the "~18 in 8" this row said | `tests/mod.rs:493` + 96 divide-by-`RUNS` sites | `89e8c3ed0` rewrote **only the label**. `git blame`: line 493 (`MINIMUM of {RUNS}`) is that commit; lines 528-542 (`stat` returning `sum/len`, `net_of`, `total_mean`) are untouched. `render_phase_table` renders the axis tables for accum, node-share, cascade, fanout, harvest, strat, gather-probe. In `accum_alpha_leftover_split` the **two halves of one table disagree** — isolated arms `.min()`, in-fire rows mean, one header. Sites: `accum_alpha_cost.rs:112,346` · `accum_cost.rs:310,607,1632` · `cascade_cost.rs:372` · `fanout_cost.rs:424,619,741,852` · `harvest_cost.rs:598` · `rank_and_instrument.rs:375,465,573` · `strat_cost.rs:225,326,422,598` |
| ~~**C2**~~ ✅ `00ca6b0eb` | ⚠ **the row said TWO mislabelled arms; there is ONE.** `accum_cost.rs:1383` carries no label — C1's own sweep moved the lines the day after this cast. And there are **FIVE** `(engine)` sites, not the three anyone had counted, incl. one inside a `#[cfg(test)]` mod in a `src/` file and one spelled `(THE ENGINE)`. The false arm's claim is **dropped outright**. |
| ~~**C3**~~ ✅ | `accum_cost.rs:1603` (⚠ the row said `:1630`) | reads phase mark `setup:seen:insert`. `of` is `unwrap_or(0)` → prints `0.00 ms` as a measurement and `−S` as a difference. On the floor. | ✅ **the row understated it.** All three in-fire rows read 0.00, and the other two marks DO fire: `fire/delta.rs:270-276` wraps BOTH `phase_end`s around the same two allocations, so `setup:seen` is **coextensive** with `:alloc` and `setup:seen:insert` names work that is in `alpha`+`production` instead (via `seen_insert`). Rows dropped, premise restated. ⛔ **My sweep said ONE dead name; the truth is FIVE** — a naive quote regex inverted parity and read the gaps between literals. The other four are `ALPHA_KIDS`, a **handled** absence (`kids_retired = kid_pairs.all(==0)`, no arithmetic on the zero), runed not deleted. Gated by `census_name_read_by_a_cost_test_is_emitted.rs` — 4 arms, 5 mutations, lints **210/210**. See `strike-dead-phase-mark/SCORE.md`. |
| ~~**C4**~~ ✅ | `accum_alpha_cost.rs:233,1102` | the arm labelled `A alpha_activate_fact` (**THE production path**) is handed an empty `bind_only`, disabling the `skip_span` branch production takes. ⚠ the row said `:1080` and *"~all 80,200 pairs"*; the site is `:1102` and 80,200 is the `compiled:calls` count, which is a UNION of both branches (see C10) — the workload is **40,200 facts**. | ✅ **driven first** (`f90d4c126`): **3 of 3** conds bind-only, production interns **nothing** (`pool=0`), the arms intern **120,200**. ⛔ The obvious fix breaks both tables — they are cumulative ladders and `A−M` means *push* only while `A ⊇ M`, which holds *because* the branch is off. So the old arm is KEPT and relabelled `A activate, skip_span forced off`, and a production-faithful `Ap` row is added beside it, deriving nothing. Both land **below `M`** (11.33 vs 11.93; 10.82 vs 11.91). ⛔ **My scorecard pinned milliseconds and three readings of the unchanged tree spanned ~16%** — the invariants `A > M` and `Ap < M` are what hold. See `strike-bind-only-arms/SCORE.md`. |
| ~~**C5**~~ ✅ | `binding_repr_bench.rs:665` (⚠ the row said `:664`) | a tautology on the release floor while measuring a representation the engine settled a third way; `:24` apportions an "in-engine bind" across three arms, none on the bind path. | ✅ all three fixed. The assert becomes the check its own comment declared (`> 0`) plus three orderings measured at ≥2.6x; both named tests now head with the live `BindSpan` (`session.rs:64`) and cite the stone the comparison is evidence FOR; the `163 ns` anchor is dropped (4 mentions → 1, which records why). ⭐ The rider **refused a fourth ordering my sketch implied** — EXTEND at card 1 ran 1.19–2.02x over twelve drives, inside the noise, and would have manufactured a flake in a repo that bans them. ⭐ Its mutation showed the table printing **DOMINANCE: YES** — R60's conclusion inverted — while the OLD assert passed green. See `strike-binding-repr-claims/SCORE.md`. |
| ~~**C6**~~ ✅ *(the check refused, deliberately)* | `node_share_cost.rs:264` (⚠ the row said `:61,286`) | reconstructs the `filter` phase from the **retired interpreter** against a hard-coded 2026-08-01 constant. Only arm F is the fire path. | ✅ constant **deleted**, phase read **live**, reconstruction moved to the **native** arm. ⛔⛔ **THE DECLARED CHECK IS STILL NOT ASSERTED, AND THAT IS THE RESULT:** made runnable for the first time it FAILS at **~7x**, stably (684/693/734/723/686/698% over six runs — a 7% spread inside the ~16% noise floor). **The fire calls `exec_where` ZERO times here** — `dispatch_where_tests` takes the `proven && is_pure_cmp` reuse branch (`fire/mod.rs:2038`); measured `evals 0, envs 0, keyallocs 0` at every size. So arm F is scaled to the pre-where-tree 10,000 and *no rescaling rescues it*. Refusal recorded in-code with its samples. ⛔ My brief quoted the **10/200** row (0.14) of a three-size table for the **50/200** row (0.38), and its central premise — that the scales are commensurable — was false. See `strike-stale-reconstruction/SCORE.md`. |
| ~~**C7**~~ ✅ `00ca6b0eb` | ⛔ **THE RULE AS WRITTEN WAS WRONG** — *"only if its body CALLS the production function"* would have **deleted a TRUE claim**: the `L` arm replicates `root_for` inline because production is not callable there, and is already pinned by a gate asserting the type and body EXACTLY. Widened to **an engine claim carries its evidence — the function it CALLS, or the gate that PINS its replication.** Gated by `rete_engine_label_names_its_evidence.rs`; resolution excludes `#[cfg(test)]` modules (a path prefix was the wrong boundary — 26 `src/rete/` files carry one). See `strike-engine-label/`. |
| ~~**C8**~~ ✅ `d9fb1b88f` | `wat-scripts/perf/grid/run-axis.sh` + the 23 recorded `GRID-*.txt` | ⛔ **THE CLARA GRID — the instrument this arc's HEADLINE numbers come off — recorded NO dispersion for its own engine.** The verdict carried `:min`/`:max` for the RATIO from day one and nothing for `:wat-ns`; the per-run values were collected into `WAT_NSS`, averaged and discarded. Every "N% faster" ever read off it was unfalsifiable. **Measured:** `fanout [40000]` showed +11.4% vs the 08-31 baseline; three consecutive 5-run verdicts on the SAME binary spanned **22.96M → 23.70M → 25.35M ns (10.4%)**, and two same-build sweeps disagreed by up to **16.3%** per cell with the ratio column swinging **+42%**. ✅ `:wat-ns-min`/`:wat-ns-max` now emitted, and a new `compare-grids.sh` tests **disjoint intervals** (a delta-vs-threshold test false-positived 3/33 on identical builds). Proven both ways: 33/33 within noise same-build, and a +40% injected regression fires on exactly one cell. |
| **C9** | every `GRID-*.txt`, all 23 | ⛔ **THE THREE-WAY SPEC CHECK HAS NEVER ONCE BEEN RUN.** All 11 axes call `fire-rules$oracle` and `run-axis.sh` carries the full diagnosis table (`oracle≠clara ⇒ SPEC wrong`, `native≠clara ⇒ fast path wrong`, `oracle≠native ⇒ PORT bug`), but **0 of 33 cells in every recorded grid carries `:oracle-accuracy`** — every one was run under `GRID_SKIP_ORACLE`. So the arc has 23 grids proving the fast path matches *Clara* and none proving it matches *its own spec*. ⚠ Cost is why: the oracle fires in-process after the timed region and takes the sweep to ~**4 hours** (builder's figure); `min-finding [500 3]` wall went **384ms → 12,990ms**, `:fire-share-pct` **0.14 → 0.00**. **This is its own strike with its own sizes and timeout — NOT a grid mode, and it must not share an artifact with a perf sweep.** The 2 cells that did run came back `:oracle-accuracy :match :port-accuracy :match`. |
| ~~**C10**~~ ✅ | `accum_cost.rs:52` + `compiled_cond.rs:928` / `fire/delta.rs:78` | ⛔ **A COUNTER THAT CANNOT SEE THE BRANCH IT IS QUOTED FOR.** `compiled:calls` is bumped in the `skip_span` arm (`delta.rs:78`) **and** inside `exec_compiled_with_key_ids` (`compiled_cond.rs:928`), deliberately — so it is a designed UNION and reads **80,200 either way** (driven 2026-09-02; it refuted my first C4 probe). `accum_cost.rs:52` pins that number as a correctness assertion and is therefore blind to which branch produced it — the exact defect C4 turned out to be, one file over. |
| ~~**C11**~~ ✅ | `accum_alpha_cost.rs` table 1 in-fire block | Rendering, cosmetic but live: `seed` / `delta` / `seed+delta` print **two columns left** of `FIRE` / `alpha`. Rust's `\`-newline continuation strips the *leading* whitespace of the continued line, so the intended indent never reaches stdout — it only shortens the pad. Confirmed in the orchestrator's own output. |
| **C12** | `node_share_cost.rs` arms A/B/D/F + the whole "ONE ROUND'S WORTH" premise | ⛔ **NO ARM IN THIS FILE MEASURES THE FILTER PHASE AS IT EXISTS.** Opened by C6's strike. The fire performs **0** `exec_where` calls, **0** `Environment` builds and **0** key allocations on this axis; arms A, B, D and F all measure work the where-tree deleted. At the true scale F contributes nothing and C alone accounts for ~34% of the phase. The missing ~66% — the per-token `where_tree.candidates` walk, `bind_view`, the two `HashSet` builds (`proven`/`maybe`) and the `d_beta` pushes — has no arm at all. **Until an arm set covers the phase, the declared reconstruction check cannot honestly be asserted.** |
| ~~**C13**~~ ⛔ **WITHDRAWN — IT WAS NEVER A QUESTION** | `src/rete/matcher.rs` | Opened by C5's DESIGN as *"reached only from tests, may be a genuine `purgare` target"*. **That claim was FALSE.** `matcher.rs:278,357` register these as the wat primitives `:wat::rete::alpha-match` / `-local` / `-under`, and `wat/rete/oracle/pass.wat:21,22,193,305,551,563,661` + `accum-pass.wat:247` call them. The orchestrator counted Rust callers under `src/` and never grepped the `.wat` corpus — **in a self-hosting repo a Rust-caller count is not a reachability proof.** The path is off the native round loop and IS the wat oracle's interpreter: live code. Closed rather than left open, because an open row saying "maybe reap this" is a trap. |
| **C14** | `compiled:calls` — `compiled_cond.rs:928`, `fire/delta.rs:78`, `fire/pass/alpha.rs:122` | ⛔ **THE COUNTER IS NOT A CALL COUNT, AND ITS NAME SAYS IT IS.** Two sites bump it per call; the third is `census_count_n("compiled:calls", ids.len() * aids.len())` — a **multiplicative bulk add** for occupancy leaf-fill. `accum_cost.rs:46-47` glosses the pinned 80,200 as *"one per (fact, matching alpha) pair"*, which holds for THIS workload and is not a property of the counter. Found by the C10 rider; the orchestrator's brief said "a union of three sources" and named two, having already seen the third in a C4 grep. This is `[[a-right-number-vouches-for-a-wrong-label]]` in the substrate. |
| **C15** | `tests/lint/rete_names_in_wat_scripts_resolve.rs` | ⛔ **A SYNTHESIZED RECORD ACCESSOR CAN NEVER BE ATTESTED, SO ANY `wat-scripts/` FILE TOUCHING ONE IS UNAVOIDABLY RED.** `:wat::rete::DerivationNode/via` and `:wat::rete::DerivationStep/constraints` are **live** — a program using them runs — but `Type/field` accessors are generated at freeze from the `defrecord` (`wat/rete.wat:374`) and never appear textually under `src/` or `wat/`, which is the only place the resolver looks. **None of the gate's three offered fixes applies**: not a typo, not retired, and a `rune:lint(rete-name-unminted)` would be a lie about a minted name. Cure: teach the resolver that `:wat::rete::<Type>/<field>` resolves when `<Type>` is an aggregate declared in `wat/rete.wat` carrying `<field>`. ⚠ Found because **the orchestrator turned the floor RED** committing a recon `.wat` into that tree without running the gates that read it. |
| ~~**C16**~~ ✅ `de4ff4af9` | `fire/delta.rs:144-151` | the occupancy differential filtered `predicted` by the SAME predicate that decides batch membership, so it compared writer 2's output against writer 2's output and read `extra=[]` while D7 dropped a fact. | ✅ filter deleted. Driven, same workload, armed: **filter present + D7 live → `predicted=2 actual=2 extra=[]`**; **filter removed + D7 live → `predicted=3 actual=2 extra=1`**; filter removed + D7 cured → clean. **The gate that was blind to a fact-dropping bug now names it.** Licence is structural: a class is `batchable` only if every leaf is `bind_only` with no `fact_bind`, and `bind_only_fields` is `Some` only when every op is a `Bind` — so every fact of the class occupies every leaf unconditionally. Gated by `seed_leaf_occupancy_differential_predicts_a_mixed_class`, mutation-proved `(2,3,0,1)` vs `(3,3,0,0)`. |
| ~~**C17**~~ ⛔ **CLOSED — STRUCTURALLY, NOT DRIVEN** | `src/rete/kernel/insert.rs:221` | `insert_facts_on_session` per-element-checks its argument **only when it is a `PersistentVector`**, so any other shape reaching `vector_concat_inner` skips `require_record_fact`. | ⛔ **The hole is closed downstream, and the argument is positive rather than a failure to construct.** `vector_concat_inner` (`collection/eval.rs:756-766`) accepts **Vec+Vec or PV+PV, SAME KIND ONLY** — the right side must match the left. The left is the session's `facts` field, declared `facts <- :wat::core::PersistentVector` (`wat/rete.wat:197`). So a non-PV argument **errors at the concat** rather than slipping past unchecked, and the `if let`'s only matching shape is the one it checks. ⚠ **NOT DRIVEN** — my probe used `:wat::core::vec`, retired (`remedy/retirement.rs:115`). What would drive it: obtain a `Value::Vec` via `into`/`keys`/`vals` (`collection/eval.rs:547,730,746`), hand it to `insert-all`, and observe the concat refuse. ⚠ The shape is still **fragile**: it is correct only because a downstream invariant holds, not because the door checks locally. |
| **F2-e** | `wat/seq.wat:163,262` · `wat/core.wat:1585` | ⛔ **`:wat::rete::insert-all-spec` IS A PHANTOM, CITED THREE TIMES WITH A LINE NUMBER PAST THE END OF ITS FILE.** Two of the three cite it as `wat/rete.wat:1508`; that file is **533 lines**. `grep -rn 'insert-all-spec' wat/ src/` finds nothing but the three citations themselves. Found incidentally while auditing C17 — I went to read the signature the citation promised and there was no signature and no file position. ⚠ `rete_citation_resolves` gates backticked names in **`src/rete`** comments; nothing gates prose under **`wat/`**, which is where these three sit. That gap is the row, not the three fixes. |

### ⚠ C1'S SCOPE, MEASURED — and the orchestrator's third wrong instrument on this row

Counted 2026-08-31 on the real signal (files binding `let r = RUNS as f64` **and** carrying a
`MINIMUM of` header), which is the only honest population:

| file | divides by RUNS | MINIMUM headers |
|---|---:|---:|
| `accum_cost.rs` | 29 | 8 |
| `fanout_cost.rs` | 28 | 5 |
| `rank_and_instrument.rs` | **21** | 5 |
| `strat_cost.rs` | 7 | 5 |
| `accum_alpha_cost.rs` | 6 | 5 |
| `cascade_cost.rs` | 4 | 2 |
| `harvest_cost.rs` | 1 | 5 |
| **TOTAL** | **96** | **35** |

⛔ **A FIRST COUNT OF 37 WAS WRONG AND IS KEPT HERE AS THE LESSON.** The regex was
`^\s*[a-z_]+ */= r;` — shaped from the first site read (`fire /= r;`) — and it cannot see
`let (a, b) = (a / r, b / r);`, which is how `harvest_cost` and `strat_cost` spell it. It reported
`rank_and_instrument.rs` as **zero** where the file has **21**. `vocare` had named those exact
sites in the cast and the orchestrator's own grep contradicted the ward; **the ward was right.**

Same class as `--exclude tests.rs` matching nothing (2026-08-30) and `doc-coverage.sh` counting
1,917 `#[cfg(test)]` lines as production: **an instrument shaped by the first example it saw,
reporting a subset as a total.** Third instance on this arc, and this one was produced in the same
breath as a promise not to produce another wrong number.

**For whoever draws this strike:** the population is the 96, `render_phase_table`'s `stat()` (which
renders the axis tables for fanout, accum, node-share and rank-and-instrument), and `net_of` /
`total_mean` which both read `stat(..).0`. Do NOT trust a `/= r` grep; the spellings are at least
three (`x /= r;`, `let (a,b) = (a / r, b / r);`, and `*x /= r;` inside a loop).

---

## CLASS D — engine behaviour

### ~~D10~~ ✅ `e38b1f46a` — THE RETE `:then` RHS NOW TYPES ITS FIELD VALUES (top level)

**Driven 2026-09-02.** The same record construction is type-checked everywhere in the language
**except** inside a rule's `:then`:

```
ordinary   (:td::Bad :n "x")   ->  #wat.check/TypeMismatch
                                   ":td::Bad: parameter #1 expects :wat::core::i64; got :wat::core::String"
in :then   (:tr::Bad :n ?s)    ->  compiles, fires, derived fact = #tr/Bad {:n "not-an-i64"}
in :then   (:tl::Bad :n "…")   ->  compiles, fires, derived fact = #tl/Bad {:n "LITERAL-STRING"}
```

Driven for **both** a bound `?var` and a **literal**, each beside a well-typed control that derives
(`Good count: 1`) — so the probe is live, not vacuous.

**The RHS walls that exist are all structural**: `RhsArityMismatch`, `RhsMissingFields`,
`RhsPositionalConstructionRetired`, `RhsUnresolvableOperand`. **None types a value.**

⚠ **NOT a parametric-record problem, and not D7's.** `:tr::Box.s` is concretely
`:wat::core::String`. And the `:when` side *does* reason about types — a comparison against an
erased `:T` is refused with `ConstraintTypeNotComparable` (driven). The gap is the `:then` surface
specifically: a wrong-typed value enters the **fact set**, where every downstream consumer trusts
the declared schema.

Repro: `wat-scripts/scratch-pad/d10-then-rhs-is-not-type-checked.wat`.
✅ **CURED at the top level of a `:then` fact form** by `RhsFieldTypeMismatch` + `check_then_field_type`,
called from **both** producers (kwargs and positional — each proven by a call-site mutation).
**Corpus measured before the cure was written: 1664 `.wat` scanned, ZERO newly-failing.** Proven not
to over-refuse: making `ComputedNotDerivableHere` a refusal REDs the not-knowable probe *and takes
four pre-existing corpus tests with it.* Floor 5351/5351. See `strike-then-rhs-types/SCORE.md`.

### ⛔ D11 — THE SAME DEFECT, ONE LEVEL DOWN: NESTED CONSTRUCTORS ARE STILL UNTYPED

D10's cure types a `:then` fact form's own fields. It does **not** reach a constructor nested inside
one, because `walk_nested_constructors` has no `binds`. Driven by the orchestrator at `e38b1f46a`:

```
:then [(:nh::Outer :i (:nh::Inner :n ?s))]      ?s : String, :nh::Inner.n : i64
  ->  compiles, fires, #nh/Outer {:i #nh/Inner {:n "nested-string"}}
```

Same class, same fact set, one level deeper. The cure needs `binds` threaded into a **recursive**
walker that has four other producers — a signature change the D10 rider deliberately declined rather
than widen its blast radius, which was correct.

⚠ Two further bounds on D10's cure, stated by that rider rather than discovered later: the check is
only as sharp as `rete_type_segment_of` (two distinct enums both segment to `enum`), and
`NotComparable` is **deliberately** passed — a parametric record's erased field arrives through that
same channel, and refusing it would be D7's ground.

### ⛔⛔ C18 — `assert!(!ok)` IS UNFALSIFIABLE IN THIS REPO'S NEGATIVE-FIXTURE IDIOM

Every `.wat.bad` fixture ends `(:user::main [] -> :wat::core::nil nil)` — **which is itself a startup
failure** (`#wat.macro/MainSignatureError`, UselessMain). So with the wall the fixture exists to
prove mutated away, **the file still fails**, for the wrong reason, and the `!ok` half of the probe
**cannot go red under the very mutation it exists to detect.** Only the `.edn` golden can.

Found when a D10 mutation dumped `MainSignatureError` where the golden expected
`RhsFieldTypeMismatch`. That rider gave its own four fixtures real `main`s — they now run and print
when the wall is absent, so both halves are load-bearing.

**Still carrying the `nil` main, named by that rider:** `probe_arc278_then_operand_wall.wat.bad`,
`probe_arc278_match_arm_body_bad.wat.bad`, `probe_arc278_query_type_safe_typo.wat.bad`,
`probe_arc278_rete_defn_*.wat.bad`, `probe_construction_*.wat.bad`. **This is a sweep, and it is
bigger than any one strike** — every one of those probes has a half that cannot fail.


⚠ **Found by answering the builder's question "you found an issue with our type checking?" about D7
— and the honest first answer was NO.** D7's parametric erasure is an engine bug; the checker
correctly refuses the unsound comparison. This is a different, broader gap that the question
uncovered.

⛔ **Three probes before this one were VACUOUS and I read them as findings.** A `sed` rename left
`collect-rules :t2` pointing at an emptied namespace, so zero rules compiled — and
`FireOutcome::Fired` still reports Fired, because it means *the fire completed*, not *a rule
produced a fact*. The well-typed control is what makes this row evidence rather than another
plausible number.

| id | site | what | found by |
|---|---|---|---|
| ~~**D1**~~ ✅ `2733b9bd9` + residual `f22704f1f` | `validate/typing.rs:231` | D1 made the misspelled variant REFUSE; the refusal then **named the wrong thing** — `UnknownField`, *"has no field `:evt::G::Hii`; available fields: [k, grade]"*, pointing the author at FIELDS for a VARIANT typo. Driven: core has the same blind spot (*"expects keyword; got `:evt::G`"*, `remedies []`), so **agreement with core was the wrong target** — naming the mistake is. | ✅ `#wat.rete/UnknownEnumVariant` — *"`:evt::G` has no variant `Hii`; available variants: [Hi, Lo]"*. `keyword_constant_segment`'s `_ => "keyword"` was the **fifth catch-all** in this arc; now a named three-state `KeywordConstant`. ⛔ **My own sketch would have shipped a false message** — the guard sat after the arity-0 arm and inherited the TAGGED case, emitting *"has no variant `Hi`; available: [Hi]"*. Split on the discriminator, not a symptom. ⚠ **Bare tagged variant used as a value keeps the wrong remedy** — a third mistake, cut and PINNED with a golden. See `strike-variant-diagnostic/`. |
| **D2** | `hash_join.rs:296` | `right_idx[J]` has two writers, only one maintains `right_idx_n[J]`. On `filter → HashJoin(a) → HashJoin(b)`, round 2 pushes without bumping, 3.7 re-appends the same element → doubled buckets, duplicate tokens. `seen_insert` hides it in the fact set; surfaces as doubled `:accumulate` counts and query rows | `sequi` |
| ~~**D3**~~ ✅ `057f9d494` | `expr_ir/eval.rs:405` | **a silent WRONG ANSWER through the public surface**, driven: the fixture fence answers 1 hit; two args for one param at slot 1 → ACCEPTED, **0 hits** (the surplus overwrote the declared parameter, since it is written into the slot whose NUMBER equals its POSITION). Past the frame → silently dropped (2 hits). Missing → `unbound symbol: slot 1`. Class A a fifth time: `lower_expr` builds `CallUser` from `lower_args` + `lower_rete_defn` without comparing them. | ✅ refused at `exec_program_on` — the one place args and params meet, downstream of the wire, the lowering and all four HOF heads — and the surplus branch **deleted**, so the loop is total by construction. **A sixth import wall affirmatively cut**: it would turn every probe green while the executor still held no invariant. 5 arms + 2 controls. ⚠ the anti-vacuity control is load-bearing: refusing EVERY call leaves the untampered-fixture control green. See `strike-calluser-arity/`. |
| ~~**D4**~~ ✅ `073546093` (code, swept) + rune | `expr_ir/eval.rs:85-126` | `EXEC_SP` is inert — the `RefMut` spans `f`, so every nested frame takes the `Err` arm and `start` is always 0. Doc holds BOTH claims: `:96` *"nested calls stack"* (false) and `:99` *"the `Err` arm is a correctness path"* (true). A panic through `f` strands `len` slots — driven **cumulative and unbounded**, (8,8) → (16,16) → (24,24) — no `Drop` guard | ✅ **the cursor is DELETED, not guarded** — a `SpGuard` would restore a `start` that is *provably* 0, so it would guard a constant. Deletion makes the strand **structurally impossible** rather than cured, dissolves the TLS-teardown trap (no `Drop` at all) and is *less* hot-path work. ⭐ **Licence measured, not argued:** cursor + correct guard + `assert_eq!(start, 0)` held **1508/1508**, and flipping it to `999` REDs — the assert runs, `left: 0`. ⭐ **The single-panic trap was real:** under mutation 1 the probe REDs at **round 2**, not round 1 — one panic passes on the bug. ⛔ **My brief cited the wrong file for `ArmLease`** (`kernel/fire/rules.rs`, which has zero of it; it is `kernel/arm.rs:829`). See `strike-exec-sp/SCORE.md`. |
| ~~**D5**~~ ✅ | `validate/mod.rs:774` (⚠ the row said `:747`) | `match` refused in `:then`, byte-identical expression accepted in the `where` fence. `walk_nested_constructors` cannot tell a match ARM from a CALL; survives only by arity coincidence. | ✅ the walker now recurses into the match **scrutinee** and each arm's **BODY**, never an arm's **PATTERN**, keyed through `resolve_core_name` so BOTH spellings are covered. ⭐ **The enumeration came first and disconfirmed the wider class**: `let`/`fn` bind in **Vectors** so the walker returns before reaching them, and `cond` clauses have a call form at `items[0]` — `match` is the only List position where a bare variant keyword means something other than a constructor. One form, not three. ⛔ **My three mutations all passed the naive rete-only key**; the rider added a fourth that reddens it. The banked repro now LOADS and its `red-by-design` rune is retired — which returns it to the docs load-check, and that IS the regression gate. See `strike-match-arm-is-not-a-call/SCORE.md`. |
| ~~**D6**~~ ✅ | `step_payload.rs:139-155` (⚠ the row said `:143`) | explain payload silently DROPS every keyword/enum-operand constraint (`sym = None`, and `value_to_ast_literal` has no `Value::Enum` arm), while its doc claims the constraint list is complete. | ✅ **driven first**: a rule with two constraints produced a payload with one. ⛔ **The row names two causes and only ONE fires** — `resolve_operand` with `sym: None`; `value_to_ast_literal`'s missing arm never executed, but was waiting one line behind. Both fixed; **mutation 2 (revert only the arm) REDs**, which is what proves a two-gate cure. Part 2 landed as a positional marker whose head is deliberately **not** a `RETE_OPS` row, so `constraints.length` always equals the condition's constraint count and an omission cannot be evaluated by mistake. ⛔ A third `continue` I briefed **could never fire** — `Constraint` has one guarded producer; deleted, with a bidirectional gate installed. See `strike-explain-drops-a-constraint/SCORE.md`. |
| ~~**D7**~~ ✅ `9a8665d8f` | writers: `fire/delta.rs:100` (push) + `fire/pass/alpha.rs:130` (replace) — ⚠ the row said both were in `alpha.rs:85-98,129-132`; they are in TWO files | two writers of `wm.alpha[aid]` in one pass. Row called it a shape finding, *"not reached from the insert door"*. | ⛔ **DRIVEN LIVE 2026-09-02: `native=2 oracle=3`.** Three facts in, three derived facts expected; native produces TWO while `fire-rules$oracle` on the identical staged session produces three — **a derived fact is silently dropped**. TRIGGER: **parametric records erase their type argument into one class** — `(:d7::Box :- [T] [k <- i64  v <- :T])` yields one runtime class whose instances differ in PACKABILITY, so `Box[i64]` batches and `Box[String]` falls to the push path; `arm.rs:334` files both under one `aid`; the batch's `insert` discards the push, and `d_alpha` then indexes different elements. ✅ **CURED by class-uniform batching**: a class batches only if EVERY fact of it packed, otherwise all of them activate in fact order — one writer per `aid`, ordering preserved by construction. `native=3 oracle=3`. Chosen over declared-schema packability (needs a `TypeEnv` in `FireSession` AND is strictly more conservative) and over a non-replacing writer 2 (`d_alpha` holds indices; appending reorders). Cost +1.9% median, 27 samples, overlapping quartiles; the batch itself is worth 30.8%. Gated twice — a 7-arm native-vs-oracle differential over the erasure seam comparing KEY SETS not counts, plus a census-counter gate for the decision itself, because batching-vs-activating is correctness-invariant and the differential structurally cannot see it. Repro: `wat-scripts/scratch-pad/d7-two-writers-one-alpha.wat`. See `strike-two-writers-one-alpha/SCORE.md`. |
### ~~D9~~ ⛔ WITHDRAWN — THERE WAS NO GAP; I DID NOT CHECK THE CONVENTION THAT ALREADY EXISTS

Raised 2026-09-02 as *"the scratch-`.wat` doctrine has no home for a must-fail probe"*. **Withdrawn
the same day, at the builder's correction.** The home exists and is large:

> `.wat.bad` is for tests that ensure a file fails correctly. `wat-scripts/scratch-pad/` is for
> throwaway used *while building*. A test's fixtures are **adjacent `.wat` / `.edn` / `.wat.bad`
> files next to the test that drives them.** — builder, 2026-09-02

Measured after the correction: **271 `.wat.bad` fixtures, 361 `.edn` goldens**, and `*.wat` **does not
match** `foo.wat.bad`, so a must-fail fixture is invisible to `wat_scripts_fixes_load.rs` by
construction. There was never a conflict to resolve. D5's own cure had already used the convention
correctly (`probe_arc278_match_arm_body_bad.wat.bad` + `.edn`).

⛔ **Two failures of mine, and the second is the worse one.** I asserted a gap in the repo's doctrine
without grepping for the convention — against a 271-file population sitting in `tests/`. And I cited
`harness-experiri/README.md` as *"recording the same tension independently, which is the tell that
this is structural"* **without reading the paragraph**. It says something narrower and partly wrong
(true of `.wat`, false of `.wat.bad`); corroboration I did not check was used to upgrade a guess into
a structural claim.

### ~~D8~~ ⛔ WITHDRAWN — IT WAS CURED TWO DAYS BEFORE I ROWED IT

Rowed 2026-09-02 as *"a driven L1 with no row to its name … it sat driven, reproduced and unrowed
for three days, which is precisely how a finding dies."* **Withdrawn the same day. It had been fixed
on 2026-08-31 by `17fc5fb3e`, "the fence and the executor share one head-space".**

Driven before withdrawing — **both repros now print `"fired"`**, including `experiri-acc-head.wat`,
the one the harness README says refuses. `expr_ir/mod.rs:972` now opens with
`if let Some(op) = rete_op_index(head)`, the minted-row ladder, carrying its own driven evidence and
its rejected alternative (tightening the fence, refused because it would delete a working
capability).

⛔ **I rowed a cured defect from a cast report I never drove**, and rowed it *loudly* — the sentence
about how a finding dies was itself the error. The harness README is a **dated record of one cast**,
and I read it as a live work list. Its §2 happened to still be live (that became D5); its §1 was two
days dead. **The tell was available and free: run the repro.** The README now carries a banner saying
so.

### ⏭ D2 — DRIVEN 2026-08-31. The code asymmetry is REAL; no constructed input reaches it. **LATENT, not live.**

The orchestrator owed this row a drive and has now paid it. **The result is a bounded negative, and
"I could not construct a trigger" is NOT "there is no trigger."**

**THE CODE DEFECT IS REAL, re-verified at HEAD `16f504e14`.** `right_idx[J]` has two writers and
only one maintains the high-water mark the other reads:

| writer | maintains `right_idx_n`? |
|---|---|
| `keyed_join_persistent` (`fire/mod.rs:776,799,815`) | **yes** — reads `already`, appends the tail, writes back |
| `hash_join_delta` (`pass/hash_join.rs`) | **no** — the fn has ZERO mentions of `right_idx_n`; it is not even a parameter |

**THE SHAPE IS CONSTRUCTIBLE, and was measured rather than assumed** — decoded from a real
`Export` (`:a`lpha `:j`oin `:t`est `:h`ashjoin `:p`roduction):

```
0:α(A) → 1:RootJoin → 2:TEST → 4:HASHJOIN(a) → 6:HASHJOIN(b) → 7:Production
         3:α(B) ──────────────↗   5:α(C) ────────↗
```

That is `filter → HashJoin(a) → HashJoin(b)` exactly. ⚠ **The first attempt at this row asserted
the shape without measuring it** — the same defect class as everything else in this cast.

**WHAT WAS DRIVEN, and found nothing:**

| attempt | native | oracle | rows |
|---|---|---|---|
| C derived mid-fixpoint, fact-count observable | `[1 1]` | `[1 1]` | — |
| A,B,M derived round 1; C from M round 2 (staggers b's right side behind a's beta) | `[1 1 1]` | `[1 1 1]` | 1 |

The third column is a query whose `:when` **mirrors the join chain**, so it yields one row per
TOKEN rather than per fact — the observable `sequi` predicted, since `seen_insert` dedups the facts.
It did not double. The oracle is a trustworthy reference again as of `16f504e14`, so agreement here
is evidence rather than two engines sharing a bug.

**WHAT THIS DOES AND DOES NOT ESTABLISH.** It does not disprove `sequi`'s trace: reaching the stale
`already` needs `beta[a]` empty at pass-3 time in one round and a derived fact landing in `alpha_b`
the next, and neither staggering above provably produced that exact interleaving — **nothing here
inspected `right_idx_n` directly.** A Rust-level probe that reads the counter after each round
would settle it and is the honest next step if this row is ever reopened.

**DISPOSITION: the structural fix is worth taking on its own merits, reachability aside.** `sequi`'s
proposal removes the class rather than the instance: make `right_idx` a newtype whose only insertion
verb is `index_upto(join_id, &[Element])`, carrying the high-water mark inside, so **no writer can
append without advancing it and a fifth writer added later inherits the guarantee.** That is a small
`extirpare` win with no behaviour change — not an urgent correctness strike.

#### ⛔ AND THE ANSWER TO "IS THERE CODE TO REAP HERE?" IS **NO** — measured, and the reason sharpens D2

The builder asked whether `indexed_n` is dead machinery. **It is not. It is a correctness guard**,
and reaping it would introduce exactly the defect D2 describes.

`keyed_join_persistent` (`fire/mod.rs:800-816`) does
`right_idx.entry(join_id).or_default()` — which returns the **EXISTING** bucket map on a second
call — then `ridx.entry(k).or_default().push(el)`, which **appends without clearing**. `already`
is the ONLY thing preventing a second call from re-pushing every element it already holds.

**BUT IT GUARDS A CASE NOTHING MEASURED REACHES.** A temporary probe over all four branches
(`already == 0` non-empty · `0 < already < len` · `already == len` · `already == 0` empty):

| workload | full | **INCREMENTAL** | skip | empty |
|---|---:|---:|---:|---:|
| 423 rete tests | **35** | **0** | 0 | 0 |
| grid `accum [50 200]` | 0 | 0 | 0 | 0 |
| grid `deep-cascade [10 100]` | 0 | 0 | 0 | 0 |
| grid `strat-neg [6 500]` | 0 | 0 | 0 | 0 |

**Every call is a first index. `indexed_n` is written every time and its stored value is never read
back as anything but 0.**

⚠ **THE PROBE'S FIRST VERSION HAD A BLIND SPOT AND REPORTED SILENCE AS A FINDING** — its `if/else`
chain covered three of the four cases, so `already == 0 && empty` fell through printing nothing,
making "never called" and "called with nothing to index" indistinguishable. Caught and re-run. Same
class as the rest of this cast, committed by the hand auditing it.

**SO D2's DISPOSITION IS NOW PRECISE:** `hash_join_delta` writing `right_idx` without advancing the
mark is a **live hole in a guard that has never yet had a second chance to matter.** It is latent
because the guarded case does not arise — not because the code is sound. `sequi`'s newtype (one
insertion verb, mark carried inside) is the right fix precisely because it makes the guard
unbreakable by a future second call or a fifth writer, and it must NOT be simplified away.

---

## ✅ CLASS E — error shape and diagnostics. **CLOSED 2026-09-01** (E1, E2, E3, E4, E5).

> ⏭ **NEW (2026-09-01, from E4):** `#[to_edn(transparent)]` — a variant directive for the `ToEdn`
> derive (~30 lines, write-only) that would let a newtype variant keep its inner tag. Wanted only if
> the pre-E4 error wire form must be preserved; nothing currently asserts it.

> ✅ **THE NESTED-CONSTRUCTOR WALL IS WIRED (`c0c883082`)** — the hole E1+E2 pinned. `defrecord`
> lowers record constructors to `:wat::core::kwargs-construct` before the wall runs, so all four of
> `walk_nested_constructors`' error kinds were unreachable; all four are now driven with per-arm
> mutation separation. ⚠ **`RhsPositionalConstructionRetired` is NEW ENFORCEMENT, not restored
> parity** — driven, rete fire never reached the retiring dispatch, so nested multi-arg positional
> construction compiled and fired. Accepted on a zero-use corpus sweep; the false doc is corrected
> at the site.
>
> ⏭ **NEW ROWS from that strike:** (a) `RhsMissingFields` / `RhsArityMismatch` render the NESTED
> operand as though it were the inserted fact — messages written for the top-level producer, reused
> verbatim. (b) The positional prime `:T'` reaches the wall **un-lowered** and `types.get` fails on
> the suffix — still silently unvalidated, zero corpus uses.

> ⏭ **NEW (2026-09-01, found during E5):** nothing gates `file:line` citations in comments.
> `no_stale_path_in_doc` checks **paths, not lines**, so every edit above a cited line rots the
> citation undetected — two were found and refreshed in `arm.rs` during E5.

| id | site | what | found by |
|---|---|---|---|
| ~~**E1**~~ ✅ `1efb42fc7` | `validate/typing.rs` | doc promised *"the span of the FIELD rather than the clause"*; **both** callers passed `clause.span()`. Driven: caret **cols 31–76** (46 chars) where the keyword sits at **col 65 len 10**. | ✅ **ONE producer**, `check_field_kw(field_kw: &WatAST, …)` — a bare `Span` no longer compiles at any call. Carets proven on three paths with hand-anchored `(line,col,end_col)` goldens; **cols 65–75**. See `strike-field-span/`. |
| ~~**E2**~~ ✅ `1efb42fc7` | `validate/mod.rs` | ⚠ **THE ROW WAS WRONG ON ITS DETAIL** — the arm PASSES `bad.span` and its doc is accurate; two arms were collapsed into one row. The row's SHAPE was the finding: the dead one documented better behaviour than any live one. | ✅ dead arm **driven** before removal, not deleted on a reading. ⛔⛔ **AND TWO OF FOUR PRODUCERS WERE DEAD, NOT ONE**: `defrecord` lowers record constructors to `:wat::core::kwargs-construct` before the wall runs, so `walk_nested_constructors`' FOUR error kinds are all unreachable — driven, a nested undeclared field is `ACCEPTED-UNVALIDATED`. Its live enum-variant sibling is why the walk looked exercised. **Pinned, not fixed** — a wall-reachability strike across four kinds. |
| ~~**E3**~~ ✅ `76e221bbb` | `signal.rs` (post-E4 `ReteCeiling`) | three doc blocks stacked onto ONE variant — Rust accumulates onto the next item — so `RuleSetMayNotTerminate` and `FixpointRoundCapExceeded` rendered with **no doc at all**. ⚠ **THE ROW'S OTHER CLAIM WAS WRONG**: the *"names an action the author can take"* justification lives at `outcome.rs:226` and was correctly placed all along. | ✅ split onto four variants, **verified in rendered HTML**. `signal.rs` 9 broken intra-doc links → 0 (only 2 were E4's; **7 were older**), tree-wide 50 → 41. ★ **Class cure**: `no_new_broken_doc_link.rs` runs rustdoc and freezes the rest as **34 NAMED `(file,target,sites)` keys** — a ratchet both ways, six arms driven incl. a genuinely held cargo lock, bounded at 300s so a held lock is a **named red, not a hang**. See `strike-doc-attribution/`. |
| ~~**E4**~~ ✅ `452953cb9` | `outcome.rs:103,161,213` | the three converters' `_ =>` left the wall's completeness to a hand-maintained `CEILING_VARIANTS` list. **No live gap** — driven: exactly four ceiling variants, exactly four listed, and the three owned sets are DISJOINT so the catch-alls were load-bearing. The defect was that nothing forced the FIFTH to be considered. | ✅ `RuntimeErrorKind::ReteCeiling(ReteCeiling)`, matched exhaustively, cross-converter arms **written** not defaulted. A fifth variant now fails to compile in **FOUR** places — three converters **plus `fmt_with_span`**, which has no wildcard, so it must be routed AND given a message. ⚠ **The derived EDN tag changes** (`#wat.runtime/ReteCeiling {:ceiling …}`) — unavoidable, prose messages byte-identical, nothing asserts the tags; accepted and recorded at the enum, with `#[to_edn(transparent)]` filed as its own strike. See `strike-ceiling-closed-set/`. |
| ~~**E5**~~ ✅ `c9cdd9d32` | `fire/mod.rs:1036`, `rules.rs:658` | both sites stamped `rust_caller_span!()` while the real wat span sat ONE FRAME UP, in hand and already spent on the same fn's arity refusal. `span_substitution_justified` could not see it: its "no span param" test is a **syntactic proxy** for its stated principle (*"never about the absence of a choice"*), and the choice lived one frame up. | ✅ threaded `span: &Span` into both fns — **the cure IS the guard**: it brings both bodies inside the EXISTING lint's view, so a future `rust_caller_span!()` there reddens it. Mutation driven on BOTH bodies. Lint **not widened** — measured first: 494 span-less sites under `src/`, 69 in rete, mostly leaves, **recorded with its instrument** in the lint's doc. ⛔ My own 534/71 was unreproducible and load-bearing; my caller table named a fn that never calls it. See `strike-refusal-span/`. |

---

## CLASS F — the description layer. **Builder's directive, 2026-08-30: greppability over correction.**

> *"counts are always wrong, every time... we must make our file suitable for greps for on the fly
> counting as necessary"* · *"more lints are almost always better"*

**F0 — THE RULE.** A number in prose is replaced by **the command that derives it**, not by a
corrected number. Every rotted claim this cast found was a count that was true when written:
*"eleven session fields plus three"*, *"55 rows × 2 positions"*, *"three callers"*, *"the dozen
non-memory fields"*, *"four cells"*, *"a 74-row table"*, *"nine"* (`RoundScratch` has ten),
*"exactly six write sites"* (thirteen). Correcting them buys weeks. Deleting the claim is the fix.

**F1 — the five lints this cast earned**, each with instances already found:

| lint | instances |
|---|---|
| ~~backticked identifiers must resolve~~ ✅ `2c7200802` | **33**, not 7 — measured by instrument, incl. `head_is_boolean_rete_predicate` — the comment guarding a silent `_ => None` on the fix-list F path. Also `token_element_compatible`, `DidNotDiscriminate`, `CoreKind`, `rule_rhs_cache`, `ref_alpha_of`, `invoke_wat_compile` |
| ~~bare `*.rs` filenames must resolve~~ ✅ `2c7200802` | **27 stale of 244 cited** — `validate.rs`×9 and `expr_ir.rs`×6 were an unseen cluster from the same split. `kernel/mod.rs:4` *"Tests are `tests.rs`"* — stale the day it was written. `no_stale_path_in_doc.rs` only extracts tokens containing `/` |
| ~~`rune:perspicere` / `rune:purgare` closed vocabularies~~ ✅ `9d4b68088` | ⛔ **THE ROW'S FINDINGS DID NOT SURVIVE.** Both vocabularies were **already authored in the ward spells** — the row (and my stone) checked `CONVENTIONS.md`, found nothing, and called them undefined; a missing COPY is not an absent definition. And `perspicere`'s `read-once` is **not** falsified: the *"alias would be a mumble"* clause is shared boilerplate at **18 sites across 4 files**, not a per-site argument, and against the authored definition the census.rs six are **correctly labelled**. ✅ Both vocabularies now copied into CONVENTIONS.md with provenance and gated by `no_unknown_ward_rune.rs` (7 arms mutation-proven, wider scan than the `sequi` gate). ⚠ **STOP-3 CONFIRMED and sharper**: no trait at any `trait-contract` site, the two real trait impls are labelled `public-api` — the categories look **swapped** — and no category covers *structural completeness*, a gap in the **ward's** vocabulary. See `strike-rune-vocabularies/`. |
| ~~`MINIMUM of` header may not co-occur with `/= r`~~ ⛔ **ALREADY SATISFIED — struck 2026-09-01.** C1 itself shipped `tests/lint/minimum_label_matches_its_estimator.rs` (446 lines, `119214aef`), whose header reads *"a table that says `MINIMUM` may not compute a MEAN"* and which names the `/= r` shape by name. Population today: **9 files say `MINIMUM of`, ZERO divide by rounds** — pattern verified sound against C1's own removals. The row was filed 2026-08-30 and the cure landed 2026-08-31; nobody struck it. **F0's thesis exactly: a claim that was true when written.** |
| ~~non-vacuity guards on walking gates~~ ✅ `58a10e1f8` | ⚠ **the row's count was wrong, and so was my audit grep's** — the instrument reports **24 in scope, 19 undeclared** (6 of those already had a real guard). **Driven first: NO gate is vacuous today.** `every_walking_gate_declares_non_vacuity.rs` now requires a declared guard — marker+assertion, or a rune naming the mechanism. ★ It caught itself: its own `///` doc parsed as a rune and it was one run from vouching for itself. |

**F2 — rotted claims inside `src/` and the arc** (do these WITH F0, not as corrections):

- `NEXT-STRIKES:1491,1512` — both TRACKED DECISIONS premises expired; the tally at `:1283`
  contradicts them. The section banning two-places-per-row committed it again. *(`exigere`)*
- `rust_deps/cache.rs:70` — cites heading *"exigere — the cache panic conversion"*; grep finds it
  **only in that source line**. *(`exigere`)*
- `purity.rs:216` — *"nothing enforces that"*; `mod completeness_gate` is at `:2093`, same file. *(`exigere`)*
- `DESIGN-STONE-4b:68` — *"its own future stone (let need reveal)"*; `delta.rs:391` says *"THIS IS
  THE STONE 4b DEFERRED … The need revealed."* Forward edge, no back edge. *(`exigere`)*
- `DESIGN-STONE-gather-no-snapshot:53` — forbids what `delta.rs:321` does; superseded 2026-08-19,
  neither earlier stone annotated. *(`conferre`)*
- **83 of 207 stones name `src/rete/kernel.rs`**, deleted 2026-08-20. `no_stale_path_in_doc.rs`
  scans `src/rete` only, so the impl side is spotless and the stones rotted. *(`conferre`)*
- `reachability.rs:820,830,832` coverage prose; `:568-578` an orphaned doc block that merged onto
  `uniform_call`'s rustdoc; `:419,446` "four cells" vs six. *(`intueri`)*
- ~~`wat-scripts/fixes/rete-where-per-type-spelling.wat:80,96`~~ ✅ `f4800ef97` — the row was right and
  the drive found the doctrine claim underneath: **`CLAUDE.md`'s "all wat stays correct, always" was
  FALSE**. Type-checking is not resolution — an invented head in an unforced `def` type-checks and
  RUNS. The two rename rows are **DELETED (41→39), not re-pointed**: a pure head-rename to
  `mapv`/`filterv` does not compile, because the eager form needs a different container. New gate
  `rete_names_in_wat_scripts_resolve.rs` resolves every `:wat::rete::` name in **code**; **prose is
  exempt by design** (`foldr`/`nth` are accurate history) and a deliberately unminted name carries a
  per-name rune. See `strike-phantom-rete-names/`. *(`cernere`)*
- ~~`remedy/retirement.rs` — zero `:wat::rete::` rows~~ ⛔ **STRUCK 2026-09-01, THE ROW'S PREMISE DOES
  NOT HOLD.** Builder's ruling: *"the retirement stuff is for in-flight work… it's meant to educate
  during migrations… there's no migration happening for rete now?"* — correct, and the module's own
  doc is the authority: entries are appended **by each HARD CUT stone at ship time**, and
  *"future-vapor entries are forbidden — only shipped retirements appear here… premature entries
  deceive the substrate."* It is an append-at-ship-time record, **not a completeness surface**.
  9 entries, all `:wat::core::`, including `foldr` — working as designed. And there is no rete
  population to serve: the only rete hard cut is `:wat::rete::core::foldr` (`30be59b68`), whose own
  message says *"the retirement had to BUILD its own refusal"* — it shipped WITH one. `map`/`filter`
  were never retired at all (`07073c091`: *"the surface takes mapv/filterv … not the lazy heads"*) —
  a choice at minting. **Adding rows now would be the deception the module forbids.** Kept struck so
  nobody re-files it. *(`cernere` — the ward saw an empty column and inferred a gap.)*

⛔ **F3 — THE REFERENT IS GONE (recorded 2026-09-01).** This row says the 70 L2 *"live in the ward
reports"*. **There are no ward reports for this cast.** The 2026-08-25 vigilia left
`GEN-VIGILIA-2026-08-25.md` (672 lines, a durable artifact); the 2026-08-30 cast left **only this
work list** — the 19 ward returns were subagent messages and evaporated. `grep -rn 'per-pair map' docs/`
finds exactly two hits: this line and the breadcrumb that copied it.

**So F3 is a set of LEADS, not recorded findings.** Each named cluster is specific enough to
re-derive — and must be re-derived and re-measured, never inherited. Treating the paragraph below
as a work list would be acting on 70 findings nobody can read. **The clusters, as leads:** Highest-value clusters:
`temperare` ×7 (all with measurement plans — `join_extend`'s three per-pair map probes;
`alpha.rs:82,94` hashing the class FQDN twice per fact in the subsystem whose own stone measured
that at 3.26 ms), `perspicere` ×5 (aliases), `partire` ×4 (`fire/mod.rs` → `gather.rs` + `query.rs`;
`compiled_cond.rs` at its own `// ─── The executor ───` banner; `stratify.rs` → `termination.rs`;
`expr_ir/eval.rs` → `ops.rs` — all four with verified one-directional seams).

---

## ⏸ DEFERRED BY BUILDER'S RULING, 2026-08-30 — docs outside `docs/arc/`

> *"our docs outside of arcs are very out of date — we've just been grinding on code correctness —
> our compiler and runtime provide coordinates and prompt injections as errors for corrections…
> i'm less keen on truing up our docs and more keen on ensuring our code is an exemplar; docs come
> after the code churn is satisfied."*

**Owner: builder. Re-read: when Classes A–E close.** Bounded, not promised — `exigere`'s rule.

`circumspicere` L1: `README.md`, `docs/USER-GUIDE.md`, `docs/README.md`, `docs/INTENTIONS.md`
contain **zero** hits for `rete` / `defrule` / `rules engine`. The module tour omits `src/rete` —
42,012 lines, the largest module in `src/`. `README.md:148` claims *"725 Rust + ~58 wat"* against a
5,165 floor; `:150` claims *"25 integration suites under `tests/`"* where there are **0** top-level
`.rs` files (19 subdirectories); `:59` claims ten arcs in 2026-03..04 where `docs/arc/2026/` holds
`04 05 06 07`. All four verified by the orchestrator on the disk.

**The reason this is deferred and not struck:** the correction mechanism for a wat author is the
compiler's own located diagnostic, not the README — so a stale README costs a reader orientation,
not correctness. That is a real ruling with a real ground, and it holds only while the ground does.

---

## Verification status

Every L1 above was weighed against the disk by the orchestrator, not credited to the report.
**Verified by direct read or drive:** A2, A3, A4, A5, C1 (git blame), C2, C3, C5, D1 (mechanism),
D5, E1, E3, F2's `exigere` four, `partire`'s two corrections to our own record, and all of
`circumspicere`'s README claims. **Passed through on the ward's citation, not independently
re-derived:** the remainder. A row's evidence is its ward's report until someone re-reads it.

⚠ **Two of `partire`'s corrections are to numbers in THIS arc's own record**: `arm.rs` is ~1,251
production lines, not the 593 the breadcrumb states — and that 593 was itself recorded as a
*correction* of an earlier wrong figure. `reachability.rs` is **0** production lines, not 1,917:
`src/rete/mod.rs:86` wraps the whole file in `#[cfg(test)]`, invisible to any per-file scan —
including `scripts/doc-coverage.sh`, which counted all 1,917 as production in the figures this arc
has been quoting.
