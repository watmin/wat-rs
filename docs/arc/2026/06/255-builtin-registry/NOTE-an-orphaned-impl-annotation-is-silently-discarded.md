# NOTE — an orphaned `#[wat_special_form_impl]` is SILENTLY DISCARDED

> Found while sabotaging 1a-β-i's meter. The sabotage did not fire, and the reason it did not fire
> is a defect in the registry's own ingestion — not in the meter.

## What happens today

`registry()` folds three streams. `SpecialFormImplSubmission` (every `#[wat_special_form_impl]`) is
bucketed by FQDN into `impls_by_fqdn`, and then claimed **only** on the `SpecialFormSubmission` path:

```rust
// src/intrinsic/mod.rs:581   — the IntrinsicSubmission path
impls: Vec::new(),                                              // ⛔ ALWAYS empty
// src/intrinsic/mod.rs:660   — the SpecialFormSubmission path
impls: impls_by_fqdn.remove(submission.name).unwrap_or_default(),
```

**Nothing checks that `impls_by_fqdn` is drained.** An annotation whose FQDN names anything that is
not a `#[wat_special_form]` row is collected, never claimed, and dropped without a word:

- a **typo** in the FQDN
- an annotation naming a **`Kind::Intrinsic`** row (measured: `impls` is hardcoded `Vec::new()` there)
- an annotation naming a **retired** name

The same hole exists for `eval_handler_by_fqdn` and `tail_handler_by_fqdn` — an orphaned
`role = eval` submission carries a real function pointer that is silently thrown away.

## ★★★ Why this is the arc's signature defect, one layer down

The annotation is in the source. The author believes the form has that implementation. The registry
answers *"it has none."* Then `every_special_form_carries_check_and_eval_impls` reports the form as
**missing a role that is annotated three feet away**, and sends the reader hunting for something that
is already there.

**An absence recorded as an answer** — the class this arc has a NOTE family about
(`[[NOTE-an-absence-recorded-as-an-answer-the-class-behind-the-apply-defect]]`), sitting inside the
authority we are making sole.

## How it surfaced — a sabotage that answered the wrong question

1a-β-i's acceptance table said: *"annotate `:wat::string::declare-acronyms` `role = declare` →
FOREIGN names it."* It did not. The branch stayed green, and for a moment that read as a hole in the
meter.

★ It is not. `declare-acronyms` is an intrinsic, so its `impls` is `Vec::new()` by construction and
the annotation evaporated before the meter could see it. **Re-aimed at a registered special form
outside the domain (`:wat::core::if`), the FOREIGN branch fires exactly as designed:**

```
FOREIGN (a Declare impl claimed, name off is_liftable_declaration_head's domain)
must be empty — a role claimed off-domain: [":wat::core::if"]
```

⚠ `[[feedback_a_probe_answers_the_question_you_asked_not_the_one_you_meant]]` — **twice in this one
stone.** The other: a first attempt at the domain-growth sabotage inserted its tenth arm into
`is_mutation_form`, which shares an arm spelling with `is_liftable_declaration_head` forty lines
below. It returned green and nearly became "the meter does not read the source." Correctly targeted,
the meter reports `len 10` and names `zorbletype`.

## The fix, when it is drawn

Assert the three maps are **drained** after the fold — every `impls_by_fqdn`, `eval_handler_by_fqdn`
and `tail_handler_by_fqdn` key must have been claimed by some `SpecialFormSubmission`. A leftover is
an orphan and should be loud.

⚠ Where it goes needs deciding: `registry()` runs inside a `OnceLock` initializer on every startup,
so a `panic!` there is a hard boot failure rather than a test red. A floor gate that rebuilds the
fold and asserts drainage is the cheaper rung and is probably right — but that is a stone, not a
line, and it wants its own four questions.

★ **Not fixed here, and not deferred vaguely: it is named, measured, and reproducible**, and the
meter that found it now stands with both directions sabotage-proven.
