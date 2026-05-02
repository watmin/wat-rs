# Arc 131 — HandlePool counts as Sender-bearing for scope-deadlock

**Status:** drafted 2026-05-01.

## TL;DR

Arc 117's scope-deadlock check explicitly excludes
`:wat::kernel::HandlePool<T>` from its Sender-bearing surface
match — and its own source comment names the resulting hole as
"future arc." Arc 130 slice 1's first sweep hit that hole at
runtime: a `Thread/join-result driver` while the pool's
`HandlePool<Handle>` was alive in scope hung indefinitely,
caught only by the `:time-limit "200ms"` safety net.

Arc 131 lifts the exclusion. Any binding whose alias-resolved
type contains `HandlePool<T>` where T is Sender-bearing
counts as Sender-bearing for arc 117's scope-deadlock rule.
Surface match list grows to include `wat::kernel::HandlePool`.

After arc 131: the exact pattern that hung is rejected at
type-check time with the canonical scope-deadlock diagnostic
naming HandlePool as the offending kind. Future authors who
write the same shape get loud, structural feedback before
runtime.

## Provenance

Arc 130 slice 1 (sonnet sweep `a79f8f3c907a28412`, killed
mid-run) reshaped `:wat::lru::*` to use pair-by-index via
HandlePool. Sonnet's diagnostic-debugging code held the test
shape in this pattern:

```scheme
(:wat::core::let*
  (...
   ((state :wat::lru::Spawn<wat::core::String,wat::core::i64>)
    (:wat::lru::spawn ...))
   ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
    (:wat::core::second state))
   ((handle :wat::lru::Handle<...>) (:wat::kernel::HandlePool::pop pool))
   ((_ :wat::core::unit) (:wat::lru::put handle ...))
   ;; THE DEADLOCK:
   ((_ :Result<...>) (:wat::kernel::Thread/join-result driver))
   ;; ... more work using handle ...
   )
  ...)
```

