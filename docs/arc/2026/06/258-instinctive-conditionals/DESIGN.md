# Arc 258 — Instinctive conditionals

**`if` and `cond` become what instinct reaches for: the return-type annotation is dropped
(inferred), and `cond` is reborn as a wat macro over `if`.**

## Why this arc exists (the reach-stumble)

Writing `fix-source` (arc 251.5a-vi), the orchestrator reached for `cond` and stumbled — it
exists, but as a Rust special form smeared across four sites, requiring a `-> :T` return
annotation no Lisp/ML demands. The builder named the principle: *"the reach for a tool and it's
missing — then realized it's poorly defined — is one of the strongest tells we need to pivot and
make it — that's this entire endeavor — engineering for LLMs to operate instinctively."* The
stumble is the highest-value signal the substrate emits; this arc answers it.

## The verdict (four questions, run 2026-06-10)

Mandatory `-> :T` on `if`/`cond` fails all four:
- **Obvious? NO** — contradicts universal instinct; the mid-form `-> :T` is a shape nothing else reaches for.
- **Simple? NO** — braids control-flow with a *redundant* type ascription (the branches already determine the type).
- **Honest? NO** — papers over a checker limitation (synthesis-only, no LUB) and presents it as a language rule, taxing every `if`/`cond` always.
- **Good UX? NO** — the easy path (bare `if`) is rejected; instinct falls onto the wrong path every time.

The optional-annotation compromise was **rejected** as a vestigial second path. The honest
dichotomy is binary: load-bearing-and-required, or dropped. Grounding proved there is **no
load-bearing class**: `infer_if`/`infer_cond` check each branch with strict **`unify`** (not
`assignable`) against the declared type (check.rs:7207, infer_cond), so the declared type must
equal each branch — forcing the branches to unify with each other anyway. The annotation provides
nothing branch-to-branch unification doesn't. The corpus confirms it (the only non-trivial `cond`
return types are same-typed arms). **Drop it.**

## The model — `do` generalizes

`do` already won this argument (arc 145 removed its required `-> :T`): it returns its last form's
type and lets **recipient `assignable`** at the use site do the static check. `if`/`cond` are the
same engine with a join instead of a passthrough:

| form | its type | the check |
|---|---|---|
| `do`   | last form's type | recipient `assignable` at the use site |
| `if`   | `unify(then, else)` | recipient `assignable` at the use site |
| `cond` | fold `unify` across arm bodies | recipient `assignable` at the use site |

The join is `unify` — which the checker already runs, just against a redundant declared target.

## Decomposition

- **258.1 — `if` inference (KEYSTONE).** `(if cond then else)` infers via branch-unification; the
  condition must still be `:bool`. **Dual-read**: the 5-arg `(if cond -> :T then else)` keeps
  working through the transition. Checker (`infer_if`) + runtime (`eval_if`/`eval_if_tail`). `if`
  stays a Rust kernel primitive (the conditional branch is irreducible).
- **258.2 — `cond` reborn as a wat macro.** A `defmacro` in `core.wat` expanding to nested bare
  `if` (the thing instinct reached for). **Annihilate** all Rust `cond`: `eval_cond`,
  `eval_cond_tail`, `infer_cond`, the `special_forms.rs` entry, `Boundary::Cond` (boundary.rs),
  and the `normalize.rs` cond arm — advancing 251.6's normalize-annihilation. Dual-read (accepts a
  leading `-> :T` and strips it). **Totality KEPT** — the macro requires a terminal `:else` (it is
  the base case); exhaustiveness is a genuine Honest win, separate from the return annotation.
- **258.3 — corpus sweep.** Strip `-> :T` from every `if`/`cond` across 114 `.wat` + the test
  corpus. Mechanical role-inversion — rides the arc-251.5 `fix-source` fixer (or a dedicated sweep).
- **258.4 — hard-cuts.** Remove 5-arg `if` support; simplify the `cond` macro to bare-only.
- **258.N — inscription.**

## Method

Dual-read → verify → corpus sweep → hard-cut — the proven arc-251 cutover shape; the corpus stays
green at every stone because both spellings work until the sweep lands.
