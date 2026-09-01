# SCORE — A5, weighed against the orchestrator's own re-run

> Re-run here at `7e24c3257`.

## The scorecard, re-run

| # | pre-value I measured at HEAD | after |
|---|---|---|
| 1 | repro `"Compiled"` | ✅ still `"Compiled"` — behaviour unchanged, as the ★ required |
| 2 | `grep` export.rs → 0 | ✅ still 0; **now a lint row instead of a grep** (finding 5) |
| 3 | one caller | ✅ unchanged; **now a lint row** |
| 4 | 3 × `Ok(())` | ✅ the fn returns `TerminationVerdict`; no `Ok(())` carries a termination meaning |
| 5 | `:894`/`:988` are proofs | ✅ **and conditionally so** — see the ★ below |
| 6 | `NotAnalysable` reachable | ✅ four ways, all driven |
| 7 | the sentence | ✅ qualified, naming **two** false doors (finding 4) |
| 8 | no `?` at the caller | ✅ `Refused` still flows to `compile_result_to_outcome` |
| 9 | radius | ⚠ **+1 file**, correctly — finding 5 |
| 10 | lint 114/114 | ✅ **116/116** (two new rows) |
| 11 | floor 5203/5203 | ✅ `Summary [ 407.065s] 5213 tests run: 5213 passed (1 slow), 21 skipped`, zero FAIL rows — exactly the rider's predicted 5,213 |
| 12 | clippy rc=0 | ✅ rc=0 |

**The discriminating mutation, re-driven here** — collapse only the graph-closed exit to `Proven`:

```
Summary  8 tests run: 7 passed, 1 failed
  FAIL  a_skipped_rule_beside_a_computing_one_is_not_analysable
  PASS  a_skipped_rule_beside_a_non_computing_one_is_not_analysable   ← rides the early exit
```

**Exactly one probe reddens.** The two arms are cleanly separated and the probes sit on the arms
they claim. Trap 1 is defeated — which matters, because trap 1 as I wrote it was wrong.

## ★ THE RIDER CORRECTED MY STONE'S SEMANTICS, AND WITHOUT IT THE STRIKE CHANGED NOTHING

My DESIGN table classified `:894` and `:988` as *"yes — proven"*, **unconditionally**. The rider
read that against the strike's own repro and found the hole: one AST-less rule reaches `:894` with
`edges` empty, so an unconditional `Proven` would have had `refuse_non_terminating` **still**
answering "proven" for a set nothing looked at. The split would have shipped and the defect would
have stayed exactly where it was.

The cure is `Proven` **iff zero skips**, else `NotAnalysable { n }` — the skip count *taints* both
proofs. Trap 2 survives because the 371-of-381 corpus rules have ASTs and skip nothing, pinned by
`a_rule_set_that_computes_nothing_is_proven` and mutation-proven able to go red.

**I classified each return site by what it structurally WAS, not by what it KNEW given a skip.**
Promoted to memory: a proof over a filtered population is not a proof.

## ⛔ Where MY brief was thin — six, and three are mine to own

- **A. ★ The DESIGN table's semantics would have left the defect alive.** Above. This is the second
  strike running where the stone's own classification, not the code, was the load-bearing error.
- **B. ★ Trap 1's stated MECHANISM is wrong.** I wrote *"if EVERY rule lacks an AST, `edges` is
  empty and the `:894` early exit fires."* The predicate is `edges.iter().all(|e| e.computed
  .is_none())` — it keys on whether any edge **computes**, not on emptiness. A mixed set of
  {AST-less rule, rule that derives but does not compute} takes the early exit too, with a
  non-empty `edges`. The real discriminator is a **computed head**. My step happened to land the
  rider in the right place; my mechanism would have sent a literal reader back onto `:894`. The
  rider repeated my wrong mechanism in its own test doc before re-deriving the predicate — which is
  how a wrong explanation propagates even when the instruction is right.
- **C. ★ DESIGN's table has four sites; there are five.** `:843` — `rule_named_field(r, "name")` →
  `None` → `continue` — is the same silent skip, now counted, and reported honestly as
  **reachable but not driven** (the wat-side vector is typed `[:wat::rete::Rule]`, so the rider
  found no wat expression that puts a non-`Rule` in it). **Third consecutive strike where I
  under-enumerated the sites** — A6 one tower of three, D3 one call site of six, now four of five.
- **D. My EXPECTATIONS mutation was a single mutation over a multi-arm gate** — "collapse
  `NotAnalysable` into `Proven`" is ambiguous across four sites, and any one alone leaves the others
  unproven. The rider ran six, each predicted before running, each with a distinct red set. **This
  is my own recorded lesson, violated while writing the scorecard rather than while reading a
  gate**; the memory is updated with the instance.
- **E. STOP-3 did not fire, and the answer was checkable all along.** I offered an escape hatch for
  the hand-assembled `Session` claim. The rider verified it instead:
  `probe_arc278_1a_data_model.wat:13` builds a `Session` directly in wat, and `fire_rules_on_session`
  reaches its arm through `rete_arm_get_or_build`, which never calls the verifier. **So `arm.rs:1294`
  had TWO false doors, not one** — and both are now named. A STOP trigger is a rejection criterion,
  not a licence to stop looking; the rider treated it correctly.
- **F. Row 9's radius excluded the file the repo's own law requires.** My qualified sentence rests
  on two totalities ("exactly one call site", "export.rs has no hit").
  `tests/lint/rete_header_claims_are_asserted.rs:33` states: *"if you cannot gate it, do not assert
  a totality about it."* The rider added two rows rather than asserting prose, turning my
  grep-shaped scorecard rows into permanent gates. **+1 file, and it is correct** — verified
  against the lint's own header. Accepted.

## The red the rider caused and cured itself

Its first call-site gate scanned all of `src/` and counted the seven probe calls in
`src/rete/kernel/tests/` as doors — 8 callers, gate red. Caught by the mandated
`binary_id(wat::lint)` run, **which is exactly the check my tier note added two strikes ago after a
floor red a scoped probe could not see.** The instruction paid for itself.

## Arms not driven, named

`:843` (rules element with no `name`) — **reachable but not driven**, named above with the reason.
Everything else — all four DESIGN sites, `Refused`, and all three caller arms — **proven**, each by
a mutation with a predicted distinct red set.
