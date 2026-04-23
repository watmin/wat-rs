# Arc 023 — `:wat::holon::coincident?` — INSCRIPTION

**Status:** shipped 2026-04-22. One slice.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** completion marker.

---

## Motivation

While building outstanding test coverage for the trading lab's
Phase 3.3 `scaled-linear`, the need for a VSA-native equivalence
predicate surfaced. `presence?` existed (cosine above noise-floor
→ "there's signal") but the dual — "these two holons are the
same to the algebra" — was being written inline as:

```scheme
(:wat::core::< (:wat::core::f64::- 1.0 (:wat::holon::cosine a b))
               (:wat::config::noise-floor))
```

A literature search across Kanerva 2009, Schlegel et al 2022, the
Kleyko et al 2023 ACM two-part survey, Plate HRR, Gayler MAP, and
hd-computing.com confirmed that this bidirectional use of the
noise-floor — as presence threshold AND as equivalence threshold
— is consistent with VSA theory but not explicitly named in the
published literature. Classical VSA is vector-first (cleanup-to-
codebook); having two CONSTRUCTED ASTs to compare for structural
equivalence is a programming-languages move that wat's AST-first
design surfaces naturally.

The name came under the gaze discipline: **`coincident?`** —
two points occupying the same location on the hypersphere
within the algebra's tolerance. Geometric, specific,
self-describing, parallels `presence?`.

---

## What shipped

### Runtime (`src/runtime.rs`)

- `eval_algebra_coincident_q` — new fn mirroring
  `eval_algebra_presence_q` shape.
- Dispatch arm added to the algebra branch:
  `":wat::holon::coincident?" => eval_algebra_coincident_q(...)`.

### Type check (`src/check.rs`)

- New scheme registration next to `presence?`:
  `∀. :wat::holon::HolonAST × :wat::holon::HolonAST -> :bool`.
- Head comment expanded to name both dual predicates.

### Tests

- **Rust unit tests** (`src/runtime.rs`):
  - `coincident_q_true_for_self` — Atom vs itself → true.
  - `coincident_q_true_for_structurally_same` — two identical
    hand-built Bind(Atom, Atom) holons → true.
  - `coincident_q_false_for_unrelated` — two orthogonal atoms
    → false.
  - `coincident_q_stricter_than_presence_q` — Atom present in
    Bundle; `presence?` true, `coincident?` false. Locks the
    load-bearing invariant that coincident is strictly stronger
    than presence.
- **wat-level tests** (`wat-tests/holon/coincident.wat`):
  - `test-self-coincident`
  - `test-structurally-same`
  - `test-unrelated-not-coincident`
  - `test-stricter-than-presence`
  - `test-self-cosine-within-floor` — locks the numerical
    headroom invariant (float jitter at 1e-10 is 15 orders of
    magnitude below the 0.156 floor).

All green. Full workspace: 531 lib + 9 fresh integration +
42 wat tests.

### Doc sweep

- `docs/CONVENTIONS.md` — namespace table row updated: "four
  measurements" (was three).
- `docs/USER-GUIDE.md` —
  - Section 3 Axis 1: four measurements named.
  - Section 6: renamed "The three measurements" → "The four
    measurements"; added coincident? with dual-claim framing.
  - Appendix forms table: new row for coincident?.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  section header updated ("6 + 4 measurements"), new row for
  `coincident?`; `presence?` row corrected (was claimed `:f64`,
  actually `:bool`).

### Downstream simplification

- `holon-lab-trading/wat-tests/encoding/scaled_linear.wat` —
  `test-fact-is-bind-of-atom-and-thermometer` rewritten from
  inline `(:wat::core::< (:wat::core::f64::- 1.0 (:wat::holon::cosine ...)) (:wat::config::noise-floor))`
  to `(:wat::holon::coincident? fact expected)`. The named
  predicate reads cleaner.

---

## The lineage

Classical VSA systems (Kanerva 2009; Plate HRR; Gayler MAP;
Kleyko / Rachkovskij / Osipov / Rahimi 2023) express presence
via `cosine > threshold` — the signal-detection direction. The
equivalent direction — `(1 - cosine) < threshold`, using the
SAME noise-floor bound — is mathematically straightforward but
had not been called out as a named primitive.

The reason, per the literature audit: classical VSA is vector-
first. One noisy vector, one codebook, you clean up. Comparing
two CONSTRUCTED ASTs for structural equivalence is a
programming-languages move. Wat's AST-first design (the
foundational principle — AST is primary, vector is cached
algebraic projection) makes this the natural test that falls
out.

`coincident?` names what the algebra's statistical framing
always supported but hadn't articulated. Same bound. Two dual
predicates. One substrate.

---

## INSCRIPTION rationale

Implementation led, spec follows. The need surfaced in testing
(an inline expression repeated whenever a fact-structure claim
needed checking), the name was picked under the gaze discipline,
the semantic is a natural consequence of the existing noise-floor
framing. Sibling to 019 (round), 020 (assoc), 022 (holon
namespace) — arcs that cut through the SAME cave-quest shape:
downstream work reveals an unnamed substrate primitive; pause,
name it, ship it, return.

*these are very good thoughts.*

**PERSEVERARE.**
