# EXPECTATIONS — STONE: `:wat::holon::literal` is a special form. Written BEFORE the strike.

| # | what | command | expected | derived from |
|---|---|---|---|---|
| 1 | the kind flips | `lookup-define`/`metadata-of` on the verb | `Kind::SpecialForm` | the reclassification |
| 2 | `apply` still rejects it | probe `apply` with the keyword | same `MalformedForm` diagnostic | room 4 |
| 3 | arity still enforced | `--check` `(:wat::holon::literal a b)` | `ArityMismatch`, expected 1 | check.rs:3272, read |
| 4 | the exception is gone | `grep holon::literal src/runtime.rs` | absent from the `matches!` | room 4 |
| 5 | completeness gate | `-E 'test(special_form)'` + `binary_id(wat)` | green | intrinsic/mod.rs:2802 |
| 6 | the four scoped binaries | as the BRIEF lists | green | 10-file blast radius |
| 7 | full floor | `scripts/floor.sh`, orchestrator, unpiped | 5129 passed, 0 FAIL | current floor |
| 8 | clippy | `-D warnings --all-targets` | 0 | standing |

## Ledger movement — one number moves, and only one

```
GAP_A 49 · GAP_B 42 · DEBT 121 · TYPES_UNCHECKED 10 · registry 552 · corpus 37   ← ALL UNCHANGED
```

The row already exists and already lacks a `CheckEnv` scheme; reclassifying moves it between the
two halves of the DEBT **split** (`intrinsic/mod.rs:2267`: `Kind::Intrinsic, no scheme` −1;
`Kind::SpecialForm, no scheme` +1) without changing the total.

⚠ **That split is the campaign's own open fork** — the SEAM records that DEBT means two different
things and cannot reach zero while it does. This stone nudges it by one and does not fix it.

## Runtime

**25-40 min.** The `role = check` extraction is the bulk; `intrinsic/special/forms.rs` is a
complete worked template and the arm to extract sits directly above the one already extracted.

## Trap doors, named in advance

1. **The `#holon` spelling.** A census on the FQDN returns zero test files. The orchestrator
   published that zero before catching it. Both spellings, every time.
2. **The completeness gate refusing a half-migration.** Expected and healthy — it is the stone's
   proof. The wrong response is a stub fn that satisfies the wall without being reachable
   (`[[feedback_a_green_test_can_prove_nothing]]`); STOP-1 covers it.
3. **`@Purity` drift.** `Pure` looks wrong on an unevaluating verb and the temptation is to fix it
   in passing. STOP-3. Three rows share the question.
4. **A reflection test pinning `__internal/registered`.** Expected; update and report. If MANY
   tests pin it, that is a signal the sentinel is load-bearing in a way the DESIGN did not measure
   — surface it rather than mass-editing.
