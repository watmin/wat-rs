# Arc 131 Slice 1 — Sonnet Brief

**Goal:** extend arc 117's `type_contains_sender_kind` to
recognize `wat::kernel::HandlePool<T>` as Sender-bearing when
T (after alias resolution) contains a Sender. Lift the
exclusion that arc 117's own source comment named as "future
arc."

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** Slice 1 = check extension + unit tests in
`src/check.rs`. NO `.wat` files modified. Slice 2 (separate
session) sweeps existing service tests that fire the new
check. Slice 3 verifies on arc 130's deadlock case.

## Read-in-order anchor docs

1. `docs/arc/2026/05/131-handlepool-scope-deadlock/DESIGN.md`
   — the rule, the implementation shape, the four-questions
   framing, the failure-engineering provenance. Source of
   truth.
2. `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
   — the parent arc. Read its detection algorithm + the
   limitations section.
3. `src/check.rs:1976-2046` — the existing
   `type_contains_sender_kind` function. The doc-comment at
   lines 1990-1994 names this exact extension as "future arc."
4. `src/check.rs:117` — `CheckError::ScopeDeadlock` variant.
   `offending_kind` is `&'static str`; today's values include
   `"Sender"`. Arc 131 adds `"HandlePool"`.
5. `src/check.rs:366-380` — the existing Display arm for
   ScopeDeadlock. The new HandlePool kind reuses the same
   diagnostic shape; the `offending_kind` interpolation
   changes the kind name.

## What changes

### Edit 1 — extend `type_contains_sender_kind`

In `src/check.rs:1995-2046` (the function body), add a new
match arm AFTER the existing Channel/Sender surface match
and BEFORE the existing alias-peel logic:

```rust
if let TypeExpr::Parametric { head, args } = ty {
    if matches!(
        head.as_str(),
        "wat::kernel::Channel" | "wat::kernel::Sender"
    ) {
        return Some("Sender");
    }
    // Arc 131 — HandlePool is Sender-bearing when its
    // parametric T (after alias resolution) contains a Sender
    // structurally. The HandlePool entries are clones of
    // Sender-carrying handles; clients pop one, drop or use it,
    // but the pool's internal storage keeps Sender clones alive
    // until each handle is popped AND dropped. A sibling pool
    // alongside Thread/join-result on the service driver is
    // the canonical service-test mistake (Console pattern).
    if matches!(head.as_str(), "wat::kernel::HandlePool") {
        for arg in args {
            if type_contains_sender_kind(arg, types).is_some() {
                return Some("HandlePool");
            }
        }
        return None;
    }
    // ... existing alias-peel + arg-recurse logic ...
}
```

The `is_some()` check on the recursive call: if the parametric
T contains a Sender-kind anywhere (after alias peeling), the
HandlePool counts as Sender-bearing. The new return value
`"HandlePool"` is a new offending_kind for ScopeDeadlock.

### Edit 2 — update the doc-comment

In `src/check.rs:1976-1994`, retire the "future arc" caveat
about HandlePool. Replace lines 1990-1994 with:

```rust
///   - `:wat::kernel::HandlePool<T>` IS flagged when T contains
///     a Sender — arc 131 lifted the exclusion. The previous
///     narrowing avoided false-positives on Console's tests,
///     but the structural pattern (pool sibling to Thread with
///     join-result on the destructured Thread) IS deadlock-prone
///     by construction. Console's tests rely on runtime
///     handle-drop ordering; arc 131 makes the discipline
///     structural rather than voluntary.
```

### Edit 3 — accept the new offending_kind

If `offending_kind` is currently typed as `&'static str` (it
is per `src/check.rs:117`), no type change needed. The Display
arm at lines 366-380 already uses `offending_kind` as a
parameter:

```
"scope-deadlock at {}: ... a {}) holds a Sender clone ..."
```

The `{}` interpolates `offending_kind`. With "HandlePool" as
the value, the message reads: "(a HandlePool) holds a Sender
clone..." which is structurally correct. No Display arm
changes needed.

OR if you decide the Display arm should ADAPT to the kind (for
clarity), update the message to be more explicit when the kind
is "HandlePool":

```
"scope-deadlock at {}: Thread/join-result on '{}' would block
forever. Sibling binding '{}' (a HandlePool) holds Sender
clones via embedded Handle / Tx fields; the driver's recv
never sees EOF until the pool is dropped. ..."
```

