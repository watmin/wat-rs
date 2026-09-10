# SCORE — stop-teaching-the-join-blowup

Urgent, narrowly-scoped fix: `wat-scripts/fixes/hoist-where-into-condition.wat` currently hoists
**every** hoistable trailing `(:wat::rete::where …)`, including the 202 single-condition, harmless
ones — copying that codemod's output onto a single-condition rule is what broke a remote SSH
daemon (another session read a single-condition fence-hoist out of our corpus and applied the
pattern somewhere it materialized a real join). This strike (1) gates the codemod to the JOIN case
only (>=2 ordinary conditions), (2) re-derives the in-scope population from scratch, and (3)
applies it — fixing what compiles, leaving and reporting what does not. Written as-I-go per house
rule. Branch `grok-rete`, floor baseline `5490/5490`.

## Re-derivation, before any edit

Read `wat-rs/CLAUDE.md` in full (floor discipline, no-known-flakes, scratch-`.wat` location,
wat-fix codemod rule). Read `wat-scripts/fixes/hoist-where-into-condition.wat` in full (405 lines)
— it already exists, dry-run proven, idempotent, but has **no join-size filter**: it hoists a
`where` whenever every var it reads is bound by EXACTLY ONE ordinary condition, regardless of how
many ordinary conditions total sit in the `:when`. Read `docs/arc/2026/06/278-rules-engine/
strike-hoist-the-corpus/SCORE.md` and `the-fence-says-what-the-clause-cannot/FINDING-*.md` — the
prior full-corpus attempt (32 files, 63 sites, `tests/` only) hit **24 failures, four mechanisms**,
reverted; that finding is the source of "expect some to fail, and that is DATA."

`git status` was clean on `grok-rete` (branch already existed with this task's starting commit
`11add456b`) before any edit; `pgrep -af 'cargo|nextest'` clear.

## The gate: restricted the codemod itself (not a selective-run-then-revert)

Chose to **modify the codemod**, per the brief's first option. Added, in
`wat-scripts/fixes/hoist-where-into-condition.wat`'s `rule-form-edits`: after computing `targets`
(the `collect-hoist-targets` result — exactly "ordinary fact conditions reachable through `:and`"),
short-circuit to **zero edits for the whole rule** when `(length targets) < 2`. This is a single
`if` wrapped around the existing where-processing `let`, closing with two extra parens — no other
line changed. A rule with fewer than two ordinary conditions is now **never touched**, even at a
`where` that WOULD otherwise satisfy the var-binding hoist criterion — so a single corpus-wide run
of this codemod can no longer touch the 202-rule harmless class, structurally, not by caller
discipline.

Verified the gate on `tests/rete/datamancer.src.wat` (dry-run copy, diffed against the real file):
the 3 multi-condition rules (`:dm::read-after`, `:dm::recolligere` with 2 conditions each,
`:dm::four` with 4) hoisted; the 4 single-condition rules in the SAME file (`:dm::gap`,
`:dm::curare`, `:dm::examinare`, `:dm::extirpare`, each `[(:dm::Beat …) (:wat::rete::where …)]`)
were left byte-identical. This is the load-bearing proof the gate works — a file that mixes both
populations only moved the join ones.

## Population census — RE-DERIVED, and it does NOT match the brief's 55

Built `wat-scripts/scratch-pad/census-join-scope-where.wat` — a **read-only** report tool that
copies the codemod's shape-classification / collection helpers VERBATIM (`top-shape-tag`,
`collect-hoist-targets`, `collect-where-sites`, `var-occurrences`, `subset?`, …) so its verdict is
guaranteed to agree with what the (now-gated) codemod itself will and will not touch. For each
`defrule`/`defquery`, it reports IN SCOPE iff (1) at least one trailing-`where` site is hoistable
to exactly one target (criterion 1) AND (2) the `:when` carries >=2 ordinary conditions total
(criterion 2). Verified against `datamancer.src.wat` first: reports exactly the 3 rules the dry-run
diff already showed, with the right target/hoistable counts each.

