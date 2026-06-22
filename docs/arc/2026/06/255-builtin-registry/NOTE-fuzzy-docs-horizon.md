# NOTE — HORIZON: fuzzy doc-search over the forced-true intrinsic corpus (later, not a priority arc)

> Surfaced 2026-06-22 (builder). NOT a build target now — captured so the thread isn't lost.

## The thread
The forced intrinsic-doc contract (this arc) produces something no normal doc corpus is: a
**structurally-uniform, guaranteed-true** body of docs (the build won't allow an incomplete or
false one — name/count, type, behavior, and purity each held by an independent witness). That is
the ideal substrate for **fuzzy / semantic doc-search via holon's VSA/HDC**.

## The prior art (grounded — the builder's early prototype)
**Challenge 003-batch — "high-performance fuzzy quote finder"** (~2026-01-27..30):
- spec `docs/challenges/003-batch/001-problem.md`; learnings `…/LEARNINGS.md`; solutions
  `scripts/challenges/003-batch/001-solution{,-http,-enhanced}.py`; test data
  `calculus-made-easy.pdf` (PyPDF2 → 2,897 units).
- Technique: words → near-orthogonal HVs; **n-gram binding** (bigrams ≈ 75% F1) for position-free
  fuzzy subsequence match; each unit = `words ⊙ chapter ⊙ paragraph ⊙ page` (structure bound INTO
  the vector); **metadata-only storage** (pointers, no text); **vector bootstrapping** via `/encode`
  (query vector without storing → O(1) repeat search); similarity + **guards**; advanced primitives
  (`prototype`/`blend`/`amplify`/`negate`) for "X and Y", "X emphasized", "X but not Y".

## Why it's far better now (the connection — apparatus's read)
003 ran that over a **noisy unstructured PDF** and still hit 75% F1. The wat doc-contract gives the
**opposite** substrate — uniform roles per unit (name, arg-types, ret-type, prose, examples, see),
guaranteed true. So you bind **structured roles** (`arg-type ⊙ ret-type ⊙ prose-ngrams ⊙ example-forms`)
instead of raw n-grams over book prose, and fuzzy-query the *shape* of an intrinsic: "takes Bytes,
returns String", "like to-hex", "the inverse of from-hex". 003's metadata-only + role-binding maps
directly onto the registry — it was practically waiting for this corpus.

## The horizon (builder's vision — not flattened into mine)
A **fuzzy doc-search MCP** over the forced-true corpus → semantic intrinsic discovery that **cannot
return a stale or false hit** (the corpus can't lie). "Docs that can't lie" = "a fuzzy-search corpus
that can't lie" → an MCP trick for rapid development. The builder sees further here; the shape is his
to name when he lands it. Prereq: the firm-doc-contract stone (this arc) ships the corpus it needs.
