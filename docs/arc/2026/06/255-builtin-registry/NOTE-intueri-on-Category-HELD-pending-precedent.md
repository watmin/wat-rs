# NOTE — `intueri` on the `Category` taxonomy: the verdict, HELD, pending precedent

**Cast 2026-08-19** against `wat/runtime-meta.wat`'s `Category` defenum, ward text embedded verbatim
from the signed datamancy MCP, told explicitly that the orchestrator's leanings were the
orchestrator's and to refute them freely, and that **row count is never an argument for keeping a
name** (builder: *"we do not fear the cost of refactors — we fear dishonesty and incorrectness"*).

## ⛔ BUILDER'S RULING — NOT ACTED ON

> *"uhm.... i don't know that i agree.... we'll revisit this once we have more precedence.... we
> continue with the names we have as seek failures to classify as we move forward."*

**Nothing in this note was applied.** No variant renamed, no prose edited. The verdict is recorded so
it is findable when precedent arrives, and so the next reader does not re-cast the same ward and
re-derive the same findings. **Do not treat any line below as a pending TODO** — it is a held
opinion, and the standing method is now the opposite one: **carve, and let a verb that will not
classify be the evidence.**

## The method the ruling establishes, which is the load-bearing part

A naming argument made in the abstract is taste. A verb that cannot be honestly filed is data. From
here, every carve is also a **classification-failure hunt**: when a body's DOING does not fit any of
the fifteen, that is the finding, and it is worth more than any amount of a-priori reasoning about
the names. Precedent accumulates; then the taxonomy gets ruled on evidence.

## What the ward said, for the record

**The rule it derived** — three legitimate families, one test:

- **Action-nouns** — `Transform`, `Probe`, `Reflection`, `Declaration`, `Projection`, `ControlFlow`,
  `Binding`. Suffix is incidental (bare stems and `-ion` both); what matters is that the word's
  ordinary sense tracks the technical one.
- **Domain-nouns** — `Arithmetic`, `Io`, `Message`, `Resource`. Legitimate where no single verb spans
  the family, so the shared object/medium stands in.
- **Quality-adjectives** — `Entropic`, `Ambient`. Precedented by the sibling `Purity`/`Determinism`
  enums — but ⚠ **borrowed silently**: `Category`'s own header never licenses adjectives as a form.
- **Illegitimate** — agent/mechanism nouns. `:Accessor` died here. (`Projector`, floated on the day,
  is the same defect wearing a better word.)

> **The one test: does the identifier, read cold with zero surrounding prose, collide with an
> unrelated common-noun sense?** Fourteen of fifteen pass.

**Its one rename** — `Combine` → `Combination`. The argument that survived its own counter-case: the
bare unglossed form DOES exist on disk, at `crates/wat-doc/src/lib.rs`'s `CATEGORY_LEGAL_VALUES`
(a comma-joined string a caller reads in a real error message) and in both macro match arms. Those
are the "reader with no context" sites. The orchestrator's counter — *"`Combine` never appears
without its prose"* — was checkable and false.

**Three prose findings, HELD:**

- **`CheckGate` — Level 1.** Its prose asserts *"One member today"* and names `require-wire-address`.
  That verb is a raw string-matched arm (`check.rs`, `runtime.rs`); it is **not registered and carries
  no `@Category`**. Actual membership: zero. Independently verified. A claim shipping as `///` API
  documentation that the disk contradicts.
- **`Entropic` — Level 2.** Its justification reads *"the verb PRODUCES AN ENTROPIC VALUE"* — a
  return-value framing, inside the rationale for a header that rejects "what it returns" as an axis.
  The classification is sound (*samples an unpredictable source*); the defending sentence reaches for
  a rejected axis. **Written by the orchestrator the same day, while quoting that reject list.**
- **`Ambient` — Level 2.** Its boundary test — *"the `sig*?` queries take no input value to
  interrogate… not a fact about something the caller holds"* — is the SOURCE axis, also on the
  header's reject list. And all seven members are POSIX-signal accessors, so the name claims a genus
  the evidence supports only a species of.

**What it could not judge** (missing rulings, named not invented): whether the header's
"not where its input comes from" rejection was ever meant to reach `Ambient`'s test; whether a
non-signal `Ambient` member is anticipated; whether `Resource`'s 13 named verbs are scheduled for
`@Category` or were minted years ahead.

## ★ THE WARD CAUGHT A DEFECT IN THE ORCHESTRATOR'S OWN CENSUS — which is why it was worth casting

The cast was given a tenant census **as ground truth**. That census was wrong: it reported
`Binding 0` and `ControlFlow 1`. The ward went to the disk and found `special/binding.rs` tags `let`
and `special/control_flow.rs` tags `if`.

**Mechanism:** the orchestrator's pattern was `@Category      X` — six spaces, matching the *aligned*
intrinsic files. `src/intrinsic/special/*.rs` writes `@Category Binding` with ONE space, so the
pattern silently skipped every special form. The correction then over-swung and swept in maintainer
comments that merely *mention* the directive. **Three patterns, three different wrong answers, one
population.** `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

**The reconciled census, 2026-08-19** — 70 `#[wat_intrinsic]` + 2 `#[wat_special_form]` = **72
registered forms, and exactly 72 `@Category` declaration lines.** It balances, which is the check:

```
Transform 25 · Entropic 17 · Ambient 7 · Io 6 · Message 5 · Reflection 4
Projection 3 · ControlFlow 2 · Arithmetic 2 · Binding 1              = 72
zero-tenant: Probe · Combine · Declaration · Resource · CheckGate
```

⚠ Commit `f87691e9`'s message records **73** and omits `Binding`. That figure is wrong; this note is
the correction. What is inscribed is inscribed — the commit stands as it shipped.

★ **And the joke is exact:** this arc exists because hand-maintained lists drift, and the builder has
ruled that **the registry is the authority**. The orchestrator spent the day grepping for a census
that the registry can answer directly. The arc's own thesis, applied to everything except the
instrument measuring it.