Ran it over every `.wat` file (NOT `.rs` — an early pass leaked 6 `.rs` fixture files that embed
`:wat::rete::where` as a string and crashed the reader; corrected to `find … -name '*.wat'`) under
`wat-scripts/` and `tests/rete/` that contains the literal string `:wat::rete::where` (108 files).

**Re-derived total: 42 rules in 17 files — not 55.** Per-directory, against the brief's table:

| dir | brief said | re-derived | delta |
|---|---|---|---|
| `wat-scripts/grep` | 11 | **10** (4 files: `bare-variant-constructors.wat` 4, `defined-twice.wat` 2, `head-position.wat` 1, `unwrap-of-lookup.wat` 3) | -1 |
| `tests/rete` | 12 | **9** (4 files: `datamancer.src.wat` 3, `probe_arc278_left_idx_latch.wat` 1, `probe_arc278_two_where_native_spec.wat` 1, `probe_arc278_where_is_positionally_free.wat` 4) | -3 |
| `wat-scripts/perf/grid` | 12 | **8** (2 files: `where-join-order.wat` 6, `where-test-chain.wat` 2) | -4 |
| `wat-scripts/scratch-pad` | 15 | **11** (6 files: `d2-derived-fact-axis.wat` 3, `probe-grep-cli.wat` 1, `probe-grep-driver.wat` 1, `probe-where-before-fact-condition.wat` 1, `rules-corpus-01-node-facts.wat` 2, `rules-corpus-03-source-to-facts.wat` 3) | -4 |
| `wat-scripts/fixes` | 5 | **4** (1 file: `to-faithful-clojure-net.wat` — `g3-genuine`, `g4-namespaced`, `g5-type-shaped`, `g6-arrow`) | -1 |
| **total** | **55** | **42** | **-13** |

I do not know the brief's counting method, so I cannot say which of us is wrong at any one site —
only that a criterion-1-AND-criterion-2 walk that **agrees with the codemod's own decision
procedure by construction** (same helpers, verbatim) finds 42, and every one of the 42 is backed
by a printed `file :: rule :: targets=N :: hoistable-wheres=M` line, not a hand count. The
`census-join-scope-where.wat` tool stays in `wat-scripts/scratch-pad/` (correct location per
`wat-rs/CLAUDE.md`) as the re-checkable instrument for this number — it is not prose.

## Priority order: `wat-scripts/grep` and `wat-scripts/fixes` first

Per the brief. Dry-ran BOTH first (5 files: 4 `grep` + `to-faithful-clojure-net.wat` from `fixes`),
copied into the scratchpad, diffed against the real corpus — every rewrite was exactly "delete the
trailing `where`(s), splice the predicate(s) onto the one binder condition, nothing else moves."
Then dry-ran the remaining 12 files (`tests/rete` x4, `perf/grid` x2, `scratch-pad` x6) the same
way. All 17 dry-run diffs read correctly before anything real was touched.

## Applied for real, batch 1 (`grep` + `fixes`, 5 files) — one broke, reverted

Applied via `cargo wat ./wat-scripts/fixes/hoist-where-into-condition.wat` (real paths, no `sed`,
no hand-edits). `every_wat_scripts_file_loads_on_the_current_runtime` (`--release`, the gate that
actually compiles every rule at startup, not just parses it) went RED on exactly one file:

```
1 of 476 wat-scripts/ files do not load on the current runtime (rotted):
  wat-scripts/fixes/to-faithful-clojure-net.wat
      #wat.rete/ReteCheckErrors {... :errors [
        ConstraintTypeMismatch {rule "fix::g3-genuine" ... field "?slen" op-type "string" field-type "i64"}
        ConstraintTypeMismatch {rule "fix::g3-genuine" ... field "?len"  op-type "string" field-type "i64"}
        MalformedClause {rule "fix::g4-namespaced" clause "(:fix/has-ns? ?name)"}
        MalformedClause {rule "fix::g5-type-shaped" clause "(:fix/type-shaped? ?name)"}
      ]}
```

Two distinct walls in ONE file:

