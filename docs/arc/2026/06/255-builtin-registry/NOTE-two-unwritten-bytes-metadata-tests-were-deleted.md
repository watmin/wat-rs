# NOTE (arc 255) — two UNWRITTEN `Bytes` metadata tests were deleted, and the reflection frontier is FOUR, not six

**Filed 2026-08-16.** Builder: *"remove the unwritten tests.. leave a NOTE-<slug>.md in their
respective arcs.... we'll deal with them when we go to close the arcs... not a now thing."*

## What was deleted

Two `#[test]` functions in `tests/reflection/probe_arc255_reflection_parity.rs`, both with
`unimplemented!()` bodies:

- `metadata_of_answers_for_bytes_to_hex_intrinsic`
- `metadata_of_answers_for_bytes_from_hex_intrinsic`

Both carried the arc's standard reason — *"RED-at-HEAD: arc-255 metadata-of reflection
(builtin-registry) not yet built; unlock when we circle back to arc 255"* — which was **wrong about
these two in two different ways.**

## ★ CORRECTION 1 — they were not "not yet built". They were not yet WRITTEN.

`unimplemented!("arc 255: metadata-of for Rust intrinsics; on unlock assert the exact :doc for
Bytes::to-hex")`. No arc closing could turn them green; there was no assertion to turn.

## ★ CORRECTION 2 — the thing they were waiting for ALREADY ANSWERS.

`metadata-of` has answered live for registered intrinsics since **`7b99d123` (2026-06-21)**. Verified
again this session through the `wat` MCP:

```clojure
(:wat::runtime::metadata-of :wat::core::Bytes::to-hex)
⇒ #wat.core.Option/Some [{:name :wat.core.Bytes/to-hex  :arity 1
                          :kind #wat.runtime.Kind/Intrinsic []
                          :defined-in #wat.runtime.DefinedIn/Rust []
                          :layer #wat.runtime.Layer/Substrate []
                          :purity #wat.runtime.Purity/Pure []
                          :determinism #wat.runtime.Determinism/Deterministic []
                          :category #wat.runtime.Category/Encoding []
                          :doc "Encode a `:wat::core::Bytes` into its lowercase-hex `:String`. …"
                          :added "1.0.0"  :ret "…"}]
```

That is exactly what these two would have asserted — *"the exact `:doc` for `Bytes::to-hex` /
`from-hex`"*. **The capability shipped two months before the tests that were meant to prove it, and
the tests were never written.** See `NOTE-arc-255-IS-HALF-BUILT-the-june-registry.md`, which found the
same inversion at arc scale.

## ★ CORRECTION 3 — MY OWN CENSUS WAS WRONG, and it is worth writing down

I told the builder that arc 255's reflection group was **six blocked tests** and called them "255's
actual frontier". Measured properly, `probe_arc255_reflection_parity.rs` is **2 real, 2 unwritten**:

| test | body |
|---|---|
| `metadata_of_answers_for_a_rust_builtin` | REAL — `assert!` over `builtin_metadata_is_some()` |
| `user_form_carries_guaranteed_baseline` | REAL — `assert!` over `user_form_metadata_is_some()` |
| `metadata_of_answers_for_bytes_to_hex_intrinsic` | ⛔ `unimplemented!()` — DELETED |
| `metadata_of_answers_for_bytes_from_hex_intrinsic` | ⛔ `unimplemented!()` — DELETED |

So the reflection frontier is **FOUR** rows, not six:
`metadata_of_answers_for_a_rust_builtin` · `user_form_carries_guaranteed_baseline` ·
`probe_arc255_ivc_metadata_plain_values::metadata_of_emits_plain_values_and_enums_not_holon_ast` ·
`probe_arc255_ivb2b_verify_examples::verify_examples_reports_no_failures`.

**A count that mixes written tests with placeholders cannot tell you what work remains.** Third
instance today of a census being wrong because it counted rows instead of reading bodies.

## What 255 still owes here — and it is NOT these tests

The live question is not *"does `metadata-of` answer for `Bytes::to-hex`"* (it does). It is the one
`NOTE-arc-255-IS-HALF-BUILT` names: **`metadata-of` has TWO tables and two branches** — the intrinsic
branch reads `crate::intrinsic::registry().lookup_entry(&name)`, the user branch reads
`sym.binding_metadata` — and `DESIGN.md`'s REFRAME calls exactly that *"the opposite of seamless"* and
rules **"The registry IS `sym`."**

If you want a `Bytes::to-hex` `:doc` assertion, write it against whichever table the arc RULES on.
Writing it now would pin the June path as correct and pre-empt the entry-shape decision this arc
reserves as DAY ONE. **Do not resurrect these from git.**

## Also still true, and unaffected

The two `probe_undefined_builtin_resolves` gates were REPAIRED (not deleted) the same day: their
fixtures declared `-> :wat::core::i64` and arc 170's main-signature check killed them before the
checker reached the call head, so they passed vacuously. Fixed to `-> :wat::core::nil`, they now
freeze clean and FAIL honestly against the still-live blanket-accept at `src/resolve/walk.rs:257` —
which is the soundness hole this whole arc exists to annihilate. Recorded in `d01fe67c`.

## Kin

- `NOTE-arc-255-IS-HALF-BUILT-the-june-registry.md` — the same inversion at arc scale.
- `docs/arc/2026/05/170-program-entry-points/NOTE-the-walker-disconnect-suspicion-was-false.md` and
  `docs/arc/2026/05/214-concurrency-toolkit/NOTE-three-unwritten-crash-diagnostic-tests-were-deleted.md`
  — the sibling deletions, same day, same class.
