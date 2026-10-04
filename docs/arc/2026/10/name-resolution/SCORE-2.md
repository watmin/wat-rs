# SCORE-2 — the lookup's discarded clone, and the parent walk measured

Executor's clone: `/var/tmp/wat-rs-names`, branch `the-little-wat`, continuing from part 1's
`41352b548`. Brief: `BRIEF-2.md` at `75cf1aac3`. Baseline floor: part 1's
`/var/tmp/wat-rs-names-logs/floor-after-r1.log` (5,405 of 5,405, confirmed by the orchestrator's
own re-run).

Write this file AS YOU GO.

## P1 — no discarded clone

Status: DONE.

### The edit

`src/value/environment.rs`'s `lookup` (around line 210): `entry.value.provenance().clone()`
(an upfront clone of the WHOLE stored provenance, unconditionally, then thrown away on every
arm except `RuntimeBuilt`) replaced with matching `entry.value.provenance()` BY REFERENCE. The
`RuntimeBuilt` arm now clones only what it keeps (`call_span.clone()`; `producer: &'static str`
is `Copy`, dereferenced with `*producer`, no clone needed). Every other arm (`Unknown` /
`Literal` / `SymbolBound`) builds `Provenance::SymbolBound` straight from `entry.binding_span`
and `head_span` — no intermediate clone of the old provenance at all.

STOP-2 checked and does not apply: `Provenance` and `Span` both use plain `#[derive(Clone,
Debug, ...)]` (confirmed by reading `src/value/observe.rs:22` and
`crates/wat-reader/src/span.rs:51,69`) — no custom `Drop` impl, no side-effecting `Clone` impl
(no counter, no hook) on either type anywhere in the tree (grepped `impl Drop for Provenance`,
`impl Clone for Provenance`, `impl Drop for Span`, `impl Clone for Span` — zero hits beyond the
derives). Nothing outside `lookup` could have been reading a side effect of the clone this row
removes, because the clone never had one.

`cargo build --release`: clean on the first attempt.

### Re-measurement (same method as R0/R1)

Checksums, every run (functional + 3×perf-record + 3×perf-stat per workload): conj-build
`499999500000`/`499999500000`; call-heavy `500013696418` — unchanged. **Gate 3 satisfied.**

**Instructions:u / cycles:u, 3 runs each, taskset -c 2, P1 vs R1 (part 1's post-R1 numbers):**

| workload | metric | R1 mean | P1 mean | change | R1 spread | P1 spread |
|---|---|---|---|---|---|---|
| conj-build (N=1e6) | instructions:u | 28,380,056,129 | 27,801,363,098 | **-2.04%** | 0.06% | 0.02% |
| conj-build (N=1e6) | cycles:u | 13,976,872,988 | 13,873,579,043 | -0.74% | 1.38% | 3.85% |
| call-heavy | instructions:u | 54,113,393,785 | 52,931,203,432 | **-2.18%** | 0.67% | 0.90% |
| call-heavy | cycles:u | 23,244,187,186 | 23,246,153,196 | +0.01% (noise) | 0.46% | 6.78% |

Against the orchestrator's prediction ("P1 takes 2–5% of instructions off call-heavy and less
off conj-build"): call-heavy landed at the BOTTOM of the predicted range (2.18%, just inside
2–5%); conj-build (2.04%) again came in essentially equal to call-heavy, not clearly "less" —
same pattern as R1's prediction miss in the same direction. Cycles:u showed no reliable signal
either way — conj-build's small drop (0.74%) and call-heavy's flat reading (+0.01%) are both
inside or close to their own noise floors (3.85%/6.78% spread on only 3 runs); this row's win
is real in instructions, not demonstrated in cycles.

**What happened to `Environment::lookup` and `drop_glue::<Provenance>`, by name, as the brief
asked:**

| workload | symbol | R1 self-time | P1 self-time | direction |
|---|---|---|---|---|
| conj-build | `Environment::lookup` | 5.20% | **4.49%** | down |
| conj-build | `drop_glue::<Provenance>` | 2.66% | 2.71% | flat (noise) |
| call-heavy | `Environment::lookup` | 7.90% | **6.72%** | down |
| call-heavy | `drop_glue::<Provenance>` | 4.33% | 4.64% | flat/up (noise) |

`Environment::lookup` itself fell on both workloads — the self-time SHARE dropped on a total
pie that ALSO shrank, so this is a real, double-confirmed win (approximate absolute-instruction
read, multiplying each row's own share by its own 3-run instructions:u mean: conj-build's
`lookup` cost ≈1.476B→≈1.248B instructions, **-15.4%**; call-heavy's ≈4.275B→≈3.557B,
**-16.8%** — both figures are cross-run estimates, not a single precise measurement, since the
% share comes from one `perf record` run and the total comes from a separate 3-run
`perf stat` mean; read as "clearly down," not to two significant figures).

`drop_glue::<Provenance>` did NOT fall — it is flat within noise on conj-build and ticked up
on call-heavy. This is the honest, slightly counter-to-hypothesis result: removing the
clone-then-discard did not show up as a smaller `drop_glue::<Provenance>` bill. The likely
reason, read from the code rather than assumed: the ELIMINATED drop (of the now-never-created
discarded match scrutinee) was almost certainly getting INLINED directly into
`Environment::lookup`'s own self-time before P1 (same inlining pattern R2 named for FxHash vs.
SipHash) — so its removal shows up as PART OF `lookup`'s drop, not as a separate line that
shrinks. What `drop_glue::<Provenance>` still measures post-P1 is the cost of dropping
provenance values that are NOT the eliminated one: every `BoundEntry`/`TrackedValue` that is
legitimately still alive and eventually torn down (environment teardown, end-of-call cleanup)
carries a `Provenance` that must be dropped regardless of this row's edit. P1 did not touch
those, so their accounting is unaffected — and a workload with proportionally MORE such
necessary drops (call-heavy, which builds many short-lived single-binding environments per
tail-loop iteration) shows a slightly higher `drop_glue::<Provenance>` share after the pie
shrinks elsewhere, which is exactly what was measured.

### Gate after P1

`NEXTEST_TEST_THREADS=4 cargo nextest run --release` → **5405 tests run: 5405 passed (7 slow),
22 skipped (1624.678s). ALL GREEN** — same result as the baseline
(`/var/tmp/wat-rs-names-logs/floor-after-r1.log`). **Gate 2 (after P1) satisfied.**