This is a quality-of-diagnostic concern. Either shape works
structurally; the second is more user-friendly. Lean toward
the second.

### Edit 4 — add unit tests

Add to `src/check.rs::tests` block:

- **`arc_131_handlepool_with_sender_fires`**: hand-craft a
  wat source with the spawn-tuple-destructure pattern. Use a
  user typealias `:my::Spawn<K,V>` =
  `(HandlePool<MyHandle<K,V>>, Thread<unit, unit>)` and
  `:my::MyHandle<K,V>` = `(Sender<MyReq<K,V>>, Receiver<MyReply<V>>)`.
  Bind `(state :my::Spawn<i64, i64>)`, `(driver :Thread)
  (second state)`, then `(Thread/join-result driver)` in body.
  Assert `CheckError::ScopeDeadlock` fires with
  `offending_kind = "HandlePool"`.
- **`arc_131_handlepool_without_sender_silent`**: hand-craft
  `(pool :HandlePool<wat::core::i64>)` sibling to
  `(thr :Thread<...>)` with `Thread/join-result thr` in body.
  Assert NO ScopeDeadlock fires (HandlePool's T is i64 with
  no Sender → not deadlock-prone).

Mirror arc 117's existing unit-test patterns at the end of
`src/check.rs::tests`.

## Constraints

- ONE file changes: `src/check.rs`. No `.wat` files. No other
  Rust files. No documentation.
- The new offending_kind value MUST be the literal string
  `"HandlePool"` (no spaces, no hyphens).
- No new public API. No new types. The existing
  `ScopeDeadlock` variant gains a new `offending_kind` value
  but the variant shape stays unchanged.
- Workspace test will FAIL on existing service tests that fire
  the new check (Console + telemetry + service-template). DO
  NOT fix those tests in this slice — slice 2 handles the
  sweep. Run `cargo test --release -p wat --lib check` to
  verify the unit tests pass; do NOT run the workspace test
  expecting green. The workspace failures are EXPECTED data
  for slice 2's sweep.

## What success looks like

1. `cargo test --release -p wat --lib check` — exit 0; new
   unit tests pass; existing arc 117 tests pass.
2. `grep -n "HandlePool" src/check.rs` — confirms the new arm
   in `type_contains_sender_kind` + the updated doc-comment +
   the Display arm if you updated it.
3. The unit test `arc_131_handlepool_with_sender_fires`
   asserts `offending_kind = "HandlePool"` and the rule fires.
4. The unit test `arc_131_handlepool_without_sender_silent`
   asserts NO error on a HandlePool<i64> + Thread sibling
   pattern.
5. No commits, no pushes.

## Reporting back

Target ~150 words:

1. File:line refs for the new HandlePool arm in
   `type_contains_sender_kind`, the doc-comment update, the
   Display arm change (if any), and the two new unit tests.
2. The exact final form of the new HandlePool arm.
3. Unit test count + pass status (`cargo test --release -p
   wat --lib check`).
4. Workspace test prediction (DO NOT RUN unless you can keep
   it under 60s with timeouts): how many existing tests fire
   the new check? Use `grep -rn "Spawn\|HandlePool::pop" wat/
   crates/*/wat/ wat-tests/ crates/*/wat-tests/` to estimate
   before running. If you do run, expect failures from
   Console + telemetry + service-template tests. Surface
   the count.
5. Honest deltas: anything you needed to invent (e.g. the
   exact Display arm wording for the new HandlePool kind).

## What this brief is testing (meta)

Per `REALIZATIONS.md`'s artifacts-as-teaching, this brief tests
whether the discipline scales to small substrate-extension
arcs that intentionally break existing tests. Arc 117's parent
DESIGN + INSCRIPTION + the source-comment caveat naming this as
"future arc" together teach what's needed; sonnet picks up
where arc 117's author left a deliberate marker.

The "expected workspace failures" framing is new — slice 1
intentionally breaks existing tests because that's the
discipline becoming enforceable. Slice 2 then closes them.
This is a multi-slice arc by design; sonnet's slice 1 is
the surface-level extension, not the consumer sweep.

Begin by reading the DESIGN, then arc 117's INSCRIPTION,
then the existing `type_contains_sender_kind` function. Then
make the edits. Then run unit tests. Then report.
