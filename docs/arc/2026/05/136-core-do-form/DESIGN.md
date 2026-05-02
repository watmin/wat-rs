# Arc 136 — `:wat::core::do` form (sequential side-effect chain)

**Status:** opened 2026-05-03. **Block on arc 135 closure** before
shipping — this arc requires a sweep of every `((_ :wat::core::unit) ...)`
chain in the codebase, and that sweep is cleanest after the
*complectēns* cleanup ships.

## TL;DR

Mint `(:wat::core::do form1 form2 form3 ...)` as a sequential
evaluation form: evaluate forms left-to-right; return the value of
the last. Replaces the let*-with-`((_ :wat::core::unit) ...)`
crutch that propagates through every test file.

## Provenance

The crutch surfaced 2026-05-03 mid arc 135 slice 1, when the
user noted:

> i think we need a (do ...) form?... [showing the let*-with-
> ((_ :unit)) pattern]... this pattern is a crutch we keep
> leaning on?...

It IS a crutch. Every wat test file uses the let*-with-anonymous-
binding pattern as a poor-man's `progn` / `begin` / `do`. The
binding name `_` LIES about what's being declared (it's not a
binding; it's a sequencing artifact). The four questions all
degrade against this pattern:

- **Obvious?** No — what does `_` name?
- **Simple?** No — five lines of binding ceremony for what should be three.
- **Honest?** No — `_` pretends to be a binding while actually being syntactic glue.
- **Good UX?** No — readers must mentally translate "unused let* binding" → "side effect sequence."

## The pattern in question

```scheme
;; ❌ Today's crutch — let* with anonymous unit bindings.
(:wat::core::let*
  (((_ :wat::core::unit) (:wat::test::assert-eq v1 e1))
   ((_ :wat::core::unit) (:wat::test::assert-eq v2 e2)))
  (:wat::test::assert-eq v3 e3))

;; ✓ With (:wat::core::do ...) — three forms, clean intent.
(:wat::core::do
  (:wat::test::assert-eq v1 e1)
  (:wat::test::assert-eq v2 e2)
  (:wat::test::assert-eq v3 e3))
```

## Naming choice — `do`

Considered:

| Name | Origin | Verdict |
|---|---|---|
| `do` | Clojure | **chosen** — short, modern Lisp idiom |
| `begin` | Scheme | longer; less compact |
| `progn` | Common Lisp | older; "PROGram N" reads cryptic |
| `seq` | various | overloads with sequence types |
| `then` | English | less Lispy |

`(:wat::core::do form1 form2 form3)` reads cleanly. Idiomatic to
Clojure; familiar to anyone reading modern Lisp.

## Semantics

- `(:wat::core::do)` — zero forms; evaluates to `()` (unit).
- `(:wat::core::do form1)` — single form; evaluates to form1's value.
- `(:wat::core::do form1 form2 ... formN)` — evaluates form1, discards its result, ..., evaluates formN, returns formN's value.

Type rule: each form except the last must have type `:wat::core::unit` (or `:wat::core::Vector<...>` / etc. — any return type whose value can be discarded). The last form's type IS the do-form's type. The substrate enforces unit-or-discardable on non-final forms — same rule that lets* applies to its non-final bindings via the `((_ :unit) ...)` shape.

## Implementation surface

Three options:

### Option A — Pure macro (preferred)

`(:wat::core::defmacro do (forms) -> :ast ...)` — expands to the
let*-with-unit-bindings shape we already have. ~10 LOC of wat in
`wat/core/...wat` (or wherever core macros live).

```scheme
(:wat::core::defmacro
  (:wat::core::do (forms :wat::core::Vector<wat::core::ast>) -> :wat::core::ast)
  ;; Expand (:do f1 f2 f3) → (:let* (((_ :unit) f1) ((_ :unit) f2)) f3)
  ;; Empty → ()
  ;; Single form → form
  ...)
```

Pros: no substrate change; fastest path to ship.
Cons: error messages still reference let*; expanded form looks like the crutch.

### Option B — Built-in special form

Substrate-level `eval_form_do` in `runtime.rs` + type-check arm in
`check.rs` (mirrors `eval_let_star`). Tighter type-checking; cleaner
diagnostics; but more code.

Pros: errors say "do form" not "let*"; no expansion noise in macroexpand.
Cons: ~150 LOC across substrate.

### Option C — A macro PLUS a substrate-known shape

The macro expands to `let*` BUT the type checker special-cases the
`((_ :unit) ...)` shape and reports errors as "do form". Sugar
without surgery. Probably more complex than B in practice.

**Recommendation: Option A first.** Ship the macro, sweep the
codebase. If diagnostic quality matters later, promote to B.

## Sweep scope

Once the form ships, every wat-tests file gets a sweep:

```
grep -rn '((_ :wat::core::unit)' wat-tests/ crates/*/wat-tests/ | wc -l
```

Estimated 100+ sites. Most replace cleanly:
```
(:wat::core::let*
  (((_ :wat::core::unit) FORM-1)
   ((_ :wat::core::unit) FORM-2))
  FORM-3)
```
becomes:
```
(:wat::core::do FORM-1 FORM-2 FORM-3)
```

Some sites are MIXED — `((_ :unit) ...)` interspersed with real
bindings. Those stay as let*. Phase-2 judgment.

## Slice plan

- **Slice 1** — mint the macro at `wat/core/...wat` (or wherever
  core macros live). Add ~5 unit tests covering empty / single /
  multiple / type-error cases.
- **Slice 2** — sweep. Replace pure `((_ :unit) ...)` chains with
  `(:wat::core::do ...)`. Mixed bindings stay let*. Workspace
  green throughout.
- **Slice 3** — closure. INSCRIPTION + USER-GUIDE row + WAT-
  CHEATSHEET note + memory pointer.

## Cross-references

- `.claude/skills/complectens/SKILL.md` — the spell whose calibration surfaced the need.
- `docs/arc/2026/05/135-complectens-cleanup-sweep/SCORE-SLICE-1.md` — the calibration record where the user named the crutch.
- arc 118 — the queued "lazy seqs vs threaded streams" arc (sibling pending; both are core-form additions).
- `wat/test.wat` — uses the let*-with-unit-bindings pattern extensively in deftest scaffolding; will benefit.

## When to start

After arc 135 closes (all 8 cleanup-queue files shipped + INSCRIPTION). The compositional rewrites flowing through arc 135 will end up using the let*-with-unit-bindings pattern; arc 136 sweeps that to the new form. Sequential — don't thrash both at once.

## Why this matters

The codebase advertises bad practices wherever the crutch shows. Wat is a Lisp; minting a clean `do` form is one line of new vocabulary that retires hundreds of lines of awkward let* boilerplate. The four questions answer YES at every test site after this arc.
