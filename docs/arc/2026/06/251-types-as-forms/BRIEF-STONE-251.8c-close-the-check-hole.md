# BRIEF — STONE 251.8c: close the check hole (#95)

**Drawn 2026-09-19 against `main` @ `d52b0ab87`** (floor 5918/5918, clippy 0, census `no STOP-8`).
Design: `DESIGN-STONE-251.8-symbol-proper.md` §251.8c. Live breadcrumb: `docs/SEAM.md`.

**Why now:** the builder ruled the clojure/EDN compliance work live — *"the last outstanding item for
non-edn compliant is our keywords having `::` in them… we need to make all call heads a symbol."* 8c is
the stone that must land **before** the corpus flip (8d), because the flip must not begin while a
dotted call head is unchecked.

---

## ⛔⛔ READ THIS FIRST — THE DESIGN'S HEADLINE IS STALE, AND THE SCOPE IS MUCH SMALLER

The design (2026-08-13) states:

> *"a call head spelled as a dotted symbol is invisible to the type checker — **args, arity and return
> are all unchecked**, because `infer_list` gates its entire call-inference universe on
> `if let WatAST::Keyword`… a namespaced `Symbol` head falls to a fresh type var, which unifies with
> anything."*

**That was true in August. Re-measured at HEAD today, most of it is CLOSED.** Something between then and
now — 8a, 8a-ii, 8b's stored `(ns, name)` tuple, or the replay's own work — closed the general case.

### What I measured (probes run by hand at `d52b0ab87`, callee `(:user::f [n <- i64] -> i64)`)

| shape | colon-quoted head | slashed head | verdict |
|---|---|---|---|
| statement position — `(do (H "boom") nil)` | rc=1 | **rc=1** | ✓ same |
| let-bound — `(let [x (H "boom")] nil)` | rc=1 | **rc=1** | ✓ same |
| plain nested arg — `(:user::g (H "boom"))` | rc=1 | **rc=1** | ✓ same |
| kwarg value, `assertion-failed! :actual (H "boom")` | rc=1 | **rc=1** | ✓ same |
| **`format` `:v` value — `(format "{v}" :v (H "boom"))`** | rc=1 | **rc=0** | ⛔ **LEAKS** |
| unresolvable head — `(user/nope 1)` | — | rc=1, `UnresolvedReference :path ":user::nope"` | ✓ normalizer fires |

⛔ **Two conclusions, and the brief's honesty rests on them:**

1. **The slashed head IS normalized and IS resolved.** `user/nope` becomes `:user::nope` and raises a
   located `UnresolvedReference`. `resolve/normalize.rs` converts `Symbol → Keyword` for every
   `ident.is_reference()`. So "the checker never sees it" is **no longer the defect**.
2. **The residue is narrow**: the arg/arity check is skipped only when the call's **result lands in a
   position that does not constrain its type**. `format`'s `:v` accepts anything; the other four shapes
   all force the callee's inference and all catch it.

### ⚠ A HYPOTHESIS, MARKED AS ONE — DO NOT BUILD ON IT UNTIL YOU CONFIRM IT

**Hypothesis:** with a `Keyword` head, `infer_list` forces the callee's signature eagerly and validates
args/arity as a side effect. With a head that arrived as a `Symbol`, some path computes the call's type
*lazily*, so when nothing downstream constrains the result the validation never runs.

⛔ **This is a story that fits six data points, not a mechanism I traced.** **Your first task is to
confirm or kill it**, and the brief is wrong if you cannot. If the real mechanism differs, **say so and
report** — the fix follows from the mechanism, not from this paragraph.

⚠ Also unconfirmed and worth one measurement: whether `format` is genuinely "unconstrained" or whether
its `:v` handling is special. If `format` turns out to be the only site in the tree with this shape, 8c
is a much smaller stone than if the class is general. **Measure the class before sizing the fix.**

---

## The work

### 1 — Establish the mechanism (measurement, no code change)

Trace one leaking call and one non-leaking call through `infer_list` (`src/check.rs:2518`, the Keyword
gate at `:2542` closing `:5568`) and whatever path a normalized-from-Symbol head actually takes. Produce:

- the **exact divergence point** — the line where the two paths stop agreeing;
- whether the arg/arity validation is *skipped* or *never reached*;
- the **class**: how many call sites in the tree could hit it. ⛔ Derive this from the data, **not** by
  grepping for `format` — and exclude comments and string literals (see the doctrine below).

### 2 — Close it at the right rung

The design's ladder applies: **no-form beats a check.** If the two paths can be made *one* path, that is
better than teaching the second path to validate. ⛔ **Do not patch `infer_list`'s Keyword gate to "also
accept Symbols"** — the design cut exactly that as treating the symptom
(*"the check rung, on a substrate where the no-form rung is reachable"*).

⛔ **If closing it honestly requires something outside 8c's scope — the registry (255), or 8b's
normalizer inversion — STOP AND REPORT.** Do not smuggle another stone in. Two things the orchestrator
already measured, so you need not re-derive them:
- **255 is NOT required for 8c.** A colon-quoted *builtin* head with a wrong arg type is caught today
  (`(:wat::i64::+ "a" 1)` → rc=1), so the machinery for builtins and user fns both works through the
  existing path. The registry supplies other things; it does not gate this.
- **8b landed only its storage half** (`71c9f2f58`, *"the tuple is STORED — `Identifier` holds
  `(ns, name)`"*). `resolve/normalize.rs` still runs `Symbol → Keyword`, which is the direction 8b was
  meant to invert. **If your fix depends on the inversion, that is a finding — report it, do not do it.**

### 3 — Prove it with the matrix, not with one test

The acceptance is the table above going **all-same**. Add a test that drives **every** row — the four
that already pass are the non-vacuity control: if a future change breaks them, the test must say so.
⛔ A test that only covers the `format` row proves nothing about the class.

---

## The gate

- `scripts/floor.sh` green, clippy `-D warnings --all-targets` 0 — **the orchestrator's row**, run
  uncontended. Predict the test-count delta from the diff first, then confirm with
  **`cargo nextest list`** (never a `#[test]` grep — see below).
- `scripts/replay/census.sh --diff` → `no STOP-8`.
- The probe matrix all-same, with the four currently-passing rows still passing.

## Doctrine — `wat-rs/CLAUDE.md` does not reach a subagent

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture it whole the **first** time; never re-run.
- ⛔ **ASK THE TOOL THAT OWNS THE FACT.** Test counts → `cargo nextest list`. Deltas → `git diff` with
  `index` **and** `@@` stripped. **The orchestrator's own instruments miscounted seven times this
  campaign** by matching source text — comments, prose, string literals, macro-expanded tests, another
  gate's runes, hunk headers, and a struct field. **Never hand-roll a pattern for a structural fact.**
- ⛔ **A GREEN FROM A MIS-AIMED PROBE IS INDISTINGUISHABLE FROM A WORKING GATE.** While drawing this
  brief the orchestrator concluded "the return type IS checked" from a probe whose `rc=1` was actually
  an unresolved function name. **Check that a probe fails for the reason you think it does.**
- ⛔ **If this brief contradicts the code or the runner, they win — land what is true and report the
  brief's error.**
- Work only in `/home/john/work/holon/wat-rs`. No `git filter-branch`. Do not push.

## Out of scope — affirmatively cut

- **The corpus flip.** That is 8d, and it must not begin until this matrix is all-same.
- **`::` retiring as a reference spelling.** Also 8d.
- **8b's normalizer inversion.** If needed, it is a finding (above), not this stone.
- **The registry (255).** Measured as not required here.
