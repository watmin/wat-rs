# DESIGN-STONE — the oracle accretes superseded accumulate results

> **Origin (2026-08-31).** Surfaced while driving Class D2 — which it is **not**. Settled by the
> builder's call: *"measure this against clara — confirm who is wrong."*

## The verdict, measured three ways

The same rule set in each engine. `Tally` is derived from an `acc::count` over a type derived
mid-fixpoint, so the count changes as the fixpoint runs.

| accumulate's count | **Clara 0.24.0** | native | oracle |
|---|---|---|---|
| always 0 | **1** (`n=0`) | 1 ✓ | 1 ✓ |
| 0 → 1 | **1** (`n=1`) | 1 ✓ | **2** ✗ |
| 0 → 1 → 2 | **1** (`n=2`) | 1 ✓ | **3** ✗ |

**Clara keeps exactly one, holding the FINAL count. Native agrees on all three shapes. The oracle
emits one per intermediate state and keeps them all — the over-emission scales with the number of
changes.** Proven directly: a rule matching `Tally(n = 0)` finds one in the oracle's fact set and
none in native's. **A fact asserting the count is zero, standing while the count is two.**

All three agree that an always-empty `count` DOES emit `n=0`, so emitting is correct. **The defect
is failing to supersede.**

## The mechanism

`wat/rete/oracle/fire.wat:238` `fire-fixpoint`:

```
derived   = collect-derived(production-memory fired)
new-facts = merge-facts(old-facts, derived)
if length(new-facts) == length(old-facts) -> done, else recurse
```

**`merge-facts` is monotone — it only ADDS.** Production memory accretes every fact ever produced,
including ones derived from an accumulate result that has since changed. Nothing removes a fact
whose support is gone.

And the oracle **states the assumption this violates**, in `stratify.wat:26-28`:

> *"WHY `fire-fixpoint` unchanged: it is correct within a stratum (**monotone**, finite, no
> negation-ordering hazard). Stratification is the ordering layer."*

**An accumulate is not monotone.** Its result is *superseded*, not extended, when its source grows.
Stratification orders negation and says nothing about this. The same file already names the failure
mode in the abstract — *"leak a spurious derived fact that is never retracted"* — and treats it as
solved for negation only.

## ★ THE ONE CONTRACT DECISION

**The oracle must reach Clara's answer BY ITS OWN ROUTE. It may not be made to mirror native.**

The oracle exists to be an *independently derived* reference. A fix that ports native's delta logic
into it makes every future differential vacuous — the two would agree by construction, which is
precisely the *"comparing one path against itself and calling that agreement"* defect this arc has
already recorded (`probe_arc278_stratified_query_replay`'s own header warns of it).

So the fix is expressed in the oracle's own interpreted terms — a fact whose supporting accumulate
result has been superseded is not in the fact set — and it is **measured against Clara**, not
against native. Native's agreement is then *evidence*, not the definition.

## ⚠ THE TRAP IN THE TERMINATION TEST

`fire-fixpoint` stops when `length(new-facts) == length(old-facts)`. **That is a COUNT comparison.**
Any fix that removes a fact can hold the length equal while the SET changes — terminating on a
false fixpoint and silently returning a wrong answer. **A fix that retracts MUST change the
termination test to compare the set, not its size.** This is the single most likely way this strike
ships a worse defect than it cures.

## Blast radius

`wat/rete/oracle/fire.wat` (the fixpoint), and probably `accum-pass.wat` or `pass.wat` where an
accumulate result becomes a token. ⚠ **Scope is genuinely unknown** — the oracle is 2,164 lines of
interpreted wat and no one has changed its monotonicity assumption before. **If the fix reaches a
third file, that is a finding to surface, not a licence to widen.**

Hand-edit, not a codemod: this is one behavioural change in one or two files, not a corpus
migration (`wat-rs/CLAUDE.md`'s wat-fix rule governs structural rewrites across many files).

## Out of scope — AFFIRMATIVELY CUT

- **Class D2**, which this is not. Removing the join chain entirely left the divergence unchanged,
  so `filter → HashJoin(a) → HashJoin(b)` is not implicated. **D2 remains UNTESTED, not
  disproven**, and still owes a fixture provably carrying two chained HashJoins.
- **Native.** It agrees with Clara on all three shapes. Do not touch it.
- **General truth maintenance.** The measured defect is accumulate results specifically. A full TMS
  for the oracle is a different, much larger thing; do not start it here.
