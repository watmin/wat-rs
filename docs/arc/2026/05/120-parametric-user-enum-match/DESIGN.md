# Arc 120 — Parametric user-defined enum match

**Status:** shipped 2026-05-01.

## Provenance

Surfaced 2026-05-01 mid-arc-119 substrate work. Sonnet's reshape of
`:wat::lru::*` to a parametric enum-based `Request<K,V>` produced a
compile error from the substrate's own type checker:

```
:wat::core::match: parameter scrutinee
  expects :wat::lru::Request;
  got :wat::lru::Request<K,V>
```

The bug had been latent in the codebase since arc 048 (user-defined
enums). Coverage gap masked it: the entire substrate, every
wat-test, every example, and every lab consumer had **zero**
parametric user-defined enums. `Option<T>` and `Result<T,E>` are
parametric but bypass the buggy code path via dedicated MatchShape
variants. Arc 119's `Request<K,V>` is the first parametric user
enum to exist anywhere in the codebase — and the first to surface
the gap.

## Root cause

`MatchShape::Enum(String)` — line 2988 of `src/check.rs` — carried
only the enum's path. When it converted back to `TypeExpr` via
`as_type()` (line 3002 of pre-fix code), it produced
`TypeExpr::Path(name)` regardless of arity. Compare against
`MatchShape::Option(t)` and `MatchShape::Result(t,e)` which carry
their type args explicitly and produce `TypeExpr::Parametric { ... }`
correctly.

The data structure `EnumDef.type_params: Vec<String>` (types.rs:92)
already supported parametric enums — registration captured them
correctly. Only the match-pattern resolver had the gap.

When the type checker tried to unify the scrutinee's declared type
(`Request<K,V>`) against the pattern-derived shape
(`TypeExpr::Path("Request")`), the parametric envelope vs. bare
path mismatched. TypeMismatch fired. The user couldn't match on
their own parametric enum.

## The fix

Two-part substrate change in `src/check.rs`:

1. **`MatchShape::Enum` carries its type-arg vector.**
   Signature: `Enum(String, Vec<TypeExpr>)`. Mirrors how
   `Option(TypeExpr)` and `Result(TypeExpr, TypeExpr)` already
   carry their type args. Empty arg vec ↔ non-parametric (unchanged
   behavior); non-empty ↔ parametric (the new case).

2. **New `enum_match_shape` helper** at the construction sites.
   Looks up the enum's `type_params.len()` from `env.types()`,
   builds `vec![fresh.fresh(); arity]` of fresh type vars,
   constructs `MatchShape::Enum(path, args)`. Three call sites
   in `detect_match_shape` use it (unit-variant pattern path,
   tagged-variant pattern path, and the unit-variant-as-tagged
   fallback).

`as_type()` reads the args vec: empty → `TypeExpr::Path`; non-empty
→ `TypeExpr::Parametric { head, args }`. Unification now succeeds
against the parametric scrutinee.

`apply_subst` (line 2787) updated to apply substitution to each arg
in the vec — fresh vars unify into concrete types as inference
proceeds.

Pattern-matching destructuring sites (5 in check.rs) updated from
`MatchShape::Enum(path)` to `MatchShape::Enum(path, _)` — they read
the path; the args don't affect their logic.

## Coverage gap fix

The test-harness `check()` helper in `src/check.rs::tests` did not
call `register_types` on user source. That meant user-defined enum
declarations in test wat code never reached `env.types()` — and any
test that tried to exercise user-enum match patterns would hit a
fall-through default rather than the actual code path.

This is why no test previously exercised the bug: the harness
silently dropped user type declarations on the floor. The harness
is now updated to:

1. Clone the stdlib type env (a fresh per-test env).
2. Call `register_types` on the user source (mirrors production
   startup pipeline).
3. Pass the post-registration env to `check_program`.

This was a separate latent gap that hid the parametric-enum bug
from any hypothetical earlier test.

## Tests added

Three failing-then-passing tests in `src/check.rs::tests`:

1. `parametric_user_enum_tagged_variant_match` — single type param,
   tagged variants. Minimal repro of the arc-119 surface.
2. `parametric_user_enum_two_type_args_match` — two type params,
   tagged variants. Mirrors `Request<K,V>` directly.
3. `parametric_user_enum_extracts_typed_field` — tagged variant
   binder must inherit the parametric instantiation (`Box<T>` →
   `(Filled v)` binds `v :T`). Verifies field-typing flows.

All three failed against pre-fix code with the exact
`expected: :my::Box; got: :my::Box<T>` error pattern. All three
pass post-fix.

## Workspace test counts

| | passed | failed | ignored |
|---|---|---|---|
| pre-arc-120 baseline | 1476 | 0 | 2 |
| post-arc-120 | 1479 | 0 | 0 |

+3 from new parametric-enum-match coverage. Two pre-existing
ignored doctests in `crates/wat-cli/src/lib.rs` converted from
`rust,ignore` to `text` fences (illustrative consumer code that
referenced external crates outside this crate's deps; never
compilable here).

Net: +3, -2 ignored. Clean green.

## What this surfaces about the codebase

- **The substrate-as-teacher pattern works.** A real use (arc 119
  reshaping LRU's protocol) surfaced a real gap. The substrate's
  own diagnostic stream pointed at the failing wat program; the
  Rust type-checker code path pointed at the latent gap.
- **Coverage is now better.** Parametric user enums are first-class
  going forward. Future arcs can use them without fear.
- **The user's instinct was right** — finding bugs is rare. This
  one had been latent since arc 048 (user-defined enums) but
  needed arc 119's first-parametric-user-enum case to manifest.
  Coverage gaps + behavior gaps compound; surfacing one revealed
  the other.

## Files changed

```
crates/wat-cli/src/lib.rs       |  4 ++--    (doctest ignore → text)
src/check.rs                    | 80 ++++++--  (substrate fix + tests + harness fix)
docs/arc/2026/05/120-.../DESIGN.md | new      (this file)
```

## Sequencing

Arc 120 ships before arc 119 resumes. Arc 119's protocol fix needs
parametric user enums to type-check — that's now possible.

After arc 120 ships, arc 119's substrate work re-attempts steps 2–7
of its execution checklist (the LRU + HolonLRU reshapes). The
sonnet brief for arc 119 step 2 needs no further changes — its
target shape is already locked in
`docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md`.

## Cross-references

- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — the arc
  whose work surfaced this gap.
- `src/check.rs::detect_match_shape` — the function with the gap.
- `src/check.rs::MatchShape` — the enum whose Enum variant gained
  its type-arg vector.
- `src/check.rs::tests::check()` — the helper that lacked
  `register_types` and hid the gap from earlier coverage.
- `src/types.rs::EnumDef` — the data structure that already
  supported parametric enums; only the consumer was wrong.
