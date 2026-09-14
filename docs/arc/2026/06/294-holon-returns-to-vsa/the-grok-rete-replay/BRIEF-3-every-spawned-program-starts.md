# BRIEF 3 — every spawned program starts: the nested-program gate

> RULED 2026-09-13 (ruling 4): "a GATE checks every nested program literal … its own stone, after 2b,
> whose first act is the census". Stone 3 runs BETWEEN replay batches (batch 1 closed; batch 2 after
> this). The census is done and re-taken at HEAD `38a865b99` (finding 8; below).

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd`. **Never use
worktrees.** Do not touch `~/work/holon/` (the frozen root) or `main`.

## Why

A child program inside `(:wat::core::forms …)` is checked only when it STARTS. `--check` and the load
gates check the parent. Main's floor has been green over at least 9 broken probes whose child dies at
startup (the 7 below, plus `probe-m1-ann-erase{,2}.wat`, which 2b fixed). The fix climbs the ladder: a
gate the mistake cannot pass, not a patch per probe.

## The evidence (read it; do not re-derive it)

- **The census** (`bootstrap/era/probe-N/extract-forms3.wat` → `census3.tsv`, HEAD `38a865b99`):
  141 literals in the tracked `.wat` outside `wat-scripts/fixes/` — identical, site by site and verdict
  by verdict, to the era census (`census2.tsv`); batch 1 added none.

  | literals | where they sit | the gate |
  |---|---|---|
  | 121 | `:wat::test::spawn-peer`, process clause `prog` | CHECKED |
  | 1 | `:wat::spawn::Locus/launch` (`tests/services/probe_arc209_locus_protocol_foundation.wat:30`) | CHECKED |
  | 7 | operands of `:wat::core::concat` (an assembled program; one is `wat/spawn.wat:570`) | COUNTED, reported |
  | 5 | under a quasiquote (`wat/service.wat:2490,2491,2528`, `wat/test.wat:171`, `wat/test.wat:450`) | TEMPLATE, skipped, counted |
  | 7 | data: let-bound then `length` (`tests/wat_lang/wat_core_forms.wat:7,14,21`), `length`, `do`, 2 `defn` | not a program |

- **The child's REAL startup path** is `src/process/verbs.rs:429-433`: an `InMemoryLoader` plus
  `startup_from_forms_with_inherit(forms, None, loader, cfg)` (`src/freeze.rs:1231`). Its `inherit` is
  the parent's `Config` only — no definitions are handed down. `--check` is STRICTER: it adds
  `validate_user_main_not_useless` (`src/freeze.rs:947`, the source path only).
- **The 19 `spawn-peer` children `--check` fails** (finding 8, re-verified): 4 UselessMain (the child
  path never runs that wall — they START); 6 deliberate negatives (`wat-tests/core/core-{arithmetic,equality}.wat`
  `…-rejected` deftests, each matching `RecvOutcome.Lost` from its child); 1 `probe-child-inherits-defns.wat`
  (a decisive probe whose answer IS the failure: a process child is a fresh universe,
  `unknown :probe::dbl`); **7 broken** (D2); 1 `tests/process/arc112_scheme_probe.wat:12` (D3).

## D1 — the gate

A new lint test beside `tests/lint/wat_scripts_fixes_load.rs`, over every tracked `.wat` outside
`wat-scripts/fixes/`:
- **Which literals — DERIVED from the one door, never a hand list.** The substrate starts a child from
  forms in exactly one place (`src/process/verbs.rs:431`); its wat verb is reached only through
  `:wat::kernel::spawn-program`'s ProcessOpts clause, whose `prog` (`wat/spawn.wat:356-358`) is the ROOT
  position. A stdlib parameter is program-carrying iff it reaches a program-carrying position
  **unchanged, as an operand of `:wat::core::concat`, or through a `let` binding** — the orchestrator's
  probe of today's stdlib: `:wat::test::spawn-peer`'s ProcessOpts `prog` (`wat/test.wat:354-362`),
  `:wat::test::spawn-hermetic-program` `prog` (`wat/test.wat:419-422`), and `Locus/launch`'s process
  `service-forms`, which reaches the root only through `(let [prog (concat (forms (def …)) service-forms)]
  (spawn-program self prog))` (`wat/spawn.wat:567-572`). The same `let` rule is what classifies the
  three let-bound data literals correctly. The thread tier takes a fn, never forms.
- A literal under a quasiquote is a template: skip it and count it. A literal assembled through `concat`
  cannot be checked alone: count it and report the count. A literal used as data is not a program.
- **How.** Parse the literal's children to `Vec<WatAST>` and start them on the child's path, exactly
  `verbs.rs:429-433`, with the `Config` a parent would pass.
- **An expected failure is pinned by a TEST that asserts it.** A co-located rune on the literal names
  that test, and the gate refuses a rune whose named test does not exist. The 6 `…-rejected` deftests
  are those tests. `probe-child-inherits-defns.wat`'s answer becomes one: a `wat-tests/` deftest whose
  child references a parent-only defn and must come back `RecvOutcome.Lost` — the recorded answer "a
  process child is a fresh universe" turned into a guard; the probe's rune names it.
- **The gate must fail once.** Drive it RED on `f2e0ac26b^`'s `probe-m1-ann-erase.wat` child: copy the old
  file into a temp fixture, never into the tree. Its message names the file, line, and startup error.
- **Its cost** is measured and reported (122 child startups on every floor).

## D2 — the 7 broken probes

`wat-scripts/probes/arc-170/probe-m1-{cf-norevoke,dial-runner,fix-norevoke,fix-revoke,grant-admits,worker-setup}.wat`
and `probe-bracket-process-runner.wat`. Each child predates the RecvOutcome/SendOutcome walls:
- `recv` and `Echo/echo` return `(RecvOutcome :- [T])` where the child expects `T`;
- `SendOutcome` gained `Stopped`, which its matches lack.
Worked example (`probe-m1-worker-setup.wat`): `(match (recv self) [Msg.Setup …] [Msg.Work …])` against
`(RecvOutcome :- [Msg])`; `er` from `Echo/echo` matched as a bare response; a `send self` match with no
`SendOutcome.Stopped` arm.

**No recorded migration applies as-is — probed.** The `Stopped` arms were added BY HAND (`86b30d5dc`,
arc 278 #73: "Four classes were absent from the 496 by construction — … (forms ...) child programs");
no `wat-scripts/fixes/*.wat` adds them. `wrap-client-method-match-in-recvoutcome.wat` run on copies of
the seven (`bootstrap/era/probe-N/d2-copies/`) changes NOTHING: its matcher is pinned to the `::`
spelling (`RecvOutcome::`, `Response::`), and today's text says `.`. So D2 WRITES the missing recorded
migrations in today's spelling, on `wat/fix.wat`'s WRAP family, each able to reach a nested
`(:wat::core::forms …)` child, each with a `wat-scripts/fixes/replay/<stem>/` fixture whose ORACLE is its
header spec: (a) a `recv`/client-method result matched as its message → wrapped in its RecvOutcome match;
(b) a `SendOutcome` match lacking `Stopped` → gains the arm. Run them over the seven. Never hand-edit
(R21). `probe-m1-ann-erase{,2}.wat` (SCORE-2b § D3) is the worked example of the end state.

## D3 — the one unclassified

`tests/process/arc112_scheme_probe.wat:12` (1 unresolved reference). Classify it against finding 8's table
and fix it if it is a defect.

## ⛔ STOP triggers — rejections; report the verbatim evidence

- **STOP-1:** the derived positions miss a literal the census found in a starting position, or reach one
  only through a hand list. Report the site verbatim.
- **STOP-2:** a child needs runtime state (a live parent) that a gate cannot build, other than the pinned
  failures above.
- **STOP-3:** a wrap codemod cannot reach a child program.
- **STOP-4:** the floor is red. Paste the whole block verbatim, and do not re-run.

## Tier

Commit on green. **Do not push.** Yield with `SCORE-3.md`. Batch 2 follows.
