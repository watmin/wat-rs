# Arc 158a — Substrate BRIEF (slice 1)

**Drafted 2026-05-07.** Slice 1 of arc 158a (the only substrate slice).

User direction (from REALIZATIONS): *"v2 needs in DESIGN: pre-flight
walker audit BEFORE substrate edit. Enumerate every reader of
let-binding-declared-type. Determine: do they need the declared
type for non-inference purposes, or can they consume inferred type
instead?"*

Arc 158a's answer: walker pattern-matches RHS instead of reading
declared `:T`. Same approach as arc 133's
`extend_pair_scope_with_tuple_destructure`.

## Workspace state pre-spawn

- HEAD: `5b51b67` (arc 158 v1 back-out + REALIZATIONS shipped)
- Working tree: clean (verify `git status -s`)
- Pre-baseline (verified): **2029 / 0 / 0 warnings**

## Goal

Extend `parse_binding_for_pair_check` (src/check.rs:3215) to
accept BOTH binding shapes:

- Legacy `((name :T) rhs)` — read declared `:T` (existing logic;
  unchanged)
- New `(name rhs)` — pattern-match RHS to derive type-ann string

Walker continues to fire on Channel-related deadlock patterns
identically; both shapes feed the same `PairScopeEntry`
downstream.

## Substrate edits

### `src/check.rs::parse_binding_for_pair_check`

Current (read declared :T):

```rust
fn parse_binding_for_pair_check(binding: &WatAST) -> Option<(String, String, WatAST)> {
    let WatAST::List(items, _) = binding else { return None; };
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
        _ => return None,
    };
    Some((name, type_ann_str, items[1].clone()))
}
```

New shape (handle both):

```rust
fn parse_binding_for_pair_check(binding: &WatAST) -> Option<(String, String, WatAST)> {
    let WatAST::List(items, _) = binding else { return None; };
    if items.len() != 2 {
        return None;
    }
    let rhs = &items[1];

    // New shape: (name rhs) where name is a bare Symbol.
    // Pattern-match RHS to derive type-ann string for the
    // closed set of channel-related shapes the walker tracks.
    if let WatAST::Symbol(id, _) = &items[0] {
        let name = id.name.clone();
        let type_ann_str = derive_type_ann_from_rhs(rhs)?;
        return Some((name, type_ann_str, rhs.clone()));
    }

    // Legacy shape: ((name :T) rhs) — read declared :T.
    let WatAST::List(parts, _) = &items[0] else { return None; };
    if parts.len() < 2 {
        return None;
    }
    let name = match &parts[0] {
        WatAST::Symbol(id, _) => id.name.clone(),
        _ => return None,
    };
    let type_ann_str = match &parts[1] {
        WatAST::Keyword(k, _) => k.clone(),
        _ => return None,
    };
    Some((name, type_ann_str, items[1].clone()))
}
```

### NEW `derive_type_ann_from_rhs` helper

Pattern-matches RHS for the closed set of channel-related
shapes. Returns the type-ann string the walker uses, or None
if RHS doesn't match any tracked pattern.

Patterns to recognize:

| RHS shape | Returned type-ann |
|---|---|
| `(:wat::kernel::make-bounded-channel TYPE N)` | `:wat::kernel::Channel<TYPE>` (TYPE is the keyword string) |
| `(:wat::kernel::make-unbounded-channel TYPE)` | `:wat::kernel::Channel<TYPE>` |
| `(:wat::core::first SOMETHING)` | `:wat::kernel::Sender<wat::core::nil>` (the trace machinery downstream resolves the actual element type via pair-anchor) |
| `(:wat::core::second SOMETHING)` | `:wat::kernel::Receiver<wat::core::nil>` (same as above) |

For `first`/`second` patterns: the placeholder `:wat::kernel::Sender<wat::core::nil>` /
`Receiver<wat::core::nil>` matches what
`extend_pair_scope_with_tuple_destructure` already does (lines
3190-3193). The walker's downstream trace logic doesn't depend
on the inner element type for Sender/Receiver detection — only
on the outer head being Sender/Receiver.

For unrecognized RHS shapes: return None. Walker conservatively
gives up tracking that binding; no false positives.

```rust
/// Pattern-match a let-binding's RHS to derive the type-ann
/// string the walker uses, for the closed set of channel-related
/// shapes the pair-deadlock walker tracks. Returns None if the
/// RHS is not recognizable as Channel / Sender / Receiver.
///
/// Mirrors the RHS pattern-match in
/// `extend_pair_scope_with_tuple_destructure` (arc 133); arc 158a
/// extends the recipe to typed-name-position bindings under the
/// new untyped binding shape.
fn derive_type_ann_from_rhs(rhs: &WatAST) -> Option<String> {
    let WatAST::List(items, _) = rhs else { return None; };
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return None,
    };
    match head {
        ":wat::kernel::make-bounded-channel" => {
            // Args: TYPE N → Channel<TYPE>
            let type_kw = match items.get(1) {
                Some(WatAST::Keyword(k, _)) => k.as_str(),
                _ => return None,
            };
            // Strip leading colon for inner-arg syntax (arc 115
            // InnerColonInCompoundArg rule).
            let inner = type_kw.trim_start_matches(':');
            Some(format!(":wat::kernel::Channel<{}>", inner))
        }
        ":wat::kernel::make-unbounded-channel" => {
            let type_kw = match items.get(1) {
                Some(WatAST::Keyword(k, _)) => k.as_str(),
                _ => return None,
            };
            let inner = type_kw.trim_start_matches(':');
            Some(format!(":wat::kernel::Channel<{}>", inner))
        }
        ":wat::core::first" => {
            // The trace machinery downstream resolves the actual
            // element type via pair-anchor. We just need the head
            // to be Sender so the walker classifies correctly.
            Some(":wat::kernel::Sender<wat::core::nil>".into())
        }
        ":wat::core::second" => {
            Some(":wat::kernel::Receiver<wat::core::nil>".into())
        }
        _ => None,
    }
}
```

