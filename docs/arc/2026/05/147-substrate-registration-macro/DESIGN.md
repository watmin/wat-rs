# Arc 147 — Substrate registration macro (single-source-of-truth for primitives)

**Status:** drafted 2026-05-03 mid-arc-146-slice-2 (sonnet
in flight). User-direction following four-questions analysis on
"how do we know with full certainty that a primitive is defined in
all the correct places."

## The half-completion class this arc closes

Today, registering a substrate primitive `:wat::core::X` requires
TWO independent edits in two different files:

1. **Check side** — `env.register(name.into(), TypeScheme {...})`
   in `check.rs::register_builtins` (around line 11640+).
2. **Runtime side** — `match` arm in `eval_list_call` dispatch +
   `fn eval_x(...)` impl in `runtime.rs`.

Nothing structurally enforces these stay in sync. We rely on
convention. We've already paid the cost:

- Arc 144 slice 3 registered a `:wat::core::length` TypeScheme
  fingerprint that contradicted the hardcoded handler — slice 3
  surfaced the disagreement as a Mode B-canary diagnostic.
- Arc 143 slice 5b changed `extract-arg-names` runtime emission;
  the type-checker scheme didn't update; tests broke until
  orchestrator caught it manually.
- A future contributor could register a TypeScheme without a
  runtime arm (or vice versa); the substrate boots fine; the
  drift surfaces only when the primitive is actually called.

User requirement (verbatim): *"i must know WHEN anyone adds half
completion."*

Detection at runtime is too late. Detection at PR-time via test
enumeration is a band-aid that itself gets forgotten. The honest
answer is structural: **make half-completion impossible at
compile time.**

## The four questions that revealed this arc

Run on three options 2026-05-03 (recorded so future-readers
inherit the reasoning):

| Option | Obvious | Simple | Honest | UX |
|---|---|---|---|---|
| Test enumeration | Y | Y | **N** — only catches what test knows; new primitives forgotten by test ARE half-complete and undetected | n/a |
| Runtime registry + startup invariant | Y | Y | Y — startup refuses to boot if drifted; CI catches at PR time | Adequate — two registration sites; system catches mistakes after they happen |
| **Single macro emits both** | Y | Y | Y — STRUCTURALLY impossible to half-complete; only one place to register | **Best** — one site; compile-time enforcement; can't accidentally half-complete |

Test enumeration FAILS honest (band-aid). Runtime registry +
macro both pass. Tiebreaker on "must know WHEN": macro catches
at COMPILE TIME (`cargo build` fails); registry catches at
STARTUP (CI surfaces). Macro is earlier + structurally prevents
existence.

**Macro wins. This arc.**

## Architecture

A single registration form binds:
- The primitive's keyword path
- Its TypeScheme (type_params, params, return)
- Its runtime impl

ONE source of truth; the substrate emits both sides from it.

### Mechanism options (slice 1 decides)

Three plausible mechanisms. All achieve "single source"; differ
on Rust-implementation flavor.

**A — Attribute proc macro on the impl function.**

```rust
#[wat_primitive(
    name = ":wat::core::foldl",
    scheme = "forall T,Acc. Vec<T> Acc fn(Acc, T) -> Acc -> Acc",
)]
fn eval_vec_foldl(args: &[WatAST], env: &Environment, sym: &SymbolTable)
    -> Result<Value, RuntimeError>
{
    // ... existing impl ...
}
```

The attribute macro records the registration into a distributed
slice (via `inventory` or `linkme` crate); `register_builtins`
and the runtime dispatcher walk the slice. The impl function is
ALSO the runtime arm — no manual `match` arm needed.

**B — Declarative macro at registration site.**

```rust
register_primitive! {
    name: ":wat::core::foldl",
    scheme: forall T, Acc. Vec<T> Acc fn(Acc, T) -> Acc -> Acc,
    impl: eval_vec_foldl,  // the function lives elsewhere
}
```

Less elegant (two sites — the registration AND the impl). Doesn't
fully prevent drift between the macro's `impl: name` and the
function's actual signature. Rejected.

**C — Build-time codegen.**

A `build.rs` walks source; emits both registration sides. Most
involved; brittle to Rust syntax changes. Rejected for arc 147;
could be retrofit later.

**Recommendation: A.** Single attribute on the impl function;
the function declaration IS the registration. Drift is
impossible because there's nothing TO drift — the macro emits
both sides from the same source.

Slice 1 brief verifies feasibility (existing `wat-macros` crate
at `crates/wat-macros/` is the home; `inventory` or `linkme`
crate handles distributed registration). If A is infeasible
(e.g., `inventory` doesn't fit the substrate's threading model),
fall back to B with explicit cross-site arity check.

### Scope: substrate primitives only

This arc addresses ONE drift class: primitives with both check
+ runtime registration.

Other entity kinds have their OWN registries today:
- **Macros** — `MacroRegistry` (single source via defmacro)
- **Special forms** — arc 144 slice 2's `special_forms.rs`
  registry (single source via insert)
- **Types** — `TypeEnv` (single source via struct/enum/etc.)
- **Dispatches** — arc 146 slice 1's `dispatch_registry`
  (single source via define-dispatch)

These DON'T have the drift class because each lives in one
registry. Only primitives have the two-sided shape that breeds
drift. Arc 147's scope is bounded to primitives.

## Slice plan

### Slice 1 — Define the macro + migrate ONE primitive (proof of mechanism)

- Add the attribute proc macro to `crates/wat-macros/`
- Add the distributed-slice infrastructure (`inventory`/`linkme`)
- Refactor `register_builtins` to walk the slice instead of
  hand-typing entries
- Refactor the runtime dispatcher to walk the slice instead of
  hand-typing match arms
