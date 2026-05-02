# Arc 133 — Tuple-destructure bindings honor scope-deadlock checks

**Status:** drafted 2026-05-01.

## TL;DR

`src/check.rs::parse_binding_for_typed_check` only recognizes
the typed-name binding shape `((name :type) rhs)`. Tests using
untyped tuple-destructure bindings — `((name1 name2 ...) rhs)`
— are silently skipped by arc 117 + arc 131's scope-deadlock
checks. The deadlock pattern can be structurally present in
such tests and the check WON'T fire. They work today by
runtime luck.

Arc 133 extends `parse_binding_for_typed_check` (or its
sibling) to handle tuple-destructure patterns. For each name
in the destructure, infer the type from the RHS's tuple-type
at the corresponding index. Run the existing classification
(Thread-kind / Sender-kind) on each inferred type.

After arc 133: tests using either binding shape get the same
structural enforcement. The "correct-by-accident" tests
become "correct-by-construction" or fire the check.

## Provenance

Arc 131 slice 2 sonnet sweep (`a53984eec7ea82ee4`) surfaced
this gap. The agent's SCORE-SLICE-2.md report:

> The slice-1 check's `parse_binding_for_typed_check` skips
> untyped tuple destructure (`((pool con-driver) ...)`), so
> several tests were also shielded by that bypass even when
> their structural shape was non-canonical (they were
> correct-by-accident; my refactor makes them
> correct-by-construction).

Arc 131's slice 2 surveyed 15 wat-test files; only 3 needed
refactoring; 12 were canonical. But sonnet noted that some
of the "12 canonical" might actually have non-canonical
shape — they're just SHIELDED by the bypass. The check sees
them as fine; runtime might disagree. Failure-engineering:
every observed gap becomes an arc.

User direction (2026-05-01):

> "ok - we need a new arc after this - the last sonnet run
> lifted a bug?"

## What's wrong today

`src/check.rs::parse_binding_for_typed_check`:

```rust
fn parse_binding_for_typed_check(binding: &WatAST) -> Option<(String, String, Span)> {
    let WatAST::List(items, span) = binding else { return None; };
    if items.len() != 2 {
        return None;
    }
    let pattern = &items[0];
    let WatAST::List(parts, _) = pattern else { return None; };
    if parts.len() < 2 {
        return None;
    }
    let name = match &parts[0] {
        WatAST::Symbol(id, _) => id.name.clone(),
        _ => return None,
    };
    let type_ann_str = match &parts[1] {
        WatAST::Keyword(k, _) => k.clone(),
        _ => return None,    // ← untyped destructure SKIPS here
    };
    Some((name, type_ann_str, span.clone()))
}
```

The function expects `((name :type-keyword) rhs)`. For tuple-
destructure shape `((name1 name2 ...) rhs)`, `parts[1]` is a
`WatAST::Symbol` (another name), not a `Keyword`. The match
arm returns `None`; the binding is silently skipped.

Caller `check_let_star_for_scope_deadlock` then doesn't see
the binding — it can't classify it as Thread-kind or
Sender-bearing. The check has no input for that binding.

## The rule

> When a let* binding pattern is a tuple-destructure
> (multiple names, no type annotation in the user-source),
> infer each name's type from the binding's RHS-inferred-
> type's tuple-element-type at the corresponding index.
> Each (name, inferred-type) pair gets fed into the same
> classifier (`type_is_thread_kind` /
> `type_contains_sender_kind`) the typed-binding path uses.

Concretely:

```scheme
((pool driver) (:wat::lru::spawn 16 1 ...))
;;     RHS type after inference: wat::lru::Spawn<...>
;;     Spawn aliases to (HandlePool<Handle>, Thread<unit, unit>)
;;     pool → HandlePool<Handle>  (Sender-bearing per arc 131)
;;     driver → Thread<unit, unit>  (Thread-kind per arc 117)
```

Both bindings now visible to arc 117/131. If a let* containing
this destructure also has `Thread/join-result driver`, both
checks fire as expected.

## Implementation

### Slice 1 — extend the binding parser

`src/check.rs` adds a new function `parse_binding_for_destructure`
(or extends `parse_binding_for_typed_check` to return a
`Vec<(name, type)>`) that handles both shapes.

Approach (sketch):

