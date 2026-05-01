# Arc 117 — INSCRIPTION

## Status

Shipped 2026-05-01 mid-arc-114-manual-fix. The deadlock shape this
arc names was hit live during the HologramCacheService.wat
migration: a `make-bounded-queue` allocation at sibling scope to a
`spawn-thread` whose body closure-captured the pair's Receiver,
with `Thread/join-result` at the same scope, deadlocked because
the pair's Sender clone outlived the worker.

The substrate compiled the program. The runtime hung. No
diagnostic. The fix required recognizing the shape from
`SERVICE-PROGRAMS.md § "The lockstep"` by hand. Arc 117 turns the
prose discipline into a structural rule the compiler enforces.

cargo test --release green throughout: 1476 lib + integration
tests; 0 failures. The substrate's HologramCacheService.wat,
service-template.wat, Console.wat, and three telemetry test
files all freeze cleanly under the new check; the deliberately-
broken probe trips the diagnostic with the canonical fix path.

Pushed across the arc-114 manual sweep:
- The check itself: `CheckError::ScopeDeadlock` + walker landed
  with the arc-114 sweep commits
- HologramCacheService.wat 6-step refactor proving the rule
  catches the very bug we made
- Console.wat false-positive narrowing (handle-bearing siblings
  don't trip the rule once `QueueSender` is matched at surface)
- Telemetry tests sweep (Service.wat, WorkUnit.wat, WorkUnitLog.wat)

## What this arc adds

A type-check-time rule that makes the prose discipline from
`SERVICE-PROGRAMS.md` structural. Before arc 117, the discipline
was carried by humans: read the doc, recognize the shape, refactor
to the inner-let* form. After arc 117, the substrate enforces it
at freeze time with a self-describing diagnostic.

### The rule

> At every `:wat::kernel::Thread/join-result thr` (and
> `:wat::kernel::Process/join-result proc`) call, the `let*`
> binding-block that introduced `thr` must NOT contain any
> sibling binding whose type carries a `QueuePair` /
> `QueueSender` (a Sender-bearing kind). Sender clones at the
> same scope as the join-result outlive the worker; the worker's
> recv never sees EOF.

### The diagnostic

```
scope-deadlock: Thread/join-result on '<thread_binding>' would
block forever. Sibling binding '<offending_binding>' carries a
'<offending_kind>' (a Sender-bearing kind). The Sender clone
outlives the worker, so the worker's recv never sees EOF.

Fix: nest the QueuePair allocation + Sender bindings in an inner
let* whose body returns the Thread. See SERVICE-PROGRAMS.md
§ "The lockstep".

  pre:  (let* ((pair ...) (rx ...) (thr (spawn-thread ...))
                (tx ...) (_s1 ...) ...)
          (Thread/join-result thr))   ;; ← deadlock

  post: (let* ((thr (let* ((pair ...) (rx ...) (tx ...)
                            (h (spawn-thread ...))
                            (_s1 ...) ...)
                       h)))
          (Thread/join-result thr))   ;; ← clean
```

Names the rule, names the canonical form, cross-references
SERVICE-PROGRAMS.md. Mirrors arc 110's substrate-as-teacher
shape — the diagnostic IS the fix brief.

## Why

The user direction (2026-05-01):

> we just did a thing i want to catch and panic on - your forms...
> how can we measure this.. and panic if the user doesn't build
> the scope correctly?..
>
> we do no half measures... i want this durably fixed
> indefinitely.... compile time is superior - yes?

The deadlock had been compile-time-undetectable. The
SERVICE-PROGRAMS.md prose carried the discipline; humans (and
agents) had to remember it. When the agent making the
HologramCacheService.wat migration didn't recognize the shape, the
program deadlocked at runtime with no diagnostic. The runtime is
the wrong layer to enforce a discipline the compiler can see.

Arc 117 lifts the discipline into the compiler. Future migrations
trip the check immediately at freeze time — the substrate teaches
the canonical shape rather than letting the bug arrive at runtime.

## Detection algorithm

The check runs after type inference, walking each `let*`:

1. **Locate join-result sites** in body or trailing position:
   `(:wat::kernel::Thread/join-result thr)` or
   `(:wat::kernel::Process/join-result proc)`.
2. **Trace `thr` to its binding** in the surrounding scope chain.
3. **Walk the scope chain** for sibling bindings introduced
   alongside `thr` (same `let*` binding-block).
4. **Check each sibling's type** — if `type_contains_sender_kind`
   surfaces `QueuePair` / `QueueSender`, the rule fires.
5. **Issue `CheckError::ScopeDeadlock`** with the binding names,
   the offending kind, and the canonical-fix hint.

### The lisp-is-data realization

The first cut used substring matching on the rendered type
string. The user named that shape: *"this is incredibly
unfathomably shallow - a single type alias fucks this... wat is a
lisp - a lisp is nothing but data - use the fucking data to help
you"*.

The second cut walks the actual `TypeExpr` from the type registry
and uses `expand_alias` for one-level peeling. But `expand_alias`
unwraps `QueueSender` → `rust::crossbeam_channel::Sender` (the
underlying alias), which broke both the deliberately-broken probe
(false negative — wrapped sender no longer recognized) AND
Console.wat (false positive — every handle-carrying sibling
matched on the substrate's lower-level Sender).

Final shape matches at the SURFACE level: `QueuePair` /
`QueueSender` are the wat-level kinds the rule cares about. Lower-
level senders inside opaque handle structs don't trip — those are
caller-allocated channel architectures, not the make-bounded-queue
+ spawn-thread coupling the rule names. Aliases peel only when
the head is unknown; the wat-visible kinds are matched directly.

## Limitations

The algorithm errs toward false-negatives — preferred over false-
positives, which would block legitimate caller-allocated channel
architectures.

- **Function-keyword bodies are skipped.** When `spawn-thread`
  takes a named-keyword body (instead of a lambda), closure-
  capture analysis can't see across the function-table boundary.
  Future arc tightens by inlining keyword-body closure analysis at
  freeze time.
- **Multi-step rx derivations skipped.** If the captured name is
  `rx2` where `(rx2 :Receiver<T>) (some-helper rx1)` and `rx1`
  came from a pair, the rule doesn't trace through `some-helper`.
- **Tuple-typealias unpacks skipped.** A custom typealias hiding
  a Sender behind a wrapper isn't expanded past the user-named
  type. Future arc widens.
- **`select` over multiple receivers** — the rule treats EACH
  closure-captured receiver as a potential deadlock channel.
  Documented as a limitation; rare in practice.

The false-negative caveats document themselves in the rule's
prose; future slices tighten coverage as patterns surface.

## What this arc closes

- **The discipline-as-prose gap.** Pre-arc-117, the SERVICE-
  PROGRAMS.md lockstep was a discipline humans carried. Post-arc-
  117, the substrate enforces it; the prose explains the why.
- **The arc-114 sweep regression vector.** The arc-114 R-via-join
  retirement multiplied the surface area where the deadlock could
  surface (every old `:wat::kernel::spawn` migration is a fresh
  scope decision). Arc 117 ensures the migration mistakes that
  humans+agents would make trip the compiler instead of the
  runtime.
- **The future arc-109 § J slice 10g vector.** When polymorphic
  `Program/join-result` lands, the rule already applies — same
  shape, same rule, no separate enforcement work.

## Slice walkthrough

### Slice 1 — the check

`src/check.rs` adds:

- `CheckError::ScopeDeadlock { thread_binding, offending_binding,
  offending_kind, span }`
- `validate_scope_deadlock` walker, called after type inference
- `walk_for_deadlock` — recursive AST walk
- `check_let_star_for_scope_deadlock` — the per-let* check
- `parse_binding_for_typed_check` — parses `let*` bindings into
  `(name, TypeExpr)` for analysis
- `type_is_thread_kind` — surface-level recognition of
  `Thread<I,O>` / `Process<I,O>`
- `type_contains_sender_kind` — surface-level recognition of
  `QueuePair` / `QueueSender`, with `expand_alias` peeling for
  unknown-head aliases

### Slice 2 — verification + sweep

- HologramCacheService.wat 6-step refactor with canonical inner-
  let* nesting (Sonnet sweep guided by the new diagnostic)
- service-template.wat refactor (driver-final-state-via-out-channel)
- Console.wat false-positive narrowing pass (the surface-level
  match keeps handle-bearing programs unflagged)
- 3 telemetry test files (Service.wat, WorkUnit.wat,
  WorkUnitLog.wat) refactored
- `tests/wat_spawn_lambda.rs` 4-test rewrite proving each
  canonical pattern (named-define body, inline-lambda body,
  closure-capture rule, non-callable-rejected)

### Slice 3 — closure (this slice)

INSCRIPTION + WAT-CHEATSHEET § 11 + USER-GUIDE common-gotcha row
+ 058 changelog row.

## The four questions (final)

**Obvious?** Yes. Every Sender clone alive at the join site holds
the worker's recv loop open; the rule fires when sibling-scope
analysis surfaces a Sender-bearing binding alongside the Thread.
Same lens as `Drop` ordering in Rust — scope IS shutdown.

**Simple?** Yes. ~120 LOC total: one `CheckError` variant, one
walker, two type-recognition helpers, surface-level matching with
one-level `expand_alias` peel. No flow analysis, no symbolic
execution, no inter-procedural reasoning. Direct AST walk against
the type registry — wat is a lisp, the type registry is data, the
rule reads the data.

**Honest?** Yes. The rule names the prose discipline; the
diagnostic cites the doc; the canonical-fix block IS the fix. False-
negatives are documented; false-positives were specifically
hunted and removed (Console.wat narrowing pass). The substrate
doesn't promise to catch every deadlock — it promises to catch
this shape, the one that surfaced live on the migration path the
arc-114 sweep created.

**Good UX?** Phenomenal. The very migration that surfaced the
bug now catches the bug at freeze time. The substrate teaches the
inner-let* shape; programmers learn the rule from the diagnostic;
SERVICE-PROGRAMS.md becomes the why-doc rather than the how-doc.
Future arcs that ship with new shapes (arc 109 § J's polymorphic
Program/join-result) inherit the rule for free.

## Cross-references

- `docs/SERVICE-PROGRAMS.md § "The lockstep"` — the prose
  discipline this rule structures.
- `docs/ZERO-MUTEX.md § "Mini-TCP via paired channels"` — the
  broader concurrency framework this rule supports.
- `docs/SUBSTRATE-AS-TEACHER.md` — the migration-discipline
  framing the diagnostic follows.
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — the
  precedent: structural rules at type-check time that prevent
  whole bug classes.
- `docs/arc/2026/04/114-spawn-as-thread/INSCRIPTION.md` — the arc
  whose sweep surfaced the deadlock and motivated this rule's
  permanence.
- `docs/WAT-CHEATSHEET.md § 11` — the one-line reminder.

## Queued follow-ups

- **Function-keyword body coverage** — when arc 109 § J ships
  polymorphic `Program/join-result`, the same rule applies; if
  the keyword-body limitation matters, a future slice inlines
  closure analysis across the function-table boundary.
- **Tuple-typealias unpack tracing** — future arc walks through
  user-defined typealiases that hide Senders inside wrappers.
- **`select` selectivity** — the rule's per-receiver treatment
  could narrow if a real false-positive surfaces; today's
  conservative shape is correct for the patterns in the wild.
- **Cross-arc rule consolidation** — arcs 110, 115, 117 are all
  structural type-check rules that prevent bug classes. A future
  meta-arc could consolidate their hint-emission patterns into
  one substrate-as-teacher framework rather than per-arc helpers.