`Thread/join-result driver` blocks because:
- The driver's per-slot ReqRx side has Sender clones inside
  the pool (`state`'s HandlePool entries) AND inside the
  popped `handle` AND in the substrate driver's matching
  DriverPair vector.
- Driver only terminates when ALL its req-rx clones see EOF
  (i.e. all req-tx senders dropped).
- Caller blocks on `join-result` waiting for driver
  termination; driver blocks on select waiting for a request
  or for senders to drop; senders are alive because `state`
  and `handle` are still bound.

Mutual block. The `:time-limit "200ms"` annotation surfaced it
as a clean panic — the ONLY reason this didn't hang the suite
indefinitely. Failure-engineering says: every observed
deadlock becomes a check.

User direction (2026-05-01):

> "we need to attack every deadlock we find.. the user know
> be told they did something that'll deadlock. this is failure
> engineering - we have a known failure - ensure we panic on
> observation"

## What's wrong today

`src/check.rs:1995-2046` defines `type_contains_sender_kind`
with a surface-level match list:

```rust
if matches!(
    head.as_str(),
    "wat::kernel::Channel" | "wat::kernel::Sender"
) {
    return Some("Sender");
}
```

`wat::kernel::HandlePool` is NOT in the list. The
doc-comment at lines 1990-1994 explains why:

> `:wat::kernel::HandlePool<T>` carries N Senders too but feeds
> a separate service driver; a sibling pool alongside a worker
> Thread isn't necessarily a deadlock (Console.wat case). Pool
> ↔ service-driver siblings ARE a deadlock but require detection
> of the spawn-tuple-destructure pattern; future arc.

The narrowing was tuned to avoid false-positives on Console's
existing tests. But the tradeoff was wrong: Console's tests
have the SAME shape (`(con-state :Console::Spawn) (con-drv
:Thread) (second con-state) ... (Thread/join-result con-drv)`)
that just deadlocked in arc 130's reshape. Console gets away
with it because the runtime sequence happens to drop handles
before the join blocks indefinitely — but the discipline isn't
structural; it's runtime luck.

## The rule

> Add `wat::kernel::HandlePool` to the surface match list in
> `type_contains_sender_kind`. When the binding's alias-resolved
> type is or contains `wat::kernel::HandlePool<T>`, treat the
> binding as Sender-bearing IF T (after alias resolution)
> contains a Sender-kind structurally. The "T contains Sender"
> requirement avoids flagging hypothetical
> `HandlePool<unit>` or `HandlePool<i64>` shapes that aren't
> deadlock-prone.

In code:

```rust
if let TypeExpr::Parametric { head, args } = ty {
    if matches!(
        head.as_str(),
        "wat::kernel::Channel" | "wat::kernel::Sender"
    ) {
        return Some("Sender");
    }
    // Arc 131 — HandlePool is Sender-bearing iff its parametric
    // T contains a Sender after alias resolution. Recurse into
    // args; the existing fallthrough handles this if the args
    // contain a Sender. We add HandlePool to the surface list
    // because its structural presence (not its T) is the signal
    // worth flagging — clients pop a Handle that holds a ReqTx;
    // that ReqTx keeps the driver alive past Thread/join-result.
    if matches!(head.as_str(), "wat::kernel::HandlePool") {
        // Recurse only into args; if T contains Sender, return
        // Some("HandlePool"). Otherwise pass through (a
        // hypothetical HandlePool<i64> isn't deadlock-prone).
        for arg in args {
            if let Some(_inner) = type_contains_sender_kind(arg, types) {
                return Some("HandlePool");
            }
        }
        return None;
    }
    // ... existing alias-peel + arg-recurse logic unchanged ...
}
```

The new "HandlePool" return value is a new `kind` variant for
`CheckError::ScopeDeadlock` — the diagnostic names the kind
specifically so the user knows the cause is HandlePool's
embedded Senders, not a direct Channel/Sender binding.

## The diagnostic

Mirroring arc 117's existing diagnostic shape, with the
`HandlePool` kind:

```
scope-deadlock at <span>: Thread/join-result on '<thread_binding>'
would block forever. Sibling binding '<offending_binding>' (a
HandlePool) holds Sender clones (via embedded Handle / Tx
fields) that outlive the worker; the driver's recv never sees
EOF.

Fix: nest the HandlePool / Handle / state binding in an inner
let* whose body returns '<thread_binding>' — outer scope holds
only the Thread. The inner scope's exit drops the pool and any
popped handles; the driver sees disconnect; join-result
unblocks cleanly. SERVICE-PROGRAMS.md § "The lockstep".

  pre:  (let* ((state :Spawn<...>) (...))
                ((driver :Thread<...>) (second state))
                ((handle :Handle<...>) (HandlePool::pop pool))
                ...
                ((_ :Result<...>) (Thread/join-result driver)))
          ()
        ;; ← deadlock: pool/handle alive at join site

  post: (let* ((driver :Thread<...>)
                (let* ((state :Spawn<...>) (...))
                       ((d :Thread<...>) (second state))
                       ((handle :Handle<...>) (HandlePool::pop pool))
                       ...)
                       d)))
          (Thread/join-result driver))
        ;; ← clean: state + handle drop at inner exit; driver
        ;; sees disconnect; outer holds only the Thread
```

## What this arc closes

- The "future arc" hole arc 117's source comment named.
- The deadlock observed in arc 130 slice 1 (sonnet's diagnostic
  block held the pattern; `:time-limit` saved the suite).
- The class of "Spawn-tuple destructure with join-result while
  pool is alive" — the canonical service-test mistake that
  pair-by-index discipline (Console's pattern) silently
  invites.

## Cost — Console's existing tests

The substrate's Console tests + telemetry tests + any pre-arc-131
service-template tests likely have the
spawn-tuple-destructure shape:

```scheme
((con-state :Console::Spawn) (...))
((con-drv :Thread) (second con-state))
... (use the pool/handle) ...
((_ :Result<...>) (Thread/join-result con-drv))
```

Post-arc-131 they'll fire `ScopeDeadlock`. They need
refactoring to the inner-let* nesting per the canonical fix:

```scheme
((con-drv :Thread)
 (let* ((con-state :Console::Spawn) (...))
        ((d :Thread) (second con-state))
        ... (use pool inside) ...)
        d)
((_ :Result<...>) (Thread/join-result con-drv))
```

This isn't a regression — it's the discipline becoming
enforceable. Console tests work today because clients happen
to drop handles in the right order. Post-arc-131 they're
structurally correct.

The refactoring cost is real but bounded: each affected test
nests its state-binding inside an inner let*. Mechanical but
not zero work. Sonnet sweep can mostly handle it, with manual
review at unusual sites.

## Implementation plan

### Slice 1 — extend the check

`src/check.rs::type_contains_sender_kind` adds the HandlePool
arm. Update the doc-comment to retire the "future arc" caveat.
Update `CheckError::ScopeDeadlock`'s `offending_kind` to
include `"HandlePool"` as a variant.

Add unit tests in `src/check.rs::tests`:

- `arc_131_handlepool_with_sender_fires`: hand-craft a let*
  with `(state :HandlePool<HandleAlias>)` sibling to
  `Thread/join-result thr`. Assert ScopeDeadlock fires with
  kind="HandlePool".
- `arc_131_handlepool_without_sender_silent`: hand-craft a
  let* with `(pool :HandlePool<i64>)` sibling to
  `Thread/join-result thr`. Assert NO error (HandlePool's T
  doesn't contain a Sender → not deadlock-prone).

### Slice 2 — fix the substrate's existing service tests

Sweep tests that fire the new check:

- Console tests in `wat-tests/console.wat` (or similar)
- Telemetry tests in `crates/wat-telemetry/wat-tests/...`
- service-template.wat
- Anywhere else that does `(state :SomeServiceSpawn)`
  sibling to `(Thread/join-result driver)`

Refactor to inner-let* nesting. Workspace stays green.

### Slice 3 — verify on arc 130's deadlock case

After slices 1+2 ship, return to arc 130's
`crates/wat-lru/wat-tests/lru/CacheService.wat` test. The test
currently has the diagnostic block. Run:

```bash
cargo test --release -p wat-lru --test test 2>&1 | tail -20
```

Expected: the LRU test fails AT FREEZE TIME (not runtime) with
`ScopeDeadlock` error naming HandlePool. The
`:time-limit "200ms"` no longer fires — the freeze rejects
the program before runtime.

This is the proof: arc 131's check would have caught arc 130's
slice 1 sonnet sweep failure at compile time.

### Slice 4 — closure

INSCRIPTION + cross-references from arc 117 (note the
"future arc" caveat is now closed) + WAT-CHEATSHEET (extend
§10 Scope-deadlock rule with the HandlePool example).

## The four questions

**Obvious?** Yes. Arc 117's source comment explicitly names
HandlePool as a known gap. The doc-comment IS the spec for the
extension; we just lift the exclusion.

**Simple?** Small. ~15 LOC in `type_contains_sender_kind`
(one new match arm + recursive arg check) + a few lines of
diagnostic update + 2 unit tests. Slice 2's sweep cost is
real but mechanical.

**Honest?** Yes. The current narrowing was tuned for
false-positive avoidance, not structural truth. The deadlock
observed in arc 130 proves the structure IS deadlock-prone.
The rule names the pattern; the diagnostic names the cost; the
fix is the canonical inner-let* nesting that
SERVICE-PROGRAMS.md already documents.

**Good UX?** Yes. Future authors writing
`(state :Spawn) (driver :Thread) (second state) ... (join driver)`
get a clear diagnostic at freeze time. The diagnostic names
the canonical fix shape with a worked pre/post block.

## Cross-references

- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  — the parent arc this extends. Source comment at
  `src/check.rs:1990-1994` names this as future work.
- `docs/arc/2026/05/130-cache-services-pair-by-index/DESIGN.md`
  — the redesign that hit the gap.
- `docs/arc/2026/05/130-cache-services-pair-by-index/BRIEF-SLICE-1.md`
  — sonnet's brief; the diagnostic block sonnet wrote held
  the deadlock pattern.
- `docs/SERVICE-PROGRAMS.md` § "The lockstep" — the canonical
  inner-let* nesting the diagnostic cites.
- `src/check.rs::type_contains_sender_kind` — the function
  that gets the new arm.
- `src/check.rs::ScopeDeadlock` — the variant whose
  `offending_kind` field gains "HandlePool".

## Failure-engineering record

Arc 131 follows the chain:

| # | Arc | Sweep | Hard rows | Substrate gap |
|---|---|---|---|---|
| 1 | arc 126 | sweep 1 | 5/6 | arc 128 (boundary guard) |
| 2 | arc 126 | sweep 2 | 14/14 | none (clean) |
| 3 | arc 126 | sweep 3 | 6/8 | arc 129 (Timeout vs Disconnected) |
| 4 | arc 129 | sweep 4 | 14/14 | none (clean) |
| 5 | arc 130 | sweep 1 (in progress) | TBD | **arc 131 (this)** + arc-130-internal substrate bug |

Pattern continues: each sweep that surfaces a substrate gap
opens a new arc. The arc-130 sweep was killed mid-run; the
deadlock it surfaced becomes arc 131's reason to exist. The
arc-130 internal substrate bug (driver dies after Put) is a
SEPARATE issue that surfaces when arc 130 resumes.

The `:time-limit "200ms"` safety net (arc 123 + arc 129)
caught the hang in this arc's case. Each tool we built carries
its weight: the time-limit converts the hang to data; the
data informs the new arc; the arc closes the gap.
