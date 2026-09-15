# BRIEF 7a — replay batch 4a: grok-rete #153 → #159, the first codemod-source steps (P1 + Q1)

> RULED 2026-09-15 (four questions, P1 and Q1): `POLICY-codemod-source.md`. 4a is the first use of the
> policy and the rete-totality sweeps — small, and verified before 4b (#160–#211) runs.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. **Never use worktrees.** Do not touch `~/work/holon/` or `main`.

## The work

Replay grok-rete **#153 through #159** (`#153 = cb2b58117`) by `BRIEF-1-pilot-first-ten-commits.md`
§ "One step" with every row it carries, and each commit body carrying the five verdict lines verbatim
(step 4; `scripts/replay/verify-step-record.sh` must exit 0 before you yield).

The census: 3 docs-only; 4 shared sweeps — #153 (25 files), #155 (188), #157 (195), #159 (205): rete
becomes total (`FireOutcome`, `InsertOutcome`, `CompileOutcome`). All four touch the stdlib (`wat/`; the
two-phase rule) and `wat-scripts/fixes/`; #155 #157 #159 change the stdlib macro
`:wat::query::sift-rules-defsvc` (2a4c's door replaces it in its copy). None touches a file absent on main.

## First — `convert.sh` admits a codemod that is not the chain

`convert.sh` skips every `wat-scripts/fixes/` path (`scripts/replay/convert.sh:56`, `:97`), because the
chain once rewrote its own tools. The rule the policy needs is narrower and DERIVED: a
`wat-scripts/fixes/*.wat` is converted like any `.wat` unless it is a CHAIN MEMBER — the list
`scripts/replay/chain-order.sh` prints (27 today). The chain still never converts itself. Its guard must
fail once: a chain member's path handed to `convert.sh` is refused, loudly.

## P1 — port grok-rete's new codemods (5 in 4a)

`wrap-fire-once-in-fireoutcome` (#153), `wrap-fire-rules-in-fireoutcome` and
`wrap-fire-rules-explain-in-fireoutcome` (#155), `wrap-insert-in-insertoutcome` (#157),
`wrap-compile-in-compileoutcome` (#159). Each is new in its step and has no fixture on grok-rete.
1. Convert the tool's source through `convert.sh` like any new `.wat` (the C output).
2. Bring any pattern held as a STRING literal to today's spelling (the chain rewrites code, not data);
   add its `;; SCOPE:` line.
3. Its fixture, `wat-scripts/fixes/replay/<stem>/{before.pre,after.post,ORACLE}`: one corpus file the step
   migrated with this tool — `before.pre` its converted C^, `after.post` its converted C; ORACLE the header
   spec. `tests/cli/every_recorded_migration_replays.rs` then proves the ported tool reproduces its own step.
   If the file's C^→C carries edits beyond the tool's (a hand touch in the sweep), pick another file.

## Q1 — re-express grok-rete's edits to main's tools on main's version

`to-faithful-clojure-rete` (#155 #157 #159), `to-faithful-clojure-net` (#157 #159) — neither is a chain
member, and both already have fixtures: the standard `.wat` recipe (convert C^ and C, `git merge-file` with
main's copy). Each tool's fixture must still replay; if grok's edit changes what it emits, update the
fixture to show it (ORACLE the header spec). The four `rete-truth-maintenance-probes/*.wat` edits are probe
programs, not tools: convert like any `.wat`; `probe_arc278_7strat_native_differential` is their proof.

## Checkpoint

After **#159**: `scripts/floor.sh` and clippy. A composition repair folds into its step (the 4b rule).

## ⛔ STOP triggers — rejections; report the verbatim evidence

STOP-1 … STOP-11 as BRIEF-1 states them, plus:
- **STOP-12:** a ported tool cannot reproduce its step on any migrated file of that step.
- **STOP-13:** a main tool's fixture stops replaying after its Q1 re-expression.

## Tier

Commit each step on green. **Do not push.** Yield after #159, or at the first STOP, with
`SCORE-7a-replay-batch-4a.md` and `REPLAY-LOG.md`.
