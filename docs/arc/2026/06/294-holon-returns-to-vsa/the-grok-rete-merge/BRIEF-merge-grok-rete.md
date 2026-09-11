# BRIEF — merge `grok-rete` into main's line. ONE strike, ending in a COMMITTED merge.

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. **Never use worktrees.**
Do not touch `~/work/holon/` (the frozen root).

## ⛔ TREE STATE — READ THIS FIRST

```
branch    merge/grok-rete        <-- YOU ARE HERE. Created off main. NOT main.
HEAD      a3218644d              clean, builds, floor 5373/5373 GREEN, clippy 0
main      a3218644d              frozen and PUSHED. Do not check it out. Do not push anything.
origin/grok-rete  37528f6e0      frozen.
```

**Both branches are frozen by the builder.** Nothing is moving under you.

## The divergence — measured, not estimated

```
merge-base  de827fb4c  2026-08-24        main +911 commits    grok-rete +651 commits
conflicts   99 files:  35 .rs · 39 .wat · 23 .edn · 1 .toml · 1 .bad
```

★ **`src/check.rs` is in the overlap but does NOT conflict.** The two sides touched different
regions of it. The same is true of `src/macros/expand.rs`.

⛔ **NEITHER SIDE'S RETE CAN BE TAKEN WHOLESALE.** Both did real work there:

```
             commits  files   insertions/deletions
main            79      19       2,321 / 1,609      <- purity.rs alone +1,145 / -912
grok-rete      162      62      29,388 / 13,693
```

`-X ours` or `-X theirs` on `src/rete/` would silently discard one of those. **Do not use a
merge strategy option.** Resolve hunk by hunk.

## Why YOU

You wrote grok-rete's 651 commits. You also struck main's last three stones — the polymorphic
accessor (`b743ae310`), the lattice `Fn` arm (`58e9563b6`), and the one-door helper
(`877e59ffb`). You are the only agent holding both sides. That is the whole reason this is
yours and not the orchestrator's.

## The strike

```bash
git merge --no-ff origin/grok-rete      # 99 conflicts, expected
# resolve, then:
git commit                              # ONE merge commit on merge/grok-rete
```

⛔ **END IN A COMMITTED MERGE.** A conflicted worktree cannot be handed back — it is the most
fragile state git has, and the orchestrator cannot measure through it. If you cannot finish,
`git merge --abort` and report; do NOT yield a half-merged tree.

### Resolution rules, by class

**`.rs` (35) — THE REAL WORK. Hunk by hunk, both intents preserved.** These are listed below.
Where main changed a signature your rete code calls, take BOTH: main's signature, your call site
updated to it.

**`.wat` (39) — ⛔ DO NOT HAND-EDIT. Take grok-rete's CONTENT.** Your side carries the rete
semantics; main's side is almost entirely codemod output (measured: pure spelling renames like
`:wat::rete::core::string::=` -> `:wat::rete::string::=`). **The orchestrator replays the
migration chain as a SEPARATE commit after yours** — 28 recorded codemods landed on main since
the split and your corpus has seen none of them. Do not try to apply them by hand; R21 exists
because that is how corpora get corrupted.

**`.edn` (23) — goldens.** Take the side whose PRODUCER survived the `.rs` resolution. If a
golden's shape is decided by code you just merged, it must match that code. Do not guess — if
you cannot tell, leave it as main's and say so; the orchestrator recaptures via `UPDATE_EDN`.

**`.toml` / `.bad` (1 each).** `.bad` is a NEGATIVE fixture: it must stay **the same kind of
wrong**, rejected for its original reason, never accidentally by a new syntax error.

## ⛔ STOP triggers — each is a REJECTION. Report and halt.

**STOP-1 — a GENUINE collision between main's type system and your rete work.** Main landed
three things you struck yourself: `assignable`'s `Fn` arm (args contravariant, return
covariant), the `Variant <: Enum` lattice reaching function slots, and `expand_form` walking
MAP and SET. If your rete code depends on the OLD invariant-`unify` behaviour, that is a
**finding for the builder**, not a hunk to pick a side on. STOP and name it.

**STOP-2 — if resolving needs a THIRD behaviour** that is on neither side. That is new design
mid-merge. STOP.

**STOP-3 — if the conflict count grows past 99** or files outside the list below conflict. The
census is the orchestrator's and it is `git`-derived; a surprise means a premise is wrong.

**STOP-4 — do NOT run `scripts/floor.sh`, clippy, or the codemod chain.** The floor is RED
by construction until the chain is replayed — your corpus is pre-flip. A red floor here proves
nothing and costs 200s. The orchestrator measures centrally, after the chain.