- **`g4-namespaced` / `g5-type-shaped` — Wall (A), exactly as briefed.** `(:fix/has-ns? ?name)` and
  `(:fix/type-shaped? ?name)` are user-function calls. `expr_is_provably_boolean`
  (`src/rete/clause.rs:242`) never covers a foreign call — legal inside a `where`'s fence, refused
  inline.
- **`g3-genuine` — a FIFTH mechanism, not one of the briefed four.** The original `where` compared
  `?slen`/`?len` (both `i64` fields of `:fix::Node`) with `:wat::rete::core::string::=` — a
  **string** comparator on **i64** operands. Inside a `where`'s fence this apparently was never
  checked against the fact schema; the inline-clause compiler DOES check the comparator's declared
  operand type against the field's schema type, and refuses it: `ConstraintTypeMismatch`. Whether
  the original predicate was a latent bug (wrong comparator, silently untriggered because `where`
  never checked it) or the fence deliberately admits looser typing is outside this strike's scope
  to adjudicate — either way, hoisting SURFACES a stricter static check the fence's interior does
  not enforce, the same shape as Wall (A) (inline stricter than fence) but a different concrete
  check (schema-type match vs. boolean-shape provability).

`g6-arrow` (`(:wat::rete::core::or (:wat::rete::core::string::= ?name "<-") (:wat::rete::core::string::= ?name "->"))`)
did **not** appear in the error list — it alone would likely have hoisted cleanly. Reverted the
**whole file** to `HEAD` rather than attempt a partial-hunk keep of `g6-arrow`: this file's four
rules were migrated in one codemod pass, and hand-surgery on the tool's own diff to keep 1-of-4
hunks is a bigger correctness risk (silently drops the OTHER three's context, invites exactly the
kind of hand-edit `wat-rs/CLAUDE.md` bans) than leaving one extra harmless-now, harmless-later
fence in an exemplar file. `git checkout -- wat-scripts/fixes/to-faithful-clojure-net.wat`.
Re-ran the lint (`--release`): green, 1 test passed.

**`wat-scripts/fixes`: 0 of 4 rules fixed, all 4 left, file untouched (reverted to HEAD).**
**`wat-scripts/grep`: all 4 files / 10 rules fixed clean**, confirmed by the same green lint run.

## Applied for real, batch 2 (`tests/rete` x4, `perf/grid` x2, `scratch-pad` x6) — clean

Applied the same way. Every hoisted predicate in these 12 files is a `:wat::rete::core::` op
(`string::=`, `i64::<`/`i64::>`/`i64::=`, `or`, `String/contains?`) — no foreign calls, no
cross-field type mismatches — so none of the five observed mechanisms applied here. Verified:

- `every_wat_scripts_file_loads_on_the_current_runtime` (`--release`): green (covers all 8 of the
  12 files that live under `wat-scripts/`: `perf/grid` x2 + `scratch-pad` x6).
- Targeted `tests/rete` probes (`--release`, all PASS): `probe_arc278_left_idx_latch::*` (2),
  `probe_arc278_two_where_native_spec::*` (1), `probe_arc278_where_is_positionally_free::*` (4),
  `probe_arc278_rete_edn::*` (4), `rete_names_in_wat_scripts_code_resolves` (1). 13/13.
- `wat-scripts/perf/grid`'s two files are in the `where-*` expressivity corpus compared against an
  oracle and against Clara — the exact "must still pass" differential this task calls out. Ran
  `wat_scripts_grid_port_check::every_grid_axis_native_matches_its_oracle`,
  `wat_scripts_grid_axes_live::grid_axes_run_and_derive_nonvacuously`, and
  `wat_scripts_grid_axes_live::spec_equals_native_on_every_where_family` (`--release`): 3/3 PASS —
  native still agrees with the oracle row-for-row after the hoist, for both files.

### `datamancer.src.wat` — regenerated its golden, verified the regen

