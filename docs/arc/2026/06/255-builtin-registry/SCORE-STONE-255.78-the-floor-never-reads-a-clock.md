# SCORE — STONE 255.78: the floor never reads a clock to decide

**Executor: a Sonnet subagent, working solo** (ran the floor itself). Drawn against local `main`
@ `628260fd9` per the brief (255.77, held unpushed — this stone lands first, one floor for both).
Commit: see the accompanying `git log` entry on `main` beside this file (committed locally, not
pushed).

## Per-row disposition

| row | file:line (original) | claim | witness built | result |
|---|---|---|---|---|
| 1 | `harvest_cost.rs:337` `harvest_wrap_split` | scan+wrap account for the combined pass | atomic counters inside the three timed closures themselves (`scan_matches_in_{s,h}`, `wrap_maps_in_{w,h}`), asserted equal | witness built, mutation-proved |
| 2 | `gather_probe_cost.rs:422` `drop_memories_cost_split` | `clear()` of the fire-scoped structures stays O(1) | allocation-BYTE delta (`crate::alloc_counter::thread_bytes`, the existing `CountingAllocator` instrument — no new counter) around the combined clear, at N=40,200 and N0=4,020; must be `<= 0` at both | witness built, mutation-proved |
| 3 | `fanout_cost.rs:328` `fanout_per_call_alpha_census` | production dominates hash-join on a fan-out workload | **FALSIFIED, not built** — see below | assertion dropped; replaced with a structural `hj_emitted == prod_derivations` invariant |
| 4 | `node_share_cost.rs:822` `node_share_where_cost_decomposition` | the (token×tid) set-probe loop is the majority rung | operation counts (`pairs` = tokens×tids vs `ntok`/`fire_reuse`/`fire_gathers`) replacing the ns ordering | witness built, mutation-proved |
| 5 | `accum_cost.rs:241` `accum_fire_phase_census` (fold) | fold has not regressed to the 68.49 ms rematch mechanism | new counter `accum:rematch` at `fact_holds_under`'s one call site in `fire/pass/accumulate.rs`, must read 0 | witness built, mutation-proved |
| 6 | `accum_cost.rs:250` (snapshot) | the snapshot `DESIGN-STONE-gather-no-snapshot` removed has not returned | `accum:snapshot-alloc-bytes` — `thread_bytes()` before/after `alpha_elements`, must read 0 | witness built, mutation-proved |
| 7 | `binding_repr_bench.rs:500` `token_bindings_representation_dominance` | array beats trie at cardinality 1 | pure speed claim | function (+ its now-solely-used helpers `bindings_extend_trie`/`bindings_extend_array`/`kv`) **deleted** from the test binary — `benches/binding_repr.rs` already carried a byte-identical, fully-diagnostic copy (landed at `ff65ae7d5`, which deliberately kept this one function in the test binary pending exactly this stone) |

### Row 3 — the brief's own premise, measured and falsified

The brief proposed re-pointing to "the per-phase counts the census already collects (mark pairs
or operations)". I added the narrowest counter the claim needed —
`census_count_n("hash-join:tokens-emitted", n_emit as u64)` at hash-join's two emit sites
(`fire/pass/hash_join.rs:300,412`) — and ran it against the existing `prod:derivations` counter.
**Measured: `prod:derivations` = 40,000, `hash-join:tokens-emitted` = 40,000 — EQUAL, not 2x.**
This fan-out is a 1-rule, 1-join, 1-RHS pipeline: every hash-join match produces exactly one
production, so the OPERATION COUNT is 1.00x regardless of which phase is slower. The real
~9-10x wall-clock gap this test documents is a PER-ITEM cost asymmetry (production pays
dedup-store + compiled-RHS execution; hash-join pays an index probe/emit) — a nanosecond fact by
construction, with no operation-count proxy. There is no `DESIGN-STONE` for this specific claim to
re-derive a different witness from (unlike rows 5/6), so per the brief's own row-7 fallback ("a
claim with no deterministic witness... the SCORE says why") the clock-based assertion is dropped.
The breakdown stays **printed** (information) rather than physically relocating to `benches/`:
`benches/*.rs` are separate crates linked only against `wat`'s PUBLIC API (confirmed against the
existing `perf_arc278_fire_baseline.rs`, which times the whole fire from OUTSIDE for exactly this
reason) — the `#[cfg(test)]`-only phase-census instrumentation this diagnostic reads is invisible
there, and exposing it publicly to relocate one print is a larger surface-area decision than this
stone's narrow-counter allowance covers. The two new counters are kept, backing a genuine
structural invariant instead (`assert_eq!(hj_emitted, prod_derivations, ...)`).

## The wall: `tests/lint/floor_never_reads_a_clock.rs`

Syntactic, single-file, per-FUNCTION-BODY (not per-file — the cost-test family reuses a tiny
alphabet of 1-2 letter names, `a b c d e f g h i j k l m n s t w`, for unrelated purposes across
sibling functions; a file-wide taint set false-positived on every unrelated reuse). Walks
`src/**/tests/**` + `tests/**`; classifies every byte as code/comment/string (escape-aware) so a
literal `(`/`)` inside an assert's own message string cannot desync the scanner; finds every
`assert!`/`assert_eq!`/`assert_ne!` call by balanced-paren scan.