```rust
/// Parse a let* binding for type-classification. Handles
/// both shapes:
///   - typed name: `((name :type) rhs)` → 1 name+type
///   - tuple destructure: `((name1 name2 ...) rhs)` → N names,
///     types inferred from rhs's tuple-type
fn parse_binding_for_typed_check(
    binding: &WatAST,
    types: &TypeEnv,
) -> Vec<(String, TypeExpr, Span)> {
    let WatAST::List(items, span) = binding else { return vec![]; };
    if items.len() != 2 {
        return vec![];
    }
    let pattern = &items[0];
    let rhs = &items[1];
    let WatAST::List(parts, _) = pattern else { return vec![]; };
    if parts.is_empty() {
        return vec![];
    }

    // Typed name shape: ((name :type) rhs)
    if parts.len() == 2 {
        if let (WatAST::Symbol(id, _), WatAST::Keyword(k, _)) =
            (&parts[0], &parts[1])
        {
            if let Ok(ty) = parse_type_expr(k) {
                return vec![(id.name.clone(), ty, span.clone())];
            }
        }
    }

    // Tuple-destructure shape: ((name1 name2 ...) rhs)
    // All parts must be Symbols.
    let names: Vec<String> = parts
        .iter()
        .filter_map(|p| match p {
            WatAST::Symbol(id, _) => Some(id.name.clone()),
            _ => None,
        })
        .collect();
    if names.len() != parts.len() {
        return vec![]; // mixed shapes — give up
    }

    // Infer rhs type. Parse_type_expr won't work; we need
    // the type-checker's inferred type for the rhs AST node.
    let rhs_ty = match infer_rhs_type(rhs, types) {
        Some(ty) => ty,
        None => return vec![],
    };

    // Peel rhs_ty to its tuple form (via expand_alias).
    let canonical = crate::types::expand_alias(&rhs_ty, types);
    let elements = match canonical {
        TypeExpr::Tuple(elems) if elems.len() == names.len() => elems,
        _ => return vec![], // not a tuple of matching arity
    };

    names
        .into_iter()
        .zip(elements)
        .map(|(name, ty)| (name, ty, span.clone()))
        .collect()
}
```

The `infer_rhs_type` helper either:
- Reuses the type-checker's already-inferred type info (stored
  in some `TypeEnv` / per-AST-node hashmap), OR
- Re-runs partial inference on the RHS (slower but
  self-contained)

The exact implementation depends on what type-info storage
the substrate already has. Sonnet's slice 1 work needs to
locate this — likely in `crate::check::infer_*` family or
`crate::types::infer_*`.

### Slice 2 — verification

- Add unit tests for both binding shapes:
  - `arc_133_typed_name_binding_still_classified` — existing
    typed-name shape continues working post-refactor.
  - `arc_133_tuple_destructure_with_handlepool_fires` —
    `((pool driver) (some-spawn-fn))` where spawn returns
    `(HandlePool<...>, Thread<...>)` and `Thread/join-result
    driver` is in the let*'s body → arc 117/131 fires
    ScopeDeadlock with offending_binding="pool".
  - `arc_133_tuple_destructure_silent_when_clean` — tuple
    destructure without Sender-bearing elements → no error.
- Workspace test: `cargo test --release --workspace`
  exit=0. Some tests that were "correct-by-accident" may now
  fire the check — those need refactoring to canonical
  inner-let* nesting (mirrors arc 131 slice 2 pattern).
  Estimate: ≤5 tests; if more, slice 2 is needed.

### Slice 3 — closure

INSCRIPTION + cross-references from arc 117 (note that arc
133 closes the typed-only-binding limitation) + arc 131
(notes that the bypass surfaced in slice 2 is now closed).

## The four questions

**Obvious?** Yes. The bypass is a Level 1 lie about what the
check covers. Tuple-destructure is valid wat syntax; the
check should see those bindings. Arc 117's promise ("scope-
deadlock fires on the structural pattern") doesn't hold when
the binding shape doesn't match.

**Simple?** Medium. The destructure-parsing logic itself is
small (~30 LOC). The harder part is `infer_rhs_type` — finding
the type-checker's per-AST-node type info. If the substrate
already exposes this (via TypeEnv or a per-node hashmap), the
arc is small. If not, a partial-inference re-run is needed.

**Honest?** Yes. The check claims to enforce scope-deadlock
discipline; the bypass undermines the claim for any test using
tuple destructure. Closing the bypass makes the discipline
universal.

**Good UX?** Phenomenal. Future authors using tuple
destructure get the same diagnostic feedback as those using
typed-name bindings. The check's promise becomes uniform.

## Cost — existing tests

Tests using tuple-destructure with deadlock patterns are
"correct by accident" today. Post-arc-133, they fire the
check. Estimated: ≤5 tests (sonnet noted "several tests" in
the slice 2 SCORE; need to enumerate). Each gets refactored
to canonical inner-let* nesting per the established arc 117/131
pattern.

If the count is >5, this becomes its own slice 2 sweep
(mirrors arc 131 slice 2's pattern).

## Cross-references

- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  — the parent arc whose check arc 133 extends.
- `docs/arc/2026/05/131-handlepool-scope-deadlock/SCORE-SLICE-2.md`
  — the SCORE doc that surfaced this bypass.
- `src/check.rs::parse_binding_for_typed_check` — the
  function to extend.
- `src/check.rs::check_let_star_for_scope_deadlock` — the
  caller; needs to handle the multi-name return.

## Failure-engineering record

Arc 133 follows the chain. Each substrate-fix arc closed a
gap surfaced by an earlier sweep:

| # | Arc | Surfaced by |
|---|---|---|
| 128 | sandbox-boundary | arc 126 sweep 1 |
| 129 | Timeout vs Disconnected | arc 126 sweep 3 |
| 131 | HandlePool scope-deadlock | arc 130 sweep killed |
| 132 | default 200ms time-limit | user direction (in progress) |
| **133** | **tuple-destructure binding parse** | **arc 131 sweep 2** |

Each non-clean sweep produces a precisely-diagnosed gap; each
arc closes it. The artifacts-as-teaching record continues.
