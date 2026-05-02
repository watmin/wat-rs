# Arc 137 — `/dissimulans` spell

**Status:** opened 2026-05-03. **Block on arc 136 closure** —
the spell needs the `(:wat::core::do ...)` form as the recommended
fix. No reason to find dissembling let*s if there's nothing
better to recommend.

## TL;DR

Mint `/dissimulans` — the third spell in the wat-rs grimoire (alongside `/perspicere`, `/vocare`, `/complectens`). The spell finds let* bindings that DISSEMBLE: chains of `((_ :wat::core::unit) FORM)` that pretend to be bindings while actually being sequencing. The mechanical phase scans for these chains; the judgment phase distinguishes pure sequencing (collapse to `do`) from mixed-binding let*s (leave alone).

## The name

`dissimulans` — present active participle of *dissimulare*: "to dissemble, to feign not to be what one is." The let* binding `((_ :wat::core::unit) FORM)` dissimulates — pretends to be a binding while actually being sequencing.

Cognates in English: dissemble, dissimulation.

Naming pass via `/gaze` (transcript 2026-05-03):

| Candidate | Verdict |
|---|---|
| `monens` (warning) | Level 2 mumble — too generic |
| `celans` (concealing) | Passes; weaker English cognate |
| `expediens` (untangling) | Poetic but obscure |
| `mendax` (lying) | Wrong shape (adjective, not participle) |
| **`dissimulans`** | **Wins — passes all four questions** |

Pairs with `/complectens`:
- `/complectens` — names the GOOD state (woven, properly composed)
- `/dissimulans` — names the BAD state (dissembling, feigning composition)

## What the spell finds

```scheme
;; ❌ Dissimulating — `_` names what isn't a binding.
(:wat::core::let*
  (((_ :wat::core::unit) (:wat::test::assert-eq v1 e1))
   ((_ :wat::core::unit) (:wat::test::assert-eq v2 e2)))
  (:wat::test::assert-eq v3 e3))

;; ✓ Honest — sequencing intent, sequencing form.
(:wat::core::do
  (:wat::test::assert-eq v1 e1)
  (:wat::test::assert-eq v2 e2)
  (:wat::test::assert-eq v3 e3))
```

The `_` binding LIES. It claims to bind a unit value but actually exists only to keep let*'s syntactic shape. Every chain of two or more such bindings is a dissimulation. The spell finds them.

## What the spell DOES NOT find (Level 3 taste)

A let* with REAL bindings interspersed with `((_ :unit) ...)` calls is NOT a violation:

```scheme
;; ✓ Mixed bindings — let* is correct here; do can't replace
;;    it because we need real names in scope for later forms.
(:wat::core::let*
  (((handle ...) (HandlePool::pop pool))
   ((_ :wat::core::unit) (:my::log "popped"))
   ((result ...) (:my::work handle)))
  (:my::process result))
```

The spell flags PURE sequence-only let* heads. Mixed-binding let*s pass.

## Severity levels

- **Level 1 — Lies.** Pure dissimulation: a let* whose ENTIRE binding head is a chain of `((_ :wat::core::unit) ...)`. Should be `(:wat::core::do ...)`. Always report.
- **Level 2 — Mumbles.** Mostly-dissimulation: 3+ unit-bindings in a let* head with one or two real bindings. Probably refactorable; phase-2 judgment.
- **Level 3 — Taste.** A single `((_ :unit) ...)` interleaved with real bindings. Not worth refactoring.

## Phase-1 mechanical survey

Scan target wat files for let* heads. For each:

1. Parse the binding-list (the second element of the let* form).
2. Count entries matching the shape `((_ :wat::core::unit) RHS)` and entries that don't.
3. Report:
   - All-unit-bindings of length ≥ 2 → Level 1 candidate.
   - Mixed but mostly-unit → Level 2 candidate.
   - One or zero unit-binding entries → no finding.

Output: `(file, line, deftest-name, unit-binding-count, real-binding-count, severity-candidate)`.

Same shape as `/complectens` phase 1 — paren-balanced extraction; line numbers; sorted by severity.

Implementation: shell/awk/python today; eventually `wat-lint` rule.

## Phase-2 judgment

Read each candidate. Apply the four questions to the let* head:

- **Obvious?** What does each `_` name? Nothing — it's syntactic glue. Lie.
- **Simple?** Five lines of binding ceremony for what should be three.
- **Honest?** `_` pretends to be a binding; isn't.
- **Good UX?** Reader must mentally translate "let* with `_`" → "sequence."

If all four are NO → fire Level 1 finding; recommend `(:wat::core::do ...)` rewrite.

If even one is honestly YES (e.g., the rest of the let* uses real bindings; the `_` chain is a localized cluster) → Level 2/3.

## Implementation

Same shape as `/complectens`:

- `.claude/skills/dissimulans/SKILL.md` — the spell's body. Frontmatter, etymology, what to find, severity levels, phase 1 / phase 2, four questions.
- `.claude/skills/dissimulans/dissimulans.wat` (FUTURE) — the wat-side mechanical phase, runnable as `cat <target> | wat .claude/skills/dissimulans/dissimulans.wat`. Same queued-arc framing as `/complectens`.

## Slice plan

1. **Slice 1** — write the SKILL.md.
2. **Slice 2** — phase-1 implementation (shell/python today; pluggable for wat-lint later).
3. **Slice 3** — first cast across the codebase. Find candidates. Confirm the discipline catches what we expect.
4. **Slice 4** (optional) — sweep arc whose actual job is rewriting candidate let*s to `do`. Likely overlaps with arc 136 slice 2 (the `do`-form sweep). Combine if convenient.

## Cross-references

- `.claude/skills/complectens/SKILL.md` — sibling spell; established the shape.
- `docs/arc/2026/05/136-core-do-form/DESIGN.md` — the form this spell recommends.
- Memory `project_spell_as_linter.md` — the architecture: each spell is one wat-lint rule.

## When this matters

After arc 136 lands `(:wat::core::do ...)`, the codebase has the honest form available. Without `/dissimulans`, future test code can drift back into the let*-with-unit-bindings crutch — there's no automated check stopping it. The spell IS the check.

The `/dissimulans` cast becomes part of the standard `/wards` (or whatever the meta-spell convention is) once it ships.