- Migrate ONE primitive (recommended: `:wat::core::foldl` — well-
  understood, scheme-shaped, has a clean rank-1 type)
- Test: assert the migrated primitive still works end-to-end
- Test: attempt to declare a primitive with a SIGNATURE that
  doesn't match the function's runtime signature → compile error
- All other primitives still register via the OLD path (coexist)

~400-700 LOC; substantial slice.

### Slices 2-N — Migrate existing primitives in batches

Group by namespace or domain (`:wat::core::*` arithmetic,
`:wat::core::*` strings, `:wat::holon::*` algebra, etc.). Each
slice migrates one batch. ~10-30 primitives per slice depending
on shape. Each migration is mechanical once the macro is proven.

After all primitives migrate: every substrate primitive's check
+ runtime are bound at the source. Half-completion class
disappears.

### Slice N+1 — Lint enforcement

Add a lint (clippy custom or pre-commit) that forbids manual
`env.register(...)` of substrate primitive names outside the
macro. Catches future bypass attempts.

### Slice N+2 — Closure

INSCRIPTION + 058 row + USER-GUIDE entry + ZERO-MUTEX cross-ref
(if the distributed-slice crate uses unsafe / atomic patterns
worth documenting).

## Open questions

### Q1 — Distributed slice crate

`inventory` vs `linkme` vs hand-rolled with `OnceLock + Vec`.
Slice 1 brief surveys + picks. Constraints: must work without
unsafe in user code; must support no_std if substrate ever needs
it; must be CI-friendly (no link-time magic that breaks under
certain build configurations).

### Q2 — Schemes that aren't expressible as a string

The DESIGN sketch shows scheme as a string literal. Some schemes
involve complex shapes (HashMap fields, Result variants, recursive
types) that may not parse cleanly from a string.

Options:
- Build a TypeScheme parser (substrate already has wat parsing —
  could reuse?)
- Allow scheme as a Rust expression: `scheme: TypeScheme { ... }`
- Hybrid: simple schemes as strings; complex as expressions

Decide in slice 1 brief.

### Q3 — Hardcoded `infer_*` handlers

A few primitives still have hardcoded handlers (the ones arc 146
hasn't migrated yet — empty?, contains?, get, conj, etc.). These
don't fit the macro shape today.

After arc 146 closes, these are gone (replaced by dispatch). Arc
147 doesn't need to handle them as a separate case; it just
migrates the post-arc-146 set of clean rank-1 primitives.

Sequencing: arc 147 lands AFTER arc 146 closes. Slices 2-N can
then assume all substrate primitives are clean rank-1.

### Q4 — Existing arc 144 reflection (`lookup_form`)

Arc 144's `Binding::Primitive` carries a `TypeScheme` reference.
Post-arc-147, this binding's data comes from the same
distributed slice. Verify reflection still works.

## Sequencing relative to arc 146

Per § 12 foundation discipline + the "no half-completion"
requirement:

- **Arc 146 slice 2** (in flight, sonnet) — migrates `length` via
  the OLD pattern (manual check + runtime + retire). Half-
  completion-prone but auditable in this one case.
- **Arc 146 slices 3-7** — would migrate empty?, contains?, get,
  conj, plus the pure-rename family. EACH is a half-completion
  risk if shipped before arc 147.

**Decision (user direction 2026-05-03):** ship arc 147 AFTER arc
146 wraps up. If arc 146 reveals we need 147 sooner, we pivot
and make it first.

The pivot signal is concrete drift evidence — e.g.:
- Sonnet's slice 2 review surfaces a check/runtime inconsistency
  that arc 147 would have structurally prevented
- A migration slice (3-7) hits the half-completion bug class
  during execution
- The aggregate cost of N more manual migrations exceeds arc
  147's investment

If any of these surfaces, arc 147 jumps ahead.

If arc 146 ships clean, arc 147 follows naturally; the existing
~150 primitives retrofit happens in arc 147 slices 2-N regardless
of order.

This is the substrate-as-teacher pattern at the arc level: let
the work surface evidence; respond to evidence; don't lock the
plan in advance.

## Why this is foundation work (not velocity work)

Per § 12: arc 109's wind-down friction IS the foundation auditing
itself. Arc 146 surfaced "polymorphic primitives violate the
substrate's design constraint." Arc 147 surfaces "primitive
registration violates the no-drift constraint."

Both are real cracks in the foundation. Both are addressable
substrate-level. Both compound into the impeccable foundation
the user's next-leg work requires.

The slow path is the right path.

## Cross-references

- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 9 + § 10 + § 12 —
  the discipline this arc embodies (no known defect; pivot don't
  defer; foundation > velocity)
- `docs/arc/2026/05/144-uniform-reflection-foundation/REALIZATIONS.md`
  — the entity-kind discipline lesson
- `docs/arc/2026/05/146-container-method-correction/DESIGN.md`
  + REALIZATIONS — the precedent arc that surfaced this drift
  class
- `crates/wat-macros/` — existing proc-macro infra (template for
  arc 147's macro)
- `src/check.rs::register_builtins` (~line 11000+) — the check-
  side registration site that arc 147 replaces
- runtime dispatch in `eval_list_call` — the runtime-side
  registration site that arc 147 replaces
- Cross-language references: Common Lisp's `defun`, Clojure's
  `defn`, Racket's `define`, Rust's `#[derive(...)]` — single-
  declaration-emits-multiple-consequences pattern is universal
  in macro-capable languages

## Status notes

- DESIGN drafted.
- Implementation deferred until arc 146 slice 2 returns.
- Arc 109 v1 closure now blocks on arc 144 + arc 130 + arc 145 +
  arc 146 + arc 147.
- The "substrate is impeccable" milestone moves further out —
  but each arc compounds; the foundation strengthens with each.
