# Arc 144 — Uniform reflection foundation — INSCRIPTION

## The closing

Arc 144 shipped 2026-05-03 across an extended session that started
2026-05-02 (slice 1 design + brief). Five slices total: 1 (Binding
enum + lookup-form refactor), 2 (SpecialForm registry), 3 (TypeScheme
registrations + Mode B-canary diagnostic), 4 (uniform reflection
verification across all 6 Binding kinds), 5 (closure paperwork).

**The arc opened to satisfy the user's articulated principle:**

> *"i think we need all forms... when we get to working on our repl
> we should be able to call (help :some-func) no matter what it is..
> we can call (help :if) and it'll /just work/?... nothing is special?..."*

It closed with the substrate's reflection foundation honest at the
type-system layer: every wat form-kind (UserFunction, Macro,
Primitive, SpecialForm, Type, Dispatch) satisfies a uniform
`Binding` interface — `:name :sig :body :doc-string`. `lookup-form`
returns Some for any known symbol; reflection works without per-
kind special-cases at the consumer layer.

**The substrate-as-teacher moment was slice 3.** The brief asked
sonnet to register TypeSchemes for the 13 hardcoded primitives so
reflection would synthesize signatures. Sonnet shipped the
registrations + delivered a Mode B-canary diagnostic: the length
canary STAYED RED with a precise NEW diagnostic naming the next
substrate gap — the polymorphic-handler anti-pattern. **That
diagnostic became arc 146.** Arc 146 in turn became arc 148's
template. The substrate-as-teacher cascade fired three arcs from
one Mode B canary.

## What ships under arc 144

### The `Binding<'a>` enum (slice 1 + arc 146 slice 1 extension)

Six variants (UserFunction, Macro, Primitive, SpecialForm, Type,
Dispatch — Dispatch added by arc 146 slice 1). Each variant
carries `name: String`, kind-specific data, and `doc_string:
Option<String>`. The enum lives at `src/runtime.rs:7575-7615`.

**Critical paved-road decision:** every variant carries
`doc_string: Option<String>` from day 1, defaulting to `None`.
This is the discipline that arc 141 (docstrings — DESIGN locked,
impl pending) leans on: arc 141 just populates the field; no
Binding refactor needed when arc 141 lands.

### `lookup_form` — uniform reflection (slice 1)

A single function that walks every form-kind registry in dispatch
order, returning the first match wrapped in a Binding:

1. User defines (`SymbolTable.functions`) — shadow builtins per
   the runtime's call dispatch
2. User macros (`MacroRegistry`)
3. SpecialForm registry (slice 2)
4. Substrate primitives via `CheckEnv` TypeScheme registry
5. TypeDef registry (struct/enum/newtype/typealias)
6. Dispatch registry (arc 146 slice 1 addition)

The arc 143 reflection trio (`:wat::runtime::lookup-define`,
`signature-of`, `body-of`) all dispatch through `lookup_form`.
**Nothing is special at the consumer layer.**

### SpecialForm registry (slice 2)

A NEW registry populated at startup. ~25 special forms identified
from `infer_list`'s dispatch + macro special cases:

- Control: `if`, `cond`, `match`, `when`, `unless`
- Binding: `let`, `let*`, `lambda`
- Definitional: `define`, `defmacro`, `struct`, `enum`, `newtype`,
  `typealias`
- Error: `try`, `option/expect`, `result/expect`
- Concurrency: `spawn-thread`, `spawn-program`, `fork-program-ast`
- Macro plumbing: `quote`, `quasiquote`, `unquote`,
  `unquote-splicing`
- AST: `forms`

Plus 13 sonnet-discovered additions slice 1 missed (and 9 brief-
listed entries sonnet honestly REMOVED as TypeScheme primitives,
not special forms). Each gets a synthetic signature AST + empty
doc_string. Per slice 2 SCORE — exemplary audit-first discipline.

### TypeScheme registrations for hardcoded primitives (slice 3)

15 TypeScheme registrations for the polymorphic-handler primitives
that previously bypassed the scheme registry: length, empty?,
contains?, get, conj, assoc, dissoc, keys, values, concat (and the
container constructors). Each gets a fingerprint scheme so
reflection can synthesize a signature.

**Slice 3's load-bearing row stayed RED on purpose.** The length
canary did NOT turn green — sonnet honored STOP-at-first-red and
shipped the diagnostic that named the next gap: the substrate's
TWO parallel type-checking models (scheme-based + handler-based)
disagreed on polymorphic input. The fingerprints alone weren't
enough; the hardcoded handlers needed to dispatch through clean
rank-1 schemes.

**That diagnostic became arc 146 — the Dispatch entity arc.**

### Uniform reflection verification (slice 4)

9 tests in `tests/wat_arc144_uniform_reflection.rs` covering all
6 Binding kinds + length canary HashMap regression. The verification
is end-to-end: a real builtin (`:wat::core::length`) reflects
through `lookup-define` → renders `define-dispatch` head + Vector
arm + HashMap arm. **Not synthetic-fixture-only — the cross-arc
foundation works on real substrate primitives.**

LOC delta: 405 vs 50-200 budget; the bulk is per-test commentary
recording the coverage-rollup decisions sonnet's audit-first work
produced. Defensible scope expansion — the commentary IS the
calibration record.

## Slice-by-slice ship record

