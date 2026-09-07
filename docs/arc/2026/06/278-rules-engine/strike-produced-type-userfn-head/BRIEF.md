# BRIEF — drive native's `produced`, then try to make the divergence change FACTS

## The work

Two arms. Arm 1 is small and certain; arm 2 is the real question and may legitimately come back
negative.

**Arm 1.** Drive native's `rule_produces` on the same two rules the oracle was already driven on,
and compare. Expected: native says `pt::Rate` for the user-fn head where the oracle says
`pt::first-rate`. Land it as a gate with both arms of the comparison measured — the oracle side via
the wat verb in the same frozen world, exactly as `stratify_numbers.rs` does. **Do not hardcode the
oracle's answer**; that was refuted once already in this strike family.

**Arm 2.** Try to build a rule set where the divergence changes derived FACTS: a rule with a
user-fn `:then` head whose stratum is raised (by negating a derived type), plus a downstream rule
consuming the produced fact type. Native would raise that type's stratum; the oracle would raise
the FUNCTION's name and leave the type at 0, stratifying the consumer below its own producer.

## Read in order

1. `wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat` — **run it first**
   (`cargo run --release --bin wat -- <path>`). It prints the oracle's two answers and carries the
   anchor. ⛔ Not the installed `wat` binary: `wat/` is `include_str!`'d and it goes stale silently.
2. `src/rete/kernel/stratify.rs:65-81` — `produced_type`, the `SymbolTable` lookup and the
   return-type substitution.
3. `wat/rete/oracle/stratify.wat:46-67` — `rule-produces`, first child + colon strip, no resolution.
4. `src/rete/kernel/tests/stratify_numbers.rs` — **the shape to copy for arm 1**: `include_str!` the
   scratch `.wat` so the two halves cannot drift, build views the way `fire/rules.rs` does, drive
   the oracle verb with `eval_in_frozen` in the same world, assert both sides.
5. `tests/rete/probe_arc278_then_user_forms_userfn.wat:1-22` — **read the header in full before
   attempting arm 2.** It records why its fn EXTRACTS instead of CONSTRUCTING: every new-record
   surface expands to `:wat::core::kwargs-construct` / `:wat::core::aggregate-new`, which
   `purity.rs` refuses in this position. That fence is what may make arm 2 unconstructible.
6. `src/rete/kernel/stratify.rs:247-253` — the sweep's assignment loop, so you can see exactly
   where `produced` decides which name gets the stratum.

## Sketch — arm 2's target shape

```
  Src(k)                                     [input]
  Bad(k)  :- Src(k)                          produces Bad, stratum 0
  ???     :- Src(k), (not Bad(k))            :then [(:userfn …)] -> returns Rate
                                             native: stratum[Rate] = 1
                                             oracle: stratum[userfn-name] = 1, Rate stays 0
  Out(n)  :- Rate(n)                         native: Out at 1 (after Rate)
                                             oracle: Out at 0 (BEFORE Rate exists)
```

Stratified firing runs each group to fixpoint and threads facts forward without re-firing lower
strata, so on the oracle `Out` would never be derived.

## STOP triggers

1. **If arm 1 shows native and oracle AGREEING** — STOP. That refutes the reading at
   `stratify.rs:65-81` and the whole row dies, which is a fine outcome. Report it.
2. **⭐ If arm 2 cannot be constructed because the purity fence refuses every user fn that yields a
   fact the rule set does not already hold — STOP, and that is OUTCOME 2, a RESULT, not a failure.**
   Report **the exact refusal text for each attempt, verbatim**, and how many distinct shapes you
   tried. "Could not reproduce" is a description of your search, never a disposition. A negative
   here is worth as much as a positive and must be evidenced to the same standard.
3. **If arm 2 DOES produce differing facts** — STOP and surface it before any tidying. That is a
   live oracle defect, Clara becomes the referee, and it outranks the rest of this strike.
4. Do not touch `purity.rs` or its ratchet. Do not change either stratifier. Do not "fix"
   `produced_type` to match the oracle or vice versa — direction is undecided until arm 2 answers.

## Blast radius

A new test file under `src/rete/kernel/tests/` plus its `mod` line; arm 2's fixture if it is
constructible. **No `src/rete/kernel/stratify.rs`. No `wat/`. No `purity.rs`.**

## Prior comparable

`../strike-stratify-numbers-never-compared/SCORE.md` — same two-engine comparison, same
`include_str!`-the-scratch trick, and its REVIEW is why arm 1 must drive the oracle rather than
print it.
