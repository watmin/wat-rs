# BRIEF — excursus 002 stone 2: the tail escape

Catch the shape that cost 38 days: a `let` that creates a `Handle`, whose tail call carries that
service's `Peer` out while the scope — and the handle — dies underneath it. Stone 1's rule stands
unchanged; the only new concept is **tail position**.

Read `DESIGN-stone-2-tail-escape.md` beside this file first. It carries the closed seven-form table,
the one contract decision, and a collision whose answer is the OPPOSITE of stone 1's.

## Read in order, and why you are being sent there

1. **`src/runtime.rs:4360-4438`** — `eval_tail` and its dispatch. **This is the authority on what
   tail position means in wat.** Seven forms carry it: `if`, `match`, `let`, `do`, `and`, `or`,
   `ann-form`. Read the dispatch match, not the doc comment above it.
2. **`src/runtime.rs:4618`** — `eval_let_tail`, and specifically that it builds `scope`, then calls
   `eval_tail` on the final body form. When that emits `EvalSignal::TailCall` the signal propagates
   OUT of this function and `scope` drops with it, before the trampoline invokes the callee. That
   is the defect in one paragraph.
3. **`src/check.rs`** — stone 1's `HandleCreationEscape` and its creation detection (a call whose
   scheme returns a service Handle aggregate — `handle` + `addr`, `addr` being
   `(Address :- [Op Reply …])` — and takes none). **Reuse it. Do not re-derive it, and do not go
   back to matching a `/start` name suffix**; the structural form is deliberately better.
4. **`src/check.rs:7749`** — `infer_let`, where a let's bindings are already available while the
   body is inferred (proven by `probe-tail-scope-sees-bindings.wat`).
5. **`wat-scripts/scratch-pad/probe-tail-scope-sees-bindings.wat`** — green today, and it carries
   `:c2::the-tail-escape-the-wall-must-reject` beside `:c2::held`, which differ ONLY in whether the
   drive sits in the body or a binding. A wall that cannot separate those two is too blunt to ship.

## The rule

At a `let` that CREATES a `Handle` of service S: if the let is in tail position AND its tail
expression is a **user-function call** taking an argument of type `(Peer :- [S::Op S::Reply])`,
raise. A builtin head does not emit `TailCall` and must not fire.

## ★ The one contract decision — read DESIGN before you choose

The checker must learn tail position, and the runtime already defines it. **Do not write a second
list.** One shared constant, read by both `eval_tail`'s dispatch and the checker. If the shapes
genuinely forbid sharing, build a drift gate — a test that fails when the two disagree — and say
plainly in the SCORE which you built and why.

A duplicated list with no gate is a **FAIL even with a green floor**, because that failure is
invisible: the wall goes wrong in both directions and nothing goes red.

## Blast radius

`src/check.rs` (+ the `CheckErrorKind` file if the error needs a new variant — reusing
`HandleCreationEscape` with a distinguishing field is also acceptable, your call, state it), plus
whichever file ends up owning the shared tail-form list. **No behavioural runtime change**: you may
MOVE the tail-form list into a shared location, but `eval_tail` must dispatch on exactly the same
seven forms afterwards. TCO is correct and is not being fixed.

## The census must be RUN, not grepped

Unlike stone 1, this shape is not greppable — it depends on the AST, not the text. Build the wall,
then `--check` the corpus and read what it rejects. Expected, all deliberate:

- `probes/red-tail-escape.wat` (write it — see below)
- `probe-self-sched-bisect.wat` ×3: `hold-in-body`, `hold-as-param`, `plain-service-tail`
- `probe-tail-scope-sees-bindings.wat`: `:c2::the-tail-escape-the-wall-must-reject`

**Anything else is a finding, not a nuisance** — it would mean live code severs a service today.
Report it; do not rune it.

## Write the acceptance criterion FIRST

`docs/excursus/2026/08/002-handle-lifetime-wall/probes/red-tail-escape.wat`, modelled on
`probes/red-creation-escape.wat` beside it: self-contained, carrying the escape that must be
REJECTED next to the near-identical binding form that must KEEP compiling.

**It goes in `probes/`, never under `wat-scripts/`** — that tree's loader gate type-checks every
file, so a must-be-rejected file there turns the floor red for as long as the wall works. That was
stone 1's specification error; do not repeat it.

## STOP triggers

**STOP-1** — if tail position cannot be threaded to `infer_let` without touching evaluation
behaviour, STOP and report where it breaks. Do not approximate it with "the let's body is a call" —
that ignores condition 2 and invents false positives on every non-tail let.

**STOP-2** — if the wall rejects anything outside the expected list above, STOP and report the
site. That is a real finding.

**STOP-3 — the collision, and its answer is the OPPOSITE of stone 1's.**
`probe-self-sched-bisect.wat` holds three deliberate tail escapes. Stone 1 solved its collision by
MOVING the file to `probes/`; that is wrong here, because the bisect probe is a program that RUNS
and prints its discrimination table, and a rejected file cannot run. It gets a `rune:` per escape,
each with a reason. The rule to hold:

> **Rune the INSTRUMENT. Never rune the ACCEPTANCE CRITERION.**

If you find yourself runing `red-tail-escape.wat`, you have silenced the wall's only proof it
fires — the exact trap refused on stone 1.

**STOP-4** — if a shared tail-form list is not reachable and you fall back to duplication, you must
ship the drift gate. Duplication with no gate: STOP and report rather than ship.

## Prior comparable result

`SCORE-stone-1-creation-scope-escape.md` beside this — same excursus, and its Row 9 section records
the probe-placement error this brief is written to avoid.
