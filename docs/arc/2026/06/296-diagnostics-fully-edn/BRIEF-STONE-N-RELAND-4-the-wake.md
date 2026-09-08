# BRIEF — STONE N RELAND 4: the wake

> 2773 → 473 → 291 → 128 → **69**, clippy 0. The remaining failures are mostly not the migration —
> they are its WAKE, and three of the four classes already have rulings on disk.

## THE 69, BY CLASS

**(1) GOLDEN DRIFT — ruled, recapture.**

```
probe_arc278_peers_bijection ×4     goldens pin a stdlib wat/service.wat LINE; the migration
                                    moved it again
pprintln_doc_row ×2                 byte-goldens over @example rows RELAND 3 edited
metadata_of_example_formats         same population
```

★ `109/NOTE-a-golden-that-pins-a-stdlib-line.md` rules this exactly, and it was written THIS
CAMPAIGN for this recurrence: **RECAPTURE, KEEP PINNING.** A pin discriminates the emitter. ⛔ Do
NOT extend `normalize_rust_source_span_lines` to `wat/*.wat` — that is the retracted option in
stdlib clothes. Verify each recapture is the line number and the edited example text ONLY.

**(2) RESIDUAL POSITIONAL SITES** — `:probe::` · `:user::` · `:wat::service::` · `:usr::`. Same
mechanism as before; the codemod's scream names them. Run it; hand-edit only a site it explicitly
declines, and say why it declined.

**(3) ⚠ A NEW CLASS — ARITY, NOT SPELLING.**

```
constructor :wat::grep::NodeKind::Keyword wants 0 fields, got …    ×6
```

This is **not** "positional retired" and **not** "bare spelling retired". A unit variant is being
constructed WITH a field. Diagnose it: either a wrap gave `{}` a key it should not have, or
`NodeKind::Keyword` is genuinely being called with an argument somewhere. **Do not fold it into the
migration count.**

**(4) ASSERTIONS ON ERROR TEXT** — `try_with_zero_args_rejected_at_check`,
`probe_arc265_acronym_registry`, `typed_if_match::match_arm_type_mismatch_named_by_arm_index`. These
assert a SPECIFIC diagnostic and now get a different one. For each: is the new error the RIGHT error
for what the test measures? If yes, update the expectation and say what changed. **If the new error
is wrong, that is a finding — the wall firing where it should not.**

⛔ Class 4 is where a real defect hides most easily: a test that asserted a precise error and now
gets the wall's message may be telling you the wall is too eager. Read each one; do not blanket-update.

## THE DONE-WHEN

```
0 failed on the floor (ORCHESTRATOR runs it), clippy 0, the five probe rows green
```

State the delta per class, not a total.

## STOP TRIGGERS

- **STOP-1 — a golden is recaptured without checking WHAT moved.** Line numbers and edited example
  text only. Anything else in the diff is a finding.
- **STOP-2 — `normalize_rust_source_span_lines` is extended to `wat/*.wat`.** Retracted option.
- **STOP-3 — the arity class is folded into the migration.** Different reason, different mechanism.
- **STOP-4 — an error-text expectation is updated without asking whether the NEW error is correct.**
- **STOP-5 — the wall is weakened to clear class 4.** If the wall is too eager, that is a wall fix,
  named as such, not a softening.
