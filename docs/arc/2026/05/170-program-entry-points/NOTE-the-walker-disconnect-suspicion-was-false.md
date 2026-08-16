# NOTE (arc 170) — the "BareLegacyMainSignature walker no longer fires" suspicion was FALSE, measured

**Filed 2026-08-16.** Builder: *"170 is completely done as i understand it... what are those tests?...
almost assuredly they either work or they can go."* They worked. The suspicion did not.

## What was deleted

Two `#[test]` functions in `tests/program/wat_arc170_program_contracts.rs`, both with
`unimplemented!()` bodies:

- `t1_legacy_3arg_main_fires_walker`
- `t11_legacy_main_signature_fires_walker_diagnostic`

Both carried this `#[ignore]` reason:

> *"ARC-170 WIP: BareLegacyMainSignature walker no longer fires for a non-canonical `:user::main`
> (freeze succeeds where it should reject — likely walker-disconnect); investigate + fix/retire
> before arc 170 closes."*

## ★ THE SUSPICION IS FALSE. The walker fires. Measured 2026-08-16.

`:user::main` with three parameters, run through `target/release/wat --check`:

```
#wat.macro/MainSignatureError {
  :message ":user::main must take exactly 0 parameters; got 3. Arc 170 slice 1e
            (REALIZATIONS pass 7) — `:user::main` takes no arguments. argv is ambient via
            `(:wat::runtime::argv)`; stdio is mediated by the three substrate services
            (slice 1f's StdInService / StdOutService / StdErrService).
            The canonical signature is `[] -> :wat::core::nil`." }
```

The guard is live and correct at **`src/check.rs:906–914`** — and it is not a narrow legacy-arity
check, it is the canonical-shape check:

```rust
// Arc 170 slice 1e — fire on anything that's NOT the canonical
// post-slice-1e shape: empty params + return-type `:wat::core::nil`.
let canonical_params = param_types.is_empty();
let canonical_ret    = matches!(ret_type.as_deref(), Some(":wat::core::nil"));
if canonical_params && canonical_ret { return; }
errors.push(CheckError { span: main_span, kind: CheckErrorKind::BareLegacyMainSignature });
```

`CheckErrorKind::BareLegacyMainSignature` is defined at `src/check/error.rs:288`, rendered at `:694`,
pushed at `src/check.rs:914`. Nothing is disconnected.

**Both arms were observed firing the same day**, independently and for unrelated reasons:

- the **params** arm — this note's probe, `got 3`.
- the **return-type** arm — hit while repairing two arc-255 fixtures that declared
  `-> :wat::core::i64`; the check killed them before the type-checker ever reached their call heads.
  (That is recorded in `d01fe67c`; it is the reason those two fixtures were vacuous.)

## ★ WHY THIS ONE IS WORTH A NOTE — the prose outlived the check it doubted

No test was ever written. A **suspicion about a substrate regression was typed into an `#[ignore]`
reason instead**, and that string then sat on disk as the only account of a walker that had been
correctly rejecting non-canonical mains the entire time. It named a mechanism
(`BareLegacyMainSignature`), a symptom (*"freeze succeeds where it should reject"*), and a diagnosis
(*"likely walker-disconnect"*) — and **nobody ever asked the binary**. One `--check` invocation
dissolves all three.

A reason string is not a finding. It is a claim with no instrument behind it, and it inherits the
authority of the code it is attached to.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

## Coverage after the deletion — 170 is NOT left with a hole

The canonical signature is still asserted in the same file: the T-series validator calls
`validate_user_main_signature(&world)` and pins `expected_user_main_signature()` to 0 params and
`TypeExpr::Tuple(vec![])` (nil) return. And the rejection path has live, incidental coverage —
`tests/wat_lang/probe_undefined_builtin_resolves_*.wat.bad` reach the return-type arm as a side
effect (which is exactly how it was caught).

**Nothing is owed here.** These two were placeholders, not coverage, and their removal takes no
assertion with it.

## For arc 170's closure

The reason strings said *"investigate + fix/retire before arc 170 closes."* **Investigated. Nothing
to fix. Retired.** If a future reader wants a dedicated positive test for the rejection arm, it is a
ten-line `startup_from_file` on a 3-arg main asserting the `MainSignatureError` — but write it
because a rejection test is wanted, not because this note left a gap.

## Kin

- `docs/arc/2026/05/214-concurrency-toolkit/NOTE-three-unwritten-crash-diagnostic-tests-were-deleted.md`
  — the sibling deletion, same day, same class: `unimplemented!()` bodies wearing `#[ignore]`.
- `docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-STONE-K-ignore-means-one-thing.md` — K split
  `#[ignore]` into *blocked* vs *deliberately outside the floor*. These were a third kind it did not
  name: **never written** — and this one adds a fourth wrinkle, *never written, and asserting a
  substrate defect in its reason string*.
