# Arc 155 — `:fn(...)` → `:wat::core::Fn(...)`; `:wat::core::lambda` → `:wat::core::fn`

**Status:** opened 2026-05-06 evening, after arc 154 closed.

## User direction (verbatim)

> *"alright... let's do another rename... (lambda ...) -> (fn ...).
> we're moving closer to clojure"*

> *"so... we need a type?... how about camel case?... Fn(:T)->:T"*

> *"so... let's swap. :fn(...) -> :Fn(...). then (:lambda ...) ->
> (:fn ...). new arc to handle both. we'll move define -> defn
> later .. we getting the pieces in place right now"*

> *"hold... /everything/ needs a namespace.. :wat::core::Fn to align
> /with everthing/ else"*

The user direction settled across four exchanges:
1. Open the lambda → fn rename
2. Recognize the keyword collision (operator position vs
   type position both want `:fn`)
3. Resolve via case-disambiguation (Cap'd type, lowercase verb)
4. Lock FQDN for full arc 109 consistency: `:wat::core::Fn(...)`
   not bare `:Fn(...)`

## Goal

Two coordinated renames bundled in one arc:

1. **Type position:** `:fn(...)` → `:wat::core::Fn(...)`. Closes
   arc 109's last ungrabbed parametric type head (slice 1e
   FQDN'd four-of-five; this closes the fifth). Aligns with
   the substrate's existing Cap+FQDN'd parametric types
   (`:wat::core::Vector<T>`, `:wat::core::HashMap<K,V>`,
   `:wat::core::Option<T>`, `:wat::core::Result<T,E>`,
   `:wat::core::Tuple<...>`, `:wat::core::HashSet<T>`).

2. **Operator position:** `(:wat::core::lambda ...)` →
   `(:wat::core::fn ...)`. Lowercase verb (Clojure-faithful
   single-letter form). Capitalization disambiguates from
   the type-position keyword.

The pair lands together. Capitalization carries the
position-vs-position semantic distinction:
- `:wat::core::Fn(...)` — function type (parametric, takes
  type args; lives at type position)
- `(:wat::core::fn (params...) -> :T body)` — function value
  (special form, takes binding+body; lives at operator
  position)

## Why bundle

Both renames touch the `fn` keyword family. Splitting into two
arcs would:
- Run two walkers concurrently (`BareLegacyLowercaseFn` for type
  position + `BareLegacyLambda` for operator position)
- Generate two atomic commits on closely-related changes
- Force users to learn two migrations instead of one

Bundling lets the substrate-as-teacher loop drive both transforms
simultaneously: every error in the diagnostic stream is one of
two known shapes; sweep 1b applies the appropriate 1:1 transform.
Same atomic-commit-across-coordinated-sweeps pattern as arc 153
+ arc 154.

## Empirical scope (verified pre-arc grep)

- **156 .wat lambda sites** (operator-position migration)
- **119 embedded Rust lambda sites** (`tests/*.rs` + `src/*.rs`
  inline wat strings)
- **125 .wat `:fn(...)` type sites** (type-position migration)
- **76 embedded Rust `:fn(...)` type sites**
- **Total: ~476 sites** for sweep 1b
- **0 existing `:wat::core::fn` operator usages** (clean slate)
- **0 existing `:wat::core::Fn(...)` type usages** (clean slate)

## The four questions

Run on the bundled rename 2026-05-06 evening:

1. **Obvious?** YES. Cap = type, lowercase = verb is conventional
   across most modern languages (Rust, Java, C#, Swift, Kotlin
   all use this pattern). FQDN form aligns with the substrate's
   existing parametric type vocabulary.
2. **Simple?** YES. Two parallel mechanical 1:1 transforms with
   substrate-as-teacher Pattern 3 walkers. Each transform is
   atomic per site; no semantic change at consumer surface.
3. **Honest?** YES. The capitalization-as-disambiguator is
   honest about what each spelling means; `Fn` is the type;
   `fn` is the verb; same root, different role at different
   syntactic positions.
4. **Good UX?** YES. One letform, two spellings, type system
   enforces the distinction. Clojure programmers see `(fn ...)`
   and read it correctly; Rust programmers see `Fn(...)` and
   read it correctly. Cross-language familiarity ships at both
   spellings.

## Substrate work

### Type-position rename (`:fn(...)` → `:wat::core::Fn(...)`)

Mirror arc 109 slice 1e's pattern (the four-of-five parametric
type heads FQDN'd: `Option`, `Result`, `HashMap`, `HashSet`).
Slice 1e was the precedent; arc 155 closes the fifth head.

- Parser/type-resolver recognizes `:wat::core::Fn(...)` as the
  canonical parametric type for function types
- Walker `walk_for_legacy_lowercase_fn` detects bare `:fn` at
  type position; emits `BareLegacyLowercaseFn` per site
- Bare `:fn(...)` retires (Pattern 3 substrate-as-teacher walker)

### Operator-position rename (`:wat::core::lambda` → `:wat::core::fn`)

Mirror arc 154's pattern exactly (the `:wat::core::let*` →
`:wat::core::let` rename). Same recipe.