**Taint seeds** (anywhere in a binding's RHS statement, or a `.push`/`.push_back`/`.fetch_add`
argument): `Instant::now(`, `.elapsed(`, `ns_per_iter(`, `elapsed_ns(`, and the `(_, ns, …)` /
`(_, ns)` phase-census tuple-destructure idiom this whole file family uses. Propagates through
simple and tuple (`let (a,b) = …`) bindings, with POSITIONAL attribution when the RHS is itself a
parenthesised tuple literal of the same arity (so `let (prod, hj) = (ns_of(..), ns_of(..));`
splits correctly) and a documented all-or-nothing fallback otherwise (an opaque call like
`of(RHS)` taints every destructured name — a stated false-negative-safer-than-false-positive
trade; see the file's own header for the full "what it cannot see" list, including a NAME
reference immediately followed by `.field` — excluded from counting as a reference at all, so a
struct that legitimately mixes a clock field with a count field under one binding does not poison
every sibling field).

**The liveness carve-out**: each top-level `&&`-conjunct is examined only if it contains a
top-level comparison operator (so a bare boolean flag or `.is_empty()`/`.all(|t| …)` call is out
of scope regardless of taint) and must match `IDENT > 0` / `IDENT > 0.0` (optionally parenthesised
/ cast) to be exempt; `assert_eq!`/`assert_ne!` on a tainted operand has no liveness form at all
and is always a hit. A co-located `// rune:lint(clock-verdict) — <reason>` (same window as
`no_rpds_rebuild_loop.rs`'s own exemption — anywhere in the macro call through the end of its
closing line) exempts a specific site.

**Mutation proof** (item 3): restored `harvest_wrap_split`'s deleted apportionment compare
(`h >= (s+w)*0.5 && h <= (s+w)*2.0`) — lint went red naming both conjuncts, verbatim:
```
src/rete/kernel/tests/harvest_cost.rs:378: assert!(…) — clock-derived conjunct `h >= (s + w) * 0.5` is not the liveness form `IDENT > 0` / `IDENT > 0.0`
src/rete/kernel/tests/harvest_cost.rs:378: assert!(…) — clock-derived conjunct `h <= (s + w) * 2.0` is not the liveness form `IDENT > 0` / `IDENT > 0.0`
```
Removed; lint returned to green, file byte-identical to its pre-mutation state (`diff` confirmed).

**Sites found outside the seven, and how each was closed** (first run: 115 hits; final: 0):

- **`node_share_cost.rs:778`** (`node_share_where_cost_decomposition`, same test as row 4) —
  `&& b > a && b > e` dropped from the non-vacuity assert. Structurally B⊇A (same loop count,
  B does strictly more), but that is a fact about the CODE two SEPARATE wall-clock loops cannot
  see — exactly the shape this file's own nearby "⛔ ADJACENT-RUNG MONOTONICITY IS NOT ASSERTED"
  block already found too noisy to assert for the sibling G..K ladder (red on the sixth
  consecutive drive, 2026-09-04). No count-based substitute exists (A and B run the identical
  loop count; a count cannot distinguish "built the env" from "built the env AND walked it"). The
  six `> 0.0` liveness atoms (the real non-vacuity guard) stay.
- **False positives from the detector's documented limits** (not genuine clock verdicts — exempted
  with `// rune:lint(clock-verdict)` and a reason naming the mechanism, all pre-existing code I
  did not otherwise touch): `node_share_cost.rs:147,182,305,322,351(×2),364,855(×2)` — this one
  600+-line function's shared pool of short/generic names (`tokens`, `v`, `n`…) swept several
  genuinely deterministic counts into the per-function taint set; `fanout_cost.rs:504,695,699,818`
  — `rhs_pairs`/`.rhs_pairs` destructured alongside a clock-derived sibling from an opaque
  `(ns, count)`-returning closure (`of`), which this detector's per-function/per-tuple pooling
  cannot split; `tests/lint/nested_program_starts.rs:574` — a census count destructured alongside
  a wall-clock `ms` from one `run_gate_on_tree()` tuple; `tests/process/lifeline_pipe_proof.rs:122,151`
  — a `libc::write`/`libc::read` byte count named `n`, pooled with an unrelated `elapsed` reuse of
  the same function.
- **STOP-3** (one site): `tests/process/doomed_child_boot_ack_does_not_hang.rs:87`
  (`elapsed < Duration::from_secs(2)`) — a genuine clock-based verdict, outside
  `src/rete/kernel/tests/`, and NOT a lint false positive. It is a hang watchdog, not a
  performance apportionment: the claim "does not hang forever" has no deterministic witness by
  construction (a stuck process is indistinguishable from a slow one except by timeout). T1 was
  ruled against the rete cost-test family's machine-relative phase comparisons; this is a
  different class the brief does not cover. **Listed, not converted** — exempted with a rune
  stating exactly this, for the builder's call, not mine.