## Acceptance

```
1. ONE merge commit on merge/grok-rete. Tree CLEAN. `git status --porcelain` empty.
2. cargo build --release  -> BUILD_EXIT=0        (this IS your gate; the floor is not)
3. no conflict markers anywhere:  grep -rn '^<<<<<<<\|^>>>>>>>' src/ tests/ crates/
4. a SCORE naming, for every .rs file: which side won each hunk, and WHY in one line
5. every STOP you hit, with the verbatim block
```

⚠ **Build green is the bar, not floor green.** The floor cannot pass until the migration chain
runs, and that is the orchestrator's next commit.

## Tier

You resolve and you COMMIT the merge (this one time — a merge cannot be handed over
half-done). **Do NOT push. Do NOT touch main. Do NOT run the floor, clippy, or any codemod.**

⛔ Do NOT call `pulsare_yield` until the merge is committed and the tree is clean.

## The 35 conflicted `.rs`

```
benches/perf_arc278_fire_baseline.rs
src/capability/registry.rs
src/comms/mod.rs
src/distribution/mcp.rs
src/distribution/mod.rs
src/edn/render.rs
src/freeze.rs
src/kernel/spawn.rs
src/lib.rs
src/panic_hook.rs
src/process/verbs.rs
src/rete/expr_ir.rs
src/rete/kernel/arm.rs
src/rete/kernel/tests.rs
src/rete/matcher.rs
src/rete/purity.rs
src/rete/step_payload.rs
src/rete/validate/mod.rs
src/runtime.rs
src/services/verbs.rs
src/string_ops.rs
src/value/pmap.rs
tests/cli/wat_cli.rs
tests/collection/list.rs
tests/diagnostics/probe_arc241_stone10_remedy.rs
tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs
tests/services/probe_arc170_m1_teeth.rs
tests/services/probe_arc209_c0b3bb_bounced.rs
tests/services/probe_arc278_dead_child_speaks.rs
tests/services/probe_arc278_recv_outcome_wall.rs
tests/services/probe_arc278_service_max_frame_bytes.rs
tests/value/clj_expr_parity.rs
tests/value/probe_arc278_a0_uniform_variant.rs
tests/value/probe_arc294_holon_bare_leaf_read.rs
tests/value/probe_arc298_1_option_result_tagged.rs
```

## The 39 conflicted `.wat` (take grok-rete's content; chain replays after)

```
tests/rete/datamancer.src.wat
tests/rete/probe_arc278_1a_data_model.wat
tests/rete/probe_arc278_fallback_generic_ret.wat
tests/rete/probe_arc278_native_insert_differential.wat
tests/rete/probe_arc278_two_where_native_spec.wat
tests/rete/probe_arc278_where_is_positionally_free.wat
wat-scripts/fixes/rete-truth-maintenance-probes/neg.wat
wat-scripts/fixes/to-faithful-clojure-net.wat
wat-scripts/fixes/to-faithful-clojure-rete.wat
wat-scripts/grep/bare-variant-constructors.wat
wat-scripts/grep/defined-twice.wat
wat-scripts/grep/head-position.wat
wat-scripts/grep/unwrap-of-lookup.wat
wat-scripts/perf/grid/min-finding.wat
wat-scripts/perf/grid/where-join-order.wat
wat-scripts/perf/grid/where-test-chain.wat
wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat
wat-scripts/scratch-pad/probe-arena-rich-graph.wat
wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat
wat-scripts/scratch-pad/probe-grep-cli.wat
wat-scripts/scratch-pad/probe-grep-driver.wat
wat-scripts/scratch-pad/probe-insert-all-cost.wat
wat-scripts/scratch-pad/probe-rete-predicate-termination-routes.wat
wat-scripts/scratch-pad/probe-rules-rich.wat
wat-scripts/scratch-pad/probe-seed-insert-vs-insert-all.wat
wat-scripts/scratch-pad/probe-sift-body-direct.wat
wat-scripts/scratch-pad/probe-stop-a-where-arith-path.wat
wat-scripts/scratch-pad/probe-where-cond-fence-execution-split.wat
wat-scripts/scratch-pad/probe-where-shape-spread.wat
wat-scripts/scratch-pad/rules-corpus-01-node-facts.wat
wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat
wat/bracket.wat
wat/grep.wat
wat/query.wat
wat/rete/oracle/explain.wat
wat/rete/oracle/fire.wat
wat/rete/oracle/insert.wat
wat/rete/oracle/pass.wat
wat/rete/oracle/stratify.wat
```
