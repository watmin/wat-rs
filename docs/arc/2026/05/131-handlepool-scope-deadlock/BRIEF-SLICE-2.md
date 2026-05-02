# Arc 131 Slice 2 — Sonnet Brief: consumer sweep

**Goal:** refactor every wat-test file that fires arc 131's
new HandlePool ScopeDeadlock check, applying the canonical
inner-let* nesting from SERVICE-PROGRAMS.md § "The lockstep."
After this slice, `cargo test --release --workspace` ships
green with arc 131's check active.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Pre-spawn workspace state:** arc 131 slice 1 in working tree
(uncommitted) at `src/check.rs`. LRU substrate + tests at HEAD
(arc-130 reshape was reverted; arc-126 slice-2 :should-panic
shape is in place).

## Read-in-order anchors

1. `docs/arc/2026/05/131-handlepool-scope-deadlock/DESIGN.md`
   — the rule + canonical-fix pattern.
2. `docs/arc/2026/05/131-handlepool-scope-deadlock/SCORE-SLICE-1.md`
   — slice 1's verified outcome, prediction count (14-20
   tests), surveyed file list.
3. `docs/SERVICE-PROGRAMS.md` § "The lockstep" — the
   canonical inner-let* pattern.
4. `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
   § "The diagnostic" — the pre/post block already shows the
   refactor pattern (arc 117's existing doc applies verbatim).

## The refactor pattern

**Pre (fires arc 131 ScopeDeadlock with kind=HandlePool):**

```scheme
(:wat::core::let*
  (((state :MyService::Spawn) (...))
   ((driver :Thread<...>) (:wat::core::second state))
   ((pool :HandlePool<...>) (:wat::core::first state))
   ((handle :MyService::Handle) (HandlePool::pop pool))
   ;; ... use handle ...
   ((_ :Result<...>) (Thread/join-result driver)))
  ())
```

**Post (state binding nested inside the inner let*):**

```scheme
(:wat::core::let*
  ;; Outer holds ONLY the Thread.
  (((driver :Thread<...>)
    (:wat::core::let*
      ;; Inner owns state + pool + handle + all the work.
      (((state :MyService::Spawn) (...))
       ((d :Thread<...>) (:wat::core::second state))
       ((pool :HandlePool<...>) (:wat::core::first state))
       ((handle :MyService::Handle) (HandlePool::pop pool))
       ((_ :unit) (HandlePool::finish pool))
       ;; ... use handle ...
       )
      ;; Inner returns the Thread; state + pool + handle drop
      ;; at inner-scope exit; driver sees disconnect.
      d)))
  ;; Outer's only operation: join the now-disconnected driver.
  (:wat::kernel::Thread/join-result driver))
```

The pattern is identical to SERVICE-PROGRAMS.md § "The
lockstep" — outer holds Thread, inner owns Senders (now
including pool/handle), inner returns Thread, pool drops at
inner exit, driver sees disconnect, join-result unblocks
clean.

For tests with multiple services (e.g. Console + a domain
service), nest both spawns in the inner scope; outer holds
both Threads (or a Tuple of them) returned from the inner.

## Files to refactor

Sonnet's slice 1 survey identified the scope. Locate them
yourself via:

```bash
grep -rn "HandlePool::pop\|::Spawn>" wat-tests/ crates/*/wat-tests/ | sort -u | head -30
```

Expected ~14-20 files. The canonical sites:

- `wat-tests/console.wat` (and any other top-level wat-tests
  that touch services)
- `wat-tests/service-template.wat`
- `crates/wat-telemetry/wat-tests/telemetry/*.wat` (Console,
  Service, WorkUnit, WorkUnitLog — multiple deftests per file)
- `crates/wat-telemetry-sqlite/wat-tests/telemetry/*.wat`
  (Sqlite + others)
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
- `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`
- `crates/wat-lru/wat-tests/lru/CacheService.wat`

For tests with `:should-panic` annotations (LRU + HolonLRU
+ step-B): KEEP the annotation. The helper-verb signature
fires arc 126's `channel-pair-deadlock` check; that's why
the test has `:should-panic` in the first place. Refactor
the OUTER scope to inner-let* so arc 131 doesn't ALSO fire
(which would shift the panic substring). Result: only arc
126 fires; `:should-panic("channel-pair-deadlock")`
substring matches; test passes.

For other tests (Console, telemetry, sqlite, etc.): no
`:should-panic` needed. After inner-let* nesting, no check
fires; test passes normally.

## Constraints

- Touch ONLY `.wat` test files (the ones that fire arc 131's
  check). No Rust files. No documentation. No commits.
- The inner-let* refactor preserves test SEMANTICS. The work
  done inside should be identical pre/post; only the binding
  scope nesting changes.
- For `:should-panic` tests: keep the annotation; keep the
  substring; just refactor the outer let* shape.
- ~14-20 file edits. >25 = surface and stop.
- `cargo test --release --workspace` MUST exit 0 after all
  refactors. Workspace stays green.

## What success looks like

1. Every wat-test file that fired ScopeDeadlock pre-slice-2
   now passes the freeze check (no ScopeDeadlock fires).
2. Tests that previously passed via :should-panic still
   pass via :should-panic (substring still matches the firing
   check, which is now arc 126 only for those tests).
3. Other tests pass cleanly (no :should-panic needed; no
   check fires).
4. `cargo test --release --workspace` exit=0.
5. No commits.

## Reporting back

Target ~200 words:

1. List of files modified (count + paths).
2. The exact final form of ONE refactored test (so the
   orchestrator can verify the inner-let* shape).
3. For each test that retains `:should-panic`, confirm the
   substring still matches.
4. Workspace test totals (passed / failed / ignored).
5. Honest deltas: any test that needed shape adjustments
   beyond simple inner-let* nesting (e.g. multiple services,
   complex closure-capture, etc.).
6. LOC delta per file (rough; expect mostly nesting changes,
   not net additions).

## What this brief is testing (meta)

Slice 2 is the FIRST consumer-sweep arc in the failure-
engineering chain that touches >5 files. Earlier sweeps were
slice-2-of-1-arc-each (arc 126 slice 2 was 6 sites; arc 122
attribute conversion was small). This is the largest sweep
yet, exercising whether the discipline scales when the brief
is "refactor N files matching a structural pattern."

The refactor pattern is mechanical (canonical from
SERVICE-PROGRAMS.md). The judgment comes in test-by-test:
- Multiple services per file? Multiple Threads? Nest jointly.
- Per-service inner-let* or one combined inner? Combined is
  usually right (joint handle ownership).
- :should-panic preservation? Yes for cache tests; no for
  others.

Begin by running the grep survey to confirm the file list.
Then read SERVICE-PROGRAMS.md § "The lockstep" + arc 117's
INSCRIPTION § "The diagnostic" pre/post block. Then refactor
each test, mirroring the canonical shape. Then verify
workspace test green. Then report.
