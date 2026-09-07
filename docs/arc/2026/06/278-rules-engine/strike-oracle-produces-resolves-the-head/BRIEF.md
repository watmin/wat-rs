# BRIEF — teach the oracle's `rule-produces` to resolve the head, using compile.wat's own recipe

## The work

`wat/rete/oracle/stratify.wat`'s `rule-produces` names the FUNCTION when a `:then` head is a user
fn, so the sweep raises the wrong key and the oracle drops a derived fact. `wat/rete/compile.wat`
already resolves that head correctly, ~700 lines away in the same subsystem. Port that resolution
into `rule-produces` and delete the colon-strip.

## Read in order

1. `docs/arc/.../strike-oracle-produces-resolves-the-head/DESIGN.md` — the driven numbers and why
   the recipe needs no fn/record discrimination.
2. `wat/rete/compile.wat:775-798` — **the recipe to port.** `eval-ast!` the head, test
   `(:wat::core::type …) == "wat::core::fn"`, else re-resolve through the PRIME `:T'` keyword,
   then `(:wat::runtime::return-type-of head-fn)`. Note its two comments: `return-type-of` raises
   "unknown type" itself for an unrecognised head (no separate check needed), and it **already
   returns a colon-free FQDN — do not re-prepend a colon and do not strip one.**
3. `wat/rete/oracle/stratify.wat:46-67` — `rule-produces` as it stands. The `raw-nm` /
   `string::subs` colon-strip is what goes.
4. `wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat` — **run it before and after**
   (`cargo run --release --bin wat -- <path>`). Before: `ORACLE facts [0 1 0]`. After: `[0 1 1]`.
5. `src/rete/kernel/tests/produced_type_userfn.rs` — the two assertions that must flip, and the
   ANCHOR that must not move.

## Sketch

Inside `rule-produces`' fold, replace the `type-hd` / `raw-nm` / `type-nm` colon-strip with the
compile.wat shape:

```
(:wat::core::let [head      (:wat::core::first (:wat::core::ast->children form))
                  head-val0 (:wat::core::Result/expect (:wat::eval-ast! head)
                              "rule-produces: :then item head failed to evaluate")
                  is-fn-val (:wat::core::= (:wat::core::type head-val0) "wat::core::fn")
                  prime-kw  (:wat::core::keyword-node
                              (:wat::core::string::concat (:wat::core::ast-name head) "'"))
                  head-fn   (:wat::core::if is-fn-val head-val0
                              (:wat::core::Result/expect (:wat::eval-ast! prime-kw)
                                "rule-produces: :then item head failed to resolve to a fn"))
                  type-nm   (:wat::runtime::return-type-of head-fn)]
  (:wat::core::PersistentVector/conj acc type-nm))
```

Keep the fold and the return type of the defn exactly as they are.

## Then flip the gate

Both assertions in `produced_type_userfn.rs` invert from DIVERGE to AGREE — native and oracle both
`pt::Rate`, both `[0 1 1]`. Update the messages too: a message still saying "oracle names the
function" under an assertion that now demands agreement is a lie the next reader will trust.
**Leave the ANCHOR assertion alone** and say in the test header why it is the non-vacuity guard.

## ⛔ The floor is the instrument, and other tests MAY move

This changes ORACLE BEHAVIOUR. The grid three-way, `wat_scripts_grid_port_check`, the TMS fuzzer
and every `$oracle` probe take their reference from it. If one goes red, **do not "un-break" it** —
capture it whole, name the arm, and ask of that test: *was it asserting the oracle's wrong answer?*
That is a finding worth more than this cure. Surface it before changing a single expectation.

## STOP triggers

1. **If `eval-ast!` cannot resolve a head at stratify time** (it works at compile time in
   `compile.wat`) — STOP and report the raise verbatim. Do NOT reinstate the colon-strip as a
   fallback; a fallback restores the defect exactly when resolution fails, which is the shape this
   cure exists to remove.
2. **If the ANCHOR moves** — `:pt::plain` must stay `pt::Rate` on both sides. If it does not, the
   recipe is wrong for record heads and the PRIME branch is not doing what the DESIGN claims. STOP.
3. **If any other test reddens** — STOP, capture whole, name the arm, surface it. See above.
4. Do not touch `produced_type` (native is not the defect), `rule-negates`, `rule-consumes`, or
   `purity.rs`.

## Blast radius

`wat/rete/oracle/stratify.wat` (one fold body) and `src/rete/kernel/tests/produced_type_userfn.rs`
(two assertions plus their messages). `wat/` is `include_str!`'d — rebuild, and drive with
`cargo run --release --bin wat`, never the installed binary.

## Prior comparable

`16f504e14` — the last time the oracle was the wrong one and the cure went into the oracle's own
route rather than porting native's. Read its message for the standard of evidence expected.