### Tests

Add 5-7 new tests as a new `mod arc_158a_walker_tests { ... }`
inside `src/check.rs::tests` (or as appropriate per existing
convention):

1. **Walker fires on new-shape Channel binding** —
   `(:wat::core::let ((pair (:wat::kernel::make-bounded-channel :wat::core::i64 1))) ...)` → walker recognizes `pair` as `Channel<i64>`.
2. **Walker fires on new-shape Sender via second** —
   chain `(let ((pair ...) (rx (:wat::core::second pair))) ...)` → walker traces `rx` to pair-anchor.
3. **Legacy shape still works** —
   `(let (((pair :wat::kernel::Channel<wat::core::i64>) (make-bounded-channel ...)) ...) ...)` → walker fires same as before (regression check).
4. **Mixed shape in same let** —
   one binding legacy, one binding new shape; both feed the same scope; walker fires correctly on the deadlock pattern.
5. **Unrecognized new-shape RHS — walker gives up** —
   `(let ((x (:my::user::make-thing))) ...)` → walker doesn't track `x` (no false positive).
6. **arc 128 outer-scope deadlock pattern in new shape** —
   verifies the existing arc 128 test pattern works under new
   binding shape.

## Constraints

- **Substrate-only edits.** EXACTLY 2 files: `src/check.rs`,
  NEW `tests/wat_arc158a_walker_migration.rs` (or extend the
  existing test module — sonnet picks per convention). NO
  consumer wat edits. NO other crate.
- **DO NOT COMMIT.** Working tree dirty until orchestrator
  scores.
- **Pre-existing 2029 baseline must stay green.** Existing
  legacy-shape tests must continue passing. New-shape tests are
  additive.
- **STOP at unexpected red.** Distinguish:
  - **Expected:** new tests in your file (some pass, some
    pending the change).
  - **Unexpected:** existing tests breaking — the legacy path is
    supposed to be unchanged.
- No grinding. No binding-shape change. No consumer sweep.
- Time-box: **45 min wall-clock**.

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/158a-let-binding-walker-migration/DESIGN.md`
   — full read
2. `docs/arc/2026/05/158-untyped-let-bindings/REALIZATIONS.md` —
   v1 back-out lessons; understand WHY this arc exists
3. `src/check.rs::parse_binding_for_pair_check` (line 3215) —
   the function you're extending
4. `src/check.rs::extend_pair_scope_with_tuple_destructure`
   (line 3138) — the closest precedent for RHS pattern-match
5. `src/check.rs::walk_for_pair_deadlock` (line 2879) — the
   walker that consumes `parse_binding_for_pair_check`'s output;
   read to confirm no other code path touches the type-ann
   string in a way that would break with new derivation
6. `src/check.rs::trace_to_pair_anchor` (line 3088) — the trace
   machinery; confirms how pair-anchor identity tracking works
7. `src/check.rs::tests::arc_128_outer_scope_deadlock_still_fires`
   — example test that depends on this walker firing correctly

## Pre-flight verification

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "FAILED|^test result" | tail -5
```

Confirms 2029 / 0 baseline.

## Verification (after edits)

```bash
cargo test --release --lib 2>&1 | tail -10
cargo test --release --workspace 2>&1 | grep -E "test result|FAILED" | head -10
```

Expect: workspace = 2029 + N new tests; 0 failed.

## Reporting (~200 words)

- Pre-flight crawl confirmation
- Edit summary per file with LOC delta
- New test pass count (all should pass — both legacy paths and
  new-shape paths)
- Path classification (Mode A / B / C)
- Honest deltas: especially around
  - The `trim_start_matches(':')` heuristic — does it match how
    the existing walker formats type-ann strings? (Spot-check
    `extend_pair_scope_with_tuple_destructure`'s output format.)
  - Whether the `:wat::kernel::Sender<wat::core::nil>` placeholder
    gets correctly traced — the walker must still fire on
    deadlock patterns even with placeholder element types
  - Any other walker-internal assumptions about how the type-ann
    string is structured

DO NOT commit. DO NOT write a SCORE doc. Orchestrator scores
after your report.

If genuinely impossible (e.g., the walker depends on type-ann
structure in a way the placeholder breaks), STOP and report.

## Time-box

45 minutes wall-clock.

## Why this matters (158a specifically)

User direction 2026-05-07: *"we are doing the hard grunt work
to enable what i have planned... we just need to do the mass
refactors step by step."* Arc 158 v1 surfaced a substrate-
walker coupling that wasn't anticipated; arc 158a is the
explicit stepping-stone that closes the dependency. Arc 158b
(binding shape change v2) ships cleanly atop 158a's settled
walker foundation.

The proactive stepping-stones discipline (recovery doc § 5)
caught this in retrospect; 158a is the pre-flight that v1
should have included.
