# Arc 131 — INSCRIPTION

## Status

**Shipped + closed 2026-05-01.** Two slices over the same day:

- **Slice 1** — substrate check extension + 2 unit tests in
  `src/check.rs`. One sonnet sweep: 8/8 hard + 5/5 soft. Commit
  `fb5d2ab`.
- **Slice 2** — consumer sweep across 3 wat-test files
  (Console, telemetry/Console, lru/CacheService) applying inner-
  let* nesting per SERVICE-PROGRAMS.md § "The lockstep". One
  sonnet sweep: 7/8 hard + 4/4 soft (row 5 was a prediction
  miss — file count 1/5th the expected; sonnet correctly refused
  to pad cosmetic edits). Slice 2 also surfaced arc 133's
  tuple-destructure binding-parse gap.
- **Slice 3** — this INSCRIPTION + WAT-CHEATSHEET §10 update +
  cross-references.

The arc surfaced one substrate gap (arc 133, tuple-destructure
binding-parse bypass). Surfaced + named in slice 2's SCORE
doc; arc 133's BRIEF + EXPECTATIONS spawned same evening.

## What this arc adds

Arc 117's `ScopeDeadlock` check originally narrowed
`type_contains_sender_kind` to bare `Sender` and `Channel`
shapes. The function's source comment named HandlePool as a
deliberate exclusion: "future arc — Console's tests rely on
runtime handle-drop ordering". Arc 131 lifted that exclusion.

`HandlePool<T>` is now recognized as Sender-bearing when T
(after alias resolution) contains a Sender structurally. This
matters because `HandlePool::pop` returns Handle clones with
embedded Sender fields; the pool's internal storage keeps
those Sender clones alive until each handle is popped AND
dropped. A pool sibling to a Thread with `Thread/join-result`
on the destructured Thread is the canonical service-test
mistake — and exactly the deadlock pattern arc 130's sweep
tripped over before being killed mid-run.

After arc 131, the structural check fires uniformly across
`Channel`, `Sender`, and `HandlePool`. Console-style services
either nest properly (inner-let* owning the pool + Thread,
returning the Thread) or trip the check.

## The diagnostic

Same shape as arc 117's; the offending_kind interpolates the
new "HandlePool" value:

```
scope-deadlock at <span>: Thread/join-result on '<thr>' would
block forever. Sibling binding '<pool>' (a HandlePool) holds
Sender clones via embedded Handle fields; the driver's recv
never sees EOF until the pool is dropped.

Fix: nest <pool> + the Thread spawn in an INNER let* whose
body returns just <thr>. The outer let* holds only <thr>;
its body's only operation is Thread/join-result. See
SERVICE-PROGRAMS.md § "The lockstep".
```

## Detection algorithm

Same recursive walker as arc 117 (`validate_scope_deadlock`).
The change is in the classifier:

```rust
fn type_contains_sender_kind(ty: &TypeExpr, types: &TypeEnv)
    -> Option<&'static str>
{
    if let TypeExpr::Parametric { head, args } = ty {
        if matches!(head.as_str(),
            "wat::kernel::Channel" | "wat::kernel::Sender") {
            return Some("Sender");
        }
        // Arc 131 — HandlePool's T after alias resolution.
        if head.as_str() == "wat::kernel::HandlePool" {
            for arg in args {
                if type_contains_sender_kind(arg, types).is_some() {
                    return Some("HandlePool");
                }
            }
            return None;
        }
        // ... existing alias-peel + arg-recurse logic ...
    }
    // ... TypeExpr::Tuple recursion ...
}
```

The recursion descends into HandlePool's parametric args. If
any element contains a Sender (after alias expansion), the
HandlePool counts. The new return value `"HandlePool"` is a
new offending_kind for the diagnostic.

## What got surfaced

**Arc 133 — tuple-destructure binding bypass.** Slice 2's
sonnet noted that `parse_binding_for_typed_check` (and its
sibling `parse_binding_for_pair_check`) skip the
`((name1 name2) rhs)` shape entirely. Tests using tuple-
destructure for spawn-tuple bindings get NO classification
input into the check. They might have the deadlock pattern
AND not fire the rule.

The slice 2 work was honest about this: 12 of 15 surveyed
files looked canonical, but some "canonical 12" might be
shielded by the bypass — non-canonical structurally but
silently invisible to the check. Arc 133 closes the bypass.

**Arc 130 (paused).** The substrate cache services were
expected to retire their `:should-panic("channel-pair-
deadlock")` annotations once arc 130 reshaped them to
HandlePool pair-by-index discipline. Arc 130 was killed
mid-run when sonnet's diagnostic work tripped the
HandlePool-not-recognized gap (this arc closed that). The
substrate redesign itself was deferred; only the consumer
test sweep landed via arc 131 slice 2 (inner-let* nesting,
not pair-by-index reshape). See arc 130 DESIGN.md for the
pause notice.

## The four questions

**Obvious?** Yes. The doc-comment in arc 117 named HandlePool
as "future arc". Arc 131 is the future arc; the marker was
deliberate.

**Simple?** Yes. ~25 LOC of classifier extension + 2 unit
tests. Recursion-descent already handled arg shapes; HandlePool
needed one new arm.

**Honest?** Yes. The original narrowing was honest about the
exclusion; the lift is honest about the structural truth
(pool entries DO hold Sender clones). The runtime-drop-
ordering argument that justified the original narrowing was
itself a discipline; arc 131 makes the discipline structural.

**Good UX?** Phenomenal. Test authors get the same
diagnostic feedback for HandlePool sibling-to-Thread mistakes
as for raw Sender / Channel siblings. The check's promise
becomes uniform across the three Sender-bearing shapes.

## Workspace impact

After slice 1: 14 wat-tests across 3 files newly fired the
check. Slice 2 swept all 3 files to inner-let* nesting; only
3 files needed actual edits (the other 12 surveyed were
already canonical from prior arcs 117/119/126/128). Workspace
test exit=0; 100 result blocks all `ok`.

The `:should-panic("channel-pair-deadlock")` annotations on
the 6 deadlock-class tests in `lru/CacheService.wat`,
`HologramCacheService.wat`, and `step-B-single-put.wat` were
preserved — they fire arc 126's check, not arc 117/131's.
They retire when arc 130's substrate redesign ships.

## Cross-references

- `DESIGN.md` — pre-implementation rule + four-questions framing.
- `BRIEF-SLICE-1.md` + `EXPECTATIONS-SLICE-1.md` + `SCORE-SLICE-1.md`
- `BRIEF-SLICE-2.md` + `EXPECTATIONS-SLICE-2.md` + `SCORE-SLICE-2.md`
- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  — the parent rule.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
  — the sibling rule.
- `docs/arc/2026/05/130-cache-services-pair-by-index/DESIGN.md`
  § "PAUSED" — the substrate redesign deferred when this arc
  shipped its enforcement first.
- `docs/arc/2026/05/133-tuple-destructure-binding-check/DESIGN.md`
  — the gap arc 131 slice 2 surfaced.
- `docs/SERVICE-PROGRAMS.md § "The lockstep"` — the canonical
  inner-let* nesting pattern.
- `src/check.rs::type_contains_sender_kind` — the classifier
  this arc extended.
- `src/check.rs::validate_scope_deadlock` — the structural
  walker (unchanged; runs identically post-arc-131).
