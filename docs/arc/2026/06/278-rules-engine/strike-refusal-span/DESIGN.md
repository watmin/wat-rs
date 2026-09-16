# DESIGN-STONE — threading the span is the cure AND the guard

> **Origin (2026-09-01).** Class E5 of `VIGILIA-2026-08-30-WORK-LIST.md`, found by `conformare`.
> Driven here at HEAD `30f78ebdf`.

## Why

`refuse_export_without_arm` is a user-reachable refusal. Both its call sites stamp
`rust_caller_span!()`:

| site | enclosing fn | its span param |
|---|---|---|
| `fire/rules.rs:658` | `fire_rules_on_session(session, sym, support)` | **none** |
| `fire/mod.rs:1036` | `fire_once_session(session, sym)` | **none** |

So the user is pointed at a line in `src/rete/kernel/fire/*.rs` — a Rust file they did not write.

**And the real span is one frame up, in hand, already used.** Verified at all three non-test
entries:

```
eval_fire_rules_native   (args, list_span: &Span, env, sym)   ← uses list_span for ArityMismatch
eval_fire_once_native    (args, list_span: &Span, env, sym)   ← same
eval_fire_rules_explain  (args, list_span: &Span, env, sym)   ← same
```

Every real caller holds the wat span and throws it away at the boundary.

## ⛔ WHY THE LINT DOES NOT CATCH THIS, AND WHY IT MUST NOT BE WIDENED

`span_substitution_justified` flags a fn that **(a)** has a used `…span: &Span` param and **(b)**
calls `rust_caller_span!()`. Its header states the exclusion as a principle:

> *"A **leaf** with no wat span in scope… Those sites have no span param, so they never match. This
> lint is about the CHOICE between a real location and a Rust one, **never about the absence of a
> choice**."*

The principle is right. **The predicate is a syntactic proxy for it** — "has no span param" stands
in for "has no wat span available" — and here the choice exists one frame up, so the proxy admits a
site the principle would refuse.

**Do not widen the lint.** Measured before proposing it: **534** sites tree-wide are a span-less fn
stamping `rust_caller_span!()`, **71** of them in `src/rete` — and the visible majority are test
helpers and internal leaves, exactly the population the exclusion exists to protect. Separating the
real defects from them needs caller analysis, and this project has a recorded failure mode for
precisely that (**FM 33 — a static audit of a call graph is wrong in both directions, and looks
right each time**). A lint that guesses across frames would be a new source of false findings.

## ★ THE ONE CONTRACT DECISION

**A refusal a user can reach takes the span the user wrote — so the function that produces it takes
a span parameter.** Threading is not merely how the diagnostic gets fixed; **it is how the site
becomes visible to the gate that already exists.** After this strike both fns have a used
`span: &Span`, so any future `rust_caller_span!()` in either body is caught by
`span_substitution_justified` with no new lint and no widened predicate.

The proxy is not defeated — it is **made true**.

## Blast radius — nine call sites, not two

`fire_rules_on_session` has **nine** callers, enumerated rather than assumed:

| where | n | what it passes |
|---|---|---|
| `fire/rules.rs`, `fire/mod.rs`, `fire/delta.rs` | 3 | the real `list_span`, already in hand |
| `kernel/tests/{strat,fanout,gather_probe,cascade,accum}_cost.rs` | 8 | a synthetic span — **legitimate; tests are leaves** |

Plus `fire_once_session` (1 caller) and `refuse_export_without_arm` (2). Files:
`fire/rules.rs`, `fire/mod.rs`, `fire/delta.rs`, and five test modules.

## Out of scope — AFFIRMATIVELY CUT

- **Widening `span_substitution_justified`.** Rejected above, with the measurement and the recorded
  failure mode. **Record the blind spot AT the lint instead**, with the 534/71 numbers, so the next
  reader knows the exclusion is a proxy and knows the population it protects.
- **E1** (`check_field_at`'s doc promises the field's span; both callers pass the clause's). Driven
  here — the caret spans **cols 31–76**, 46 characters, where the offending keyword is at **col 65,
  length 10** — and it is the NEXT strike, not this one. Different file, different cure, and a stone
  covering two mechanisms invites one ambiguous mutation.
- **The other 71 rete sites.** Overwhelmingly test helpers and leaves. Naming them as a sweep would
  be a count, not a finding, and this list already carries a warning about counts with no proposals.