- `infer_lambda` body moves under `:wat::core::fn` keyword
- `eval_lambda` body moves under `:wat::core::fn` keyword
- `:wat::core::lambda` dispatch arms retain functional fall-through
  per arc 113 precedent (silent alias)
- Walker `walk_for_legacy_lambda` detects `:wat::core::lambda`
  keyword; emits `BareLegacyLambda` per site
- Special-forms registry: `:wat::core::fn` minted; `:wat::core::lambda`
  retained as orphaned scaffolding entry

### Test file

NEW `tests/wat_arc155_fn_rename.rs` — 8-12 tests covering both
renames:
- Type-position canonical (`:wat::core::Fn(...)` works)
- Type-position retired (bare `:fn(...)` fires walker)
- Operator-position canonical (`(:wat::core::fn ...)` works)
- Operator-position retired (`(:wat::core::lambda ...)` fires walker)
- Mixed (one site uses both new forms cleanly)
- Walker narrowness (other type/keyword spellings unaffected)
- Reflection round-trip
- Tail-call sanity through new fn

## Slice plan

### Slice 1a — substrate (atomic with 1b)

Substrate work for both renames bundled. ~150-200 LOC across
`src/check.rs` + `src/runtime.rs` + `src/special_forms.rs` +
type-resolver edits + NEW `tests/wat_arc155_fn_rename.rs`.

`model: "sonnet"` explicit on Agent spawn per FM 12.

DO NOT COMMIT (atomic with 1b per recovery doc § 7).

### Slice 1b — consumer sweep

~476 sites mechanical 1:1 across two transforms. Substrate-as-
teacher walker-driven for both. Same sweep order as arc 154:
stdlib → crates/*/wat/ → wat-tests/ → crates/*/wat-tests/ →
examples/ → embedded wat in tests/*.rs + src/*.rs.

`model: "sonnet"` explicit on Agent spawn per FM 12.

Atomic commit when workspace = 0-failed.

### Slice 2 — retirement + paperwork

- Walker bodies retired (both `walk_for_legacy_lowercase_fn` and
  `walk_for_legacy_lambda`); CheckError variants + Display retained
  as orphaned scaffolding per arc 113 precedent
- Operator-position dispatch arms for `:wat::core::lambda` retained
  as transitional runtime scaffolding (silent alias to `:wat::core::fn`;
  mirrors arc 154's let* dispatch retention)
- Type-position retirement decision: same orphaned-scaffolding
  pattern (parser still recognizes bare `:fn` but produces NO
  diagnostic post-walker-retirement; OR fully retires and bare
  `:fn` becomes "unknown type" — sonnet's call per honest delta)
- INSCRIPTION orchestrator-side per
  `feedback_paperwork_orchestrator_side.md`
- 058 row, USER-GUIDE update, WAT-CHEATSHEET update
- Pre-INSCRIPTION grep mandatory per FM 11
- Task list update: arc 109 slice 1e completion note + this arc
  cross-reference

## Cross-references

- **Arc 109 slice 1e** — closest precedent for type-position
  FQDN'ing of parametric type heads. Arc 155 closes the fifth
  head (lambda's `fn`).
- **Arc 154** — closest precedent for operator-position keyword
  rename (let* → let). Arc 155 mirrors the recipe at the
  operator-position layer.
- **Arc 153** — Pattern 3 walker recipe for symbol migration
  (BareLegacyUnitName). Arc 155 ships two walkers in parallel.
- **Arc 145** — typed-let detour backout; the substrate-already-
  typed insight. Arc 155's renames are pure vocabulary
  cleanup; no typed-form discipline involved.
- **`feedback_paperwork_orchestrator_side.md`** — closure
  paperwork orchestrator-side discipline (arc 153 protocol break
  + correction).
- **`feedback_agent_model_explicit.md` + recovery doc § FM 12**
  — every Agent spawn must include `model: "sonnet"` explicitly.
- **Future arc** — `:wat::core::define` → `:wat::core::defn`
  rename (per user's "we'll move define -> defn later"). Arc 155
  does NOT touch define; that's a separate arc when the user
  surfaces it. Out of scope; not tracked elsewhere.

## When to start

Now. Pattern is well-trodden today (arcs 153 + 154 used the same
recipe). Slice 1a substrate brief drafts next; spawn sonnet
(model: "sonnet" per FM 12); sweep 1b after; closure orchestrator-
side.

## Estimated effort

- Slice 1a substrate: ~30-50 min sonnet wall-clock (two walkers +
  two semantic switches; ~1.5x arc 154's slice 1a profile)
- Slice 1b consumer sweep: ~15-25 min sonnet wall-clock (~476
  sites; mechanical; smaller than arc 154's ~806)
- Slice 2 retirement + paperwork: ~30 min orchestrator
- Total: ~1.5 hours wall-clock if Mode A clean throughout

## Why this matters

> *"we're moving closer to clojure"*

Three foundation marks landed today: `nil`, `do`, `let`
sequential. Arc 155 lands the fourth — single-letform `fn` for
function values + Cap'd `Fn` for function types. The Lisp on Rust
keeps consolidating its user-facing surface toward Clojure-
familiar idiom while honoring Rust's type-name convention.

Plus: arc 109's last ungrabbed parametric type head closes,
advancing arc 109 v1 closure trajectory by another link.