`tests/rete/datamancer.src.wat` is NOT under `wat-scripts/` (so the corpus-load lint doesn't cover
it) and is not loaded by any `.rs` test directly — its own header says "Tests do not invoke
`:user::main`; the CLI does." `tests/rete/datamancer.rete.edn` is a checked-in COMPILED snapshot of
it, consumed independently by `probe_arc278_rete_edn.rs` as pure data (behavioral checks: deduces
Datamancer, correct sigil, canonical round-trip, impostor is Hollow — never a byte-exact structural
comparison against the source). Ran `cargo run --release --bin wat -- tests/rete/datamancer.src.wat`
per the file's own documented regen command; this rewrote `datamancer.rete.edn` in place — fewer
`:t` (beta test) nodes, more compound `:a` (alpha) nodes, exactly the intended shape change (e.g.
node 0 goes from `[:a 0 0 1 6]` to `[:a 0 0 1]` with the string test folded into the alpha cond).
Re-ran `probe_arc278_rete_edn::*` (4 tests) against the regenerated golden: all 4 PASS, including
`disk_program_edn_is_canonical` and `practice_on_disk_program_deduces_datamancer` — the compiled
shape changed, the behavior did not. **Kept the regenerated `.edn`** (committing source and its
derived golden together; leaving them out of sync would be the actual defect).

**`tests/rete`: 4 files / 9 rules fixed clean. `wat-scripts/perf/grid`: 2 files / 8 rules fixed
clean. `wat-scripts/scratch-pad`: 6 files / 11 rules fixed clean.**

## Idempotency, re-checked on the real corpus

Re-ran the (gated) codemod a second time over all 16 successfully-migrated files: zero further
edits (`git status` unchanged by the second pass). Ran `census-join-scope-where.wat` over the same
16 files: zero output lines — no rule in them is still in scope. Matches the tool's documented
idempotence.

## Floor

`pgrep -af 'cargo|nextest'` clear before starting (after killing an accidental duplicate
`nextest` invocation of my own mid-session — a Monitor watch on `probe_arc278_rete_edn` that
overlapped a still-running targeted run; confirmed no corruption, re-ran clean). `./scripts/floor.sh`,
foreground, captured:

```
     Summary [ 455.719s] 5490 tests run: 5490 passed (2 slow), 19 skipped
```

Matches baseline exactly (`5490/5490`). Log kept at `.floor/2026-09-10T02-39-02Z/` (symlinked
`.floor/latest`) — a green run is evidence too.

## Commit

Floor is green — committing per house rule (commit only on green). Staged explicit paths only
(never `git add -A`): the 17 corpus `.wat` files, the regenerated `datamancer.rete.edn`, the
codemod's join-only gate, the new census script, and this `SCORE.md`.

## What was fixed / left / not touched

- **Fixed (hoisted, verified): 38 of the re-derived 42 rules, in 16 of the 17 files** —
  `wat-scripts/grep` (4 files, 10 rules), `tests/rete` (4 files, 9 rules), `wat-scripts/perf/grid`
  (2 files, 8 rules), `wat-scripts/scratch-pad` (6 files, 11 rules).
- **Left, reported, NOT hoisted: 4 rules in 1 file** — `wat-scripts/fixes/to-faithful-clojure-net.wat`
  (`g3-genuine`, `g4-namespaced`, `g5-type-shaped` genuinely refuse to compile after hoisting —
  Wall (A) x2 + a fifth, schema-type-mismatch mechanism; `g6-arrow` collaterally left with its
  siblings for a safe whole-file revert). Tracked against
  `docs/arc/2026/06/278-rules-engine/the-fence-says-what-the-clause-cannot/`.
- **Not touched at all: the harmless single-condition population** (202 rules, per the brief) —
  structurally excluded by the codemod's own new `< 2 targets -> no edits` gate, not by caller
  discipline. Every OTHER `.wat` file containing `:wat::rete::where` outside the re-derived 42-rule
  list was never passed to the codemod.
- **Not touched: any file outside `wat-scripts/` and `tests/rete/`** — the census only walked those
  two trees per the brief's own scope table; a `:wat::rete::where` elsewhere (if any) was not
  surveyed.

---

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_018ntHDMRNCKDKNr2gVfzXmP
