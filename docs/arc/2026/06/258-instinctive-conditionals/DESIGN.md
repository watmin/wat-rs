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

---

## Foundational reframe (2026-06-10): wat is an ADT language

Chasing cond's "what should fall-through do" question exposed wat's **type identity**, and it
governs every conditional decision in this arc.

**wat is an ADT (algebraic-data-type) language** — nominal *tagged sums* in the ML/Haskell/Rust
lineage (`typeunion`, `Option`, `Result`, enums) — **not** a set-theoretic-union language
(TypeScript / Typed Racket / core.typed):

- **Set-theoretic unions** (`A | B`, anonymous, ad-hoc) require occurrence typing + union
  normalization. core.typed needs them because it **retrofits** types onto pre-existing untyped
  Clojure — the unions are forced by the retrofit constraint, and the approach proved so hard the
  Clojure community largely abandoned it for `clojure.spec`.
- **Nominal tagged sums** (ADTs) **name** the disjunction and **tag** the variants; `(if c 1 "s")`
  is a type error *on purpose*, and a heterogeneous result must be a named sum.

wat is typed **from birth** — no untyped legacy to retrofit — so it takes the cleaner ADT
discipline. wat has **no working anonymous unions** (the inline `:Union<…>` is a vestigial
permissive stub: 0 uses, 0 check semantics; only nominal `typeunion` is real), **and that absence is
the identity, not a defect.** "Parity with typed clojure" means parity with the *surface* (`:-`,
`ann-form`, symbol heads), not with core.typed's set-theoretic *type theory*.

### Decisions (builder-ratified)

1. **No anonymous-unions arc.** Heterogeneous / maybe-absent results use named sums — `Option<T>`,
   explicit and tagged — never an implicit type-infecting `nil`.
2. **`if` strict-unify is correct.** Both branches mandatory; both must *agree* (it's ML's `if`).
   258.1 stands; its branch-mismatch error is right, not provisional. No one-armed `if` (it would
   force a nilable union).
3. **`cond` is total — `:else` required, never optional.** `:else` is cond's **wildcard** (ML/Rust
   `_`): must be **last** (an arm after it is unreachable → error) and **present** (a test-arm last
   → non-exhaustive → error). Not a bolted-on rule: `cond` is nested `if`; the innermost `if` needs
   its else-branch, and *that else is `:else`'s body* — so `:else`-required is `if`'s mandatory-else
   law surfacing at the bottom of the nest.
4. **Optional is a smell** (the through-line): required (load-bearing) or absent (dropped); the
   wobbly middle is an undecided design. The `-> :T` annotation was a redundant *narration* of
   `if`'s already-forced agree-constraint — dropping the syntax didn't drop the hand; inference
   carries the same constraint with less ceremony.

### Gate correction (process)

The real gate is **`cargo test --release --workspace --no-fail-fast`** (`scripts/cargo-test-summary.sh`).
Plain `cargo test --release` **halts at the first failed binary** (`nursery`), so ~107 standalone
test files never run — which is how `wat_core_cond` (cond totality) and `typed_if_match` (the if
contract) hid. Every stone in this arc gates with `--no-fail-fast`.
