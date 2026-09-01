# SCORE — A3, weighed against the orchestrator's own re-run

> Re-run here at `17fc5fb3e`.

## The scorecard, re-run

| # | expected | actual |
|---|---|---|
| 1 | repro RED before | ✅ `unknown rete-defn :wat::rete::core::PersistentVector/length` |
| 2 | wrapped control GREEN before | ✅ `"fired"` |
| 3 | gate RED before the ladder | ✅ **re-driven by me**, below |
| 4 | repro GREEN after | ✅ `"fired"` — the same answer the wrapped control gives, on the final binary |
| 5 | gate GREEN after | ✅ |
| 6 | the gate can FAIL | ✅ three ways, one of them the rider's own design |
| 7 | eligible set computed, not named | ⚠ **the row's INTENT holds; its COMMAND was unrunnable** — see below |
| 8 | non-empty set fails loudly | ✅ proven by mutating the predicate to match no row |
| 9 | no hollow tests | ✅ the new gate has zero `println!`; the six in the file are the pre-existing sweep's |
| 10 | `:65-68` no longer lies | ✅ rewritten: it now says the acc position IS modelled, where, and why it is not a third enum variant |
| 11 | blast radius | ⚠ **three files, not two** — `arm.rs` joined, by my own reversal (below) |
| 12 | lints | ✅ 114/114 — the rider ran it, per the last strike's floor red. **No floor red this time** |
| 13 | floor | ✅ `Summary [ 406.126s] 5203 tests run: 5203 passed (1 slow), 21 skipped`, zero FAIL rows |
| 14 | clippy | ✅ rc=0, zero warnings |

**Row 3, re-driven here** — `git stash push src/rete/expr_ir/mod.rs`, gate and fence kept:

```
FAIL  rete::reachability::every_acc_head_shaped_row_runs_as_an_acc_head
  A ROW THE ACC-FORM FENCE ADMITS IS A ROW THE EXECUTOR CANNOT RUN.
  ─── acc-form-head position, 1 eligible row(s) computed from RETE_OPS ───
  :wat::rete::core::PersistentVector/length             NOT-FIRING
```

Restored byte-identical. The gate fails on the thing it exists for.

## ⛔ I AUTHORIZED A REGRESSION, AND THE RIDER REFUSED TO SHIP IT SILENTLY

My BRIEF's trap 2 said: *"D3 landed the arity wall this session, so a synthesized program of the
wrong arity is refused with both counts named. Do not add a second refusal."* **That was wrong
against this file's own law**, which I verified on the disk after the rider cited it
(`expr_ir/mod.rs:14-19`):

> *"`lower` IS TOTAL OR IT REFUSES. A `Program` that exists is one `exec` can run — every name
> resolved, **every arity checked**, every head known. … **A refusal that belongs at compile time
> and lands at fire time is a defect in this file**, because it moves a diagnostic from the rule
> the author is writing to the millionth row of someone's data."*

Driven, `(:wat::rete::core::PersistentVector/contains? ?v)` as an acc head:

```
before  :message  ":wat::rete::call-user: expected 2 arguments, got 1"
        :location {:file "src/rete/kernel/fire/acc.rs" :line 488}
after   :message  ":wat::rete::core::PersistentVector/contains?: expected 2 arguments, got 1"
        :location {:file "<the author's .wat>" :line 7 :col 18–67}
```

**Three things moved, not one:** the location (a Rust file with no end position → the author's
acc-form, spanned), the op (`call-user`, the calling convention → the head they actually typed),
and the timing. The rider did not take the timing on trust: a program that compiles and **never
fires** raises with the fence and prints `"compiled, never fired"` without it. Measured, not
inferred from where the code sits.

The fence went at `arm.rs:430` because that is the only place that knows the operand count —
`lower_named_rete_fn` receives `(head, span, sym)` and can read a row's *declared* arity but not
how many operands the call site supplies. **D3's wall is untouched** and remains the backstop for
every other caller.

**And the rider widened my instruction, correctly.** I framed the fence around minted rows; it
reads `program.params.len()` after the lower call, so a *user* `rete::core::defn` of the wrong
arity is now refused at compile time too. That is the invariant's actual scope — `lower` is total
regardless of which registry resolved the head — and narrowing it would have made the fence a
special case for the one bug I happened to drive. 170/170 rete says nothing depended on the old
behaviour. Agreed and kept.

## ⛔ Where MY brief was thin

- **A. ★ Trap 2 was a licence for the regression above.** The strike would have shipped a worse
  diagnostic than it inherited, in the file whose module doc forbids exactly that. Caught only
  because the brief asks where it is thin — no scorecard row could have seen it, since the row I
  wrote *endorsed* the behaviour.
- **B. ★ Row 7's command could never return what I demanded.**
  `grep -c 'PersistentVector/length' src/rete/reachability.rs` returns **1 at HEAD** and 4 now —
  `operands_for:783` has always hard-coded that row for the *inline/fence* sweep, and my three
  additions are comments. Run literally, my own check fails work that passes. **Third scorecard row
  this session that could not do its job** — after a pinned count that capped coverage and a control
  that could not see a check refusing everything. Promoted to memory: *run every row against HEAD
  before shipping the scorecard.*
- **C. The container the accumulator hands over was left undecided.** "Rows whose signature fits
  `(head ?v)`" does not say *which* collection. The rider resolved it from the wrapped fixture
  (`PersistentVector`) and the non-vacuity assert now names that failure mode — the right guard, but
  I should have stated it.
- **D. Credit where it is due.** All six `file:line` citations in the read-list were accurate. After
  two strikes of miscounted sites (A6: one tower of three; D3: one call site of six), this one held
  — the difference was enumerating the callers before writing, which is now a memory.

## Arms not driven, named

`lower_named_rete_fn`'s three pre-existing refusal arms (`sym.get` → None, `func.rete.is_none()`,
non-wat body) — **reachable but not driven**, all untouched. The gate's four loud-failure arms
(`Ret::NoScheme`, unwritable return, unmappable comparator, comparator absent from `RETE_OPS`) —
**not reachable today**, no eligible row exercises them; each fails with the row named rather than
skipping it. `items.len().saturating_sub(1)` underflow — **not reachable**: the loop has already
matched `items.first()` as the head, so `items` is non-empty by construction.

D3's wall via the acc path — **no longer reachable from this caller**, proven rather than argued:
removing the fence is exactly what restores the `acc.rs:488` raise. It stays reachable for the
wire's `:user` arm, `exec_call`, and the four HOF arms, which is why the wall stays.

## Left open, deliberately

The acc-form fence in `wat/rete/compile.wat` still admits an arity-2 row: it tests pure ∧
deterministic ∧ total ∧ `primitive?`, and **arity is not one of its axes**. The refusal now lands
at compile time with the author's span, which is the right division of labour — the fence admits by
capability, the caller checks the call. Recorded so the permissiveness is on the record as
understood rather than overlooked.