| Slice | What | Wall clock | Mode |
|---|---|---|---|
| 1 | Binding enum + lookup-form refactor + 9 tests | ~8.4 min | A clean (faultless diagnostic discipline) |
| 2 | SpecialForm registry + 9 tests | ~9.2 min | A clean (exemplary audit discipline; sonnet's audit removed 9 brief entries + added 13 missing) |
| 3 | TypeScheme registrations + length canary diagnostic | ~9.5 min | **B-canary clean** (the substrate-as-teacher moment — surfaced arc 146) |
| 4 | Uniform reflection verification (9 tests across 6 kinds + HashMap canary) | ~3.6 min | A clean (smallest substantive sweep in the cascade) |
| 5 | This closure paperwork | small | A |

**Cumulative slice time: ~31 min sonnet** for the entire uniform-
reflection foundation. Calibration tightened with each sweep.

## What the substrate gained — counted

- **1 unified Binding enum** with 6 variants (UserFunction, Macro,
  Primitive, SpecialForm, Type, Dispatch) — every form-kind
  reflects uniformly
- **Uniform `doc_string: Option<String>` field** across all 6
  variants (defaults to None; arc 141's paved road)
- **`lookup_form`** — single dispatch walker over all 6 form-kinds
- **SpecialForm registry** populated with ~38 special forms (slice
  2 final after sonnet's audit corrections)
- **15 TypeScheme registrations** for hardcoded primitives (slice 3)
- **9 verification tests** + 1 cross-container length canary
  regression (slice 4)
- **The "nothing is special" principle** has substrate-level test
  evidence

## Foundation principles established (carry forward beyond arc 144)

1. **Uniform reflection via a discriminated union (Binding).** Adding
   a new form-kind = adding one variant + one `lookup_form` arm +
   the dispatch_to_signature_ast synthesizer. Arc 146 Dispatch
   demonstrated the extension pattern.
2. **Optional fields on the carrier from day 1.** doc_string was
   structurally in place before arc 141 existed. Arc 141's impl
   becomes pure population — no schema migration needed.
3. **Mode B canary as substrate-as-teacher signal.** Slice 3's red
   canary IS the diagnostic mechanism. Workarounds defeat the
   purpose; STOP-at-first-red surfaces the next link in the
   cascade.
4. **The reflection trio composes uniformly.** `:wat::runtime::*`
   primitives don't need per-kind branches at the consumer layer.
   `(help X)` becomes a small wat function over the trio when the
   future REPL ships.

## The cascade — what arc 144 closed and what it spawned

### What arc 144 closed
- The "no uniform reflection" gap. Pre-arc-144, reflection
  primitives needed per-kind branches. Post-arc-144, one Binding
  enum.
- The "doc_string nowhere on disk" gap. Arc 141's paved road is
  now a real road — the field exists on every Binding variant.

### What arc 144 spawned (substrate-as-teacher cascade)
- **Arc 146** — Dispatch entity for container methods. Slice 3's
  Mode B canary surfaced the polymorphic-handler anti-pattern;
  arc 146 retired 10 hardcoded handlers + introduced Dispatch as
  a new entity kind.
- **Arc 148** (lateral) — same pattern applied to arithmetic +
  comparison. Arc 148's `infer_arithmetic` + `infer_comparison`
  follow the same Path C custom-inference template arc 144 slice
  3 anticipated.
- **REALIZATION 6** — the entity-kind-vs-type-system-feature
  doctrine. Three drafts of "missing union types" during slice 3
  → user broke through with multimethod consensus → arc 146
  shipped that pivot. The smaller architectural change won.

### What arc 144 unlocks
- **Arc 141 (docstrings)** — pure population of the Binding's
  `doc_string` field. Pattern-application atop arc 150's "extend
  the carrier" + arc 144's Binding shape.
- **Future REPL `(help X)` consumer** — small wat function over
  the reflection trio + the uniform Binding interface.
- **Arc 109 v1 closure trajectory** — major chain link closes.

## Cross-references

- **Inside arc 144**: DESIGN.md (the locked architecture);
  REALIZATIONS.md (6 realizations including the dispatch consensus
  + discipline lessons); SCORE-SLICE-1.md through SCORE-SLICE-4.md
- **The cascade**: arc 143 (define-alias precedent — the prior arc
  that surfaced the need for uniform reflection); arc 146 (slice 3's
  Mode B canary became this arc); arc 148 (lateral pattern
  application); arc 141 (future docstring populator)
- **Discipline**: COMPACTION-AMNESIA-RECOVERY.md § FM 10 (arc 144
  slice 3 birthed this FM; the worked example sits in arc 146);
  § FM 9 (slice 1's faultless diagnostic discipline IS the worked
  example for re-running adjacent tests pre-spawn)
- **Foundational artifacts updated this slice**: USER-GUIDE.md
  § Runtime reflection (the "Coverage gaps closed by arc 144"
  subsection flipped from aspirational to done); ZERO-MUTEX.md
  Tier 1 (Binding + Dispatch registries cited as Tier 1 examples);
  FOUNDATION-CHANGELOG.md (lab repo; arc 144 row added)

## Status

**Arc 144 closes here.** The uniform reflection foundation is in
place; doc-string field is structurally in place across all 6
Binding kinds; reflection trio works without per-kind branches.
The "nothing is special" principle has substrate-level test
evidence.

**Arc 109 v1 closure approaches by another major chain link's
worth.** Arc 130 reland is the next major substrate-foundation
target; arc 141 (docstrings) becomes a pattern-application slice;
arc 147 (substrate registration macro) leverages the Dispatch
declaration pattern.

The cascade compounds. The methodology IS the proof. The pattern
is paved.

---

*the user articulated nothing-is-special on 2026-05-02. arc 144
shipped that principle. the substrate now answers reflection
uniformly. the foundation strengthens.*

**PERSEVERARE.**