## Gates

| gate | command | result |
|---|---|---|
| the lint, + mutation (item 3) | `cargo test --release --test lint floor_never_reads_a_clock::` | **11 passed; 0 failed** (10 unit tests on the detector itself + the live sweep) |
| benches build | `cargo bench --no-run` and `cargo build --release --benches` | **rc 0**, both |
| release floor | `scripts/floor.sh`, foreground, `timeout: 600000` | first run: **RED**, 3 failures (none clock-related — see below); fixed; second run: **GREEN** |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | **rc 0** |

### The floor's first red, verbatim arms, and the fix for each

Run `.floor/2026-10-01T18-20-44Z/` (summary line: `6349 tests run: 6346 passed (25 slow), 3
failed, 24 skipped`). Per doctrine, **not re-run** before capture; each arm diagnosed and fixed
before anything ran again.

1. **`wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert`** —
   ```
   🔥🔥🔥 LOOSE STRING ASSERTIONS — 1 site(s) assert a value with contains/starts_with/
   ends_with where an exact `assert_eq!` belongs. …
   tests/lint/floor_never_reads_a_clock.rs:712
   ```
   My own new lint's unit test used `assert!(v[0].contains("h < 25.0"), ...)`. Fixed: exact
   `assert_eq!(v, ["  fixture.rs:4: assert!(…) — clock-derived conjunct `h < 25.0` is not the
   liveness form `IDENT > 0` / `IDENT > 0.0`"])`.
2. **`wat::lint rete_citation_resolves::every_bare_filename_in_a_rete_comment_names_a_file`** —
   ```
   🔥 1 bare filename(s) cited in a comment under src/rete name no file a reader can open.
   src/rete/kernel/tests/fanout_cost.rs:366  `baseline.rs` — no file with that name exists anywhere in the corpus
   ```
   My own prose, line-wrapped so `perf_arc278_fire_` landed on one line and `baseline.rs\`` on the
   next — the bare-filename citation checker (which requires no `/` in the token to fire) saw a
   standalone `baseline.rs`. Fixed: reflowed so the filename is not split across the wrap.
3. **`wat rete::kernel::tests::accum_cost::accum_matcher_op_census`** —
   ```
   assertion `left == right` failed: the operation census is no longer describing the same set
   of operations — a counter appeared, vanished, or was renamed. Compare the two lists rather
   than a total
     left: […, "accum:snapshot-alloc-bytes", …]
    right: […]  (missing it)
   ```
   This is a deliberate closed-universe gate (its own doc: "adding a counter anywhere in the fire
   path turns this test red … The NAMES are the claim"). My new `accum:snapshot-alloc-bytes`
   counter (row 6) fires unconditionally on every accumulate node this [200 200] world visits, so
   it correctly caught the addition. Fixed: added the name to the expected list with a dated note
   (the sibling `accum:rematch` counter does NOT appear here, since this axis has no leftover
   `SeedCmp` and never takes the rematch branch — noted alongside).

Re-ran the two global census-universe lints (`census_name_read_by_a_cost_test_is_emitted`,
`census_emitted_name_is_read_or_declared`) and the whole `rete::kernel::tests::` module directly
before re-running the floor — both green, 127/127 — to bound the blast radius of the three new
counters before paying for another 6.5-minute run.

**Second floor run** (`.floor/2026-10-01T18-31-12Z/`): `Summary [ 387.113s] 6349 tests run: 6349
passed (26 slow), 24 skipped`. **exit=0.** Count against the brief's 6339 baseline (`628260fd9`,
`.floor/2026-10-01T09-29-58Z`): **+10** — the new lint file's 11 tests (1 sweep + 10 detector
unit tests) minus row 7's 1 deleted test. Exact.

## What moved to `benches/` vs stayed

Nothing physically moved. Row 7's function was **deleted** (its byte-identical, fully-diagnostic
twin already lives in `benches/binding_repr.rs`, landed at `ff65ae7d5`, which named this exact
stone as the reason it had not yet followed). Row 3's diagnostic stays printed in the test, not
gated and not relocated, for the public-API reason given above.

## Reds and STOPs

- **STOP-3** (one site, listed above): `tests/process/doomed_child_boot_ack_does_not_hang.rs:87`.
- No STOP-1 or STOP-2 fired on any of the seven rows: every witness built encodes its row's claim
  and passed on first measurement except row 3, whose premise the brief proposed and which I
  measured and falsified (documented above, not a STOP — the brief's own row-7 fallback applies).
- The brief contradicted the code once, named per doctrine: row 3's "production ≥ 2× hash-join"
  by operation count does not hold; the real counts are equal (1:1), and the dominance is a
  per-item cost fact, not a volume fact.

## Doctrine notes

`$?` never captured via a piped command. Floor read via `scripts/floor.sh`'s own captured
`clean.log`/`ARM.txt`, never a summary. No test re-run before its arm was captured and quoted
verbatim. No `git add -A` — every commit stages named paths only.
