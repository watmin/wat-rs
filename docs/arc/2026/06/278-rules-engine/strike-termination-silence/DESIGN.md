# DESIGN-STONE — "it terminates" and "nothing was looked at" may not be the same value

> **Origin (2026-08-31).** Class A5 of `VIGILIA-2026-08-30-WORK-LIST.md`, found by `circumspicere`.
> Driven here at HEAD `09b973d2c`. **The row's two line numbers have moved and its second half is
> understated** — see below; the sentence is false for a more basic reason than "hand-assembled
> Sessions".

## Why — two halves, two different mechanisms

### Half 1 — the import door does not skip the verifier's ANALYSIS. It never calls the verifier.

`arm.rs:1294`: *"`compile-all` is the one door **EVERY** rule passes."* Unqualified.

`grep -n 'refuse_non_terminating\|verify_termination' src/rete/export.rs` → **no hit.**
`import_export` never calls it. Not a weaker check, not a skipped rule — **the door is not on the
path at all**, and the sentence claims it is.

`stratify.rs:339-342` already states the gap from its own side (*"An imported Export carries no rule
AST (`rules_lack_ast`), so there is nothing to analyse. That is where the runtime round cap keeps
earning its place"*, and `rules_lack_ast` is real at `fire/rules.rs:814` — checked, not assumed).
**So two module docs describe the same boundary and only one of them is true.** A reader who lands
on `arm.rs:1294` has no way to reach the correction.

### Half 2 — the verdict conflates three states, and it is DRIVEN

`refuse_non_terminating` (`stratify.rs:833`) returns `Result<(), EvalBreak>`. `Ok(())` is returned
from **four** places and means **three different things**:

| site | what it means | legitimate? |
|---|---|---|
| `:838` | `rules` was not even a `PersistentVector` | **no** — nothing was analysed |
| `:894` | nothing computes, so no cycle can be unbounded (371 of 381 corpus rules) | **yes** — proven |
| `:988` | the derivation graph closed with no unbounded cycle | **yes** — proven |
| `:853` (`continue`) | this rule's `lhs`/`rhs` are empty — no AST to analyse | **no** — skipped |

Driven, `wat-scripts/scratch-pad/a5-termination-silence.wat` — a `Rule` with empty `:lhs`/`:rhs`,
which is exactly the shape an imported Export's rules have:

```
compile-all  →  "Compiled"
```

**The verdict "termination was proven" and the verdict "there was nothing to look at" are the same
value to every caller.** The `continue`'s own comment says *"saying so is the honest outcome rather
than passing it as proven"* — and then says it to no one. That comment is this arc's recurring
alibi shape (A6's `unpack_driver`): a true statement of intent standing where the mechanism is
absent.

## ★ THE ONE CONTRACT DECISION

**The verifier returns a verdict with a name for each state, and its caller matches all of them.**
After this strike there must be no path on which "proven terminating" and "not analysable" are the
same value — the conflation gets no representation, so a future arm cannot re-mint it.

**Behaviour does not change.** `NotAnalysable` proceeds exactly as today; this strike makes the
state *sayable*, not fatal. Refusing it would break every session whose rules legitimately carry no
AST, and that is a policy question this stone does not open.

This is the arc's signature defect for the fourth time — A2b's `Option` (two facts), D3's missing
arity (three faces), A6's `None => true`, and now an `Ok(())` holding three. Same cure each time:
**climb to the type.**

## The algorithm

1. `refuse_non_terminating` returns `TerminationVerdict` — `Proven`, `NotAnalysable { rules: usize }`,
   `Refused(EvalBreak)` — instead of `Result<(), EvalBreak>`. Each `Ok(())` site above becomes the
   arm that names what it actually knew; `:853`'s `continue` counts instead of vanishing.
2. `arm.rs:1301` — the **only** caller — matches all three arms explicitly. `Refused` behaves
   exactly as today (the verdict reaches the outcome converter); `Proven` and `NotAnalysable` both
   proceed, and the difference is now stated at the site rather than lost.
3. `arm.rs:1294`'s sentence is qualified to what it can claim — every **locally compiled** rule —
   and names `import_export` as the door that does not call this at all, so the two module docs
   agree.

## Blast radius

`src/rete/kernel/stratify.rs` and `src/rete/kernel/arm.rs`. `refuse_non_terminating` has **exactly
one caller** (`arm.rs:1301`) — enumerated with `grep -rn`, not assumed.

## Out of scope — AFFIRMATIVELY CUT

- **A new `CompileOutcome` variant surfacing `NotAnalysable` to wat.** That is a wire-visible enum
  behind this arc's outcome wall, with a much larger radius, and it is a policy change (what should
  a caller *do* about an unverified rule set?) rather than the honesty fix this row asks for. Its
  own strike, if it is wanted at all.
- **Making `import_export` call the verifier.** There is no AST at that door to analyse — the round
  cap is the runtime answer, exactly as `stratify.rs:341` says. Adding a call that can only return
  `NotAnalysable` would be theatre.
- **A7 — import charging nothing to the session ceiling, and its O(N²) build.** The neighbouring
  row at the same door, its own strike.
