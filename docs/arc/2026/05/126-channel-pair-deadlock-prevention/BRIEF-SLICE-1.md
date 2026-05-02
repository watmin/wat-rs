# Arc 126 Slice 1 — Sonnet Brief

**Goal:** Add a compile-time check to `src/check.rs` that catches
function-call sites passing both halves of one
`make-bounded-channel` pair. The new check is a sibling to arc 117's
`ScopeDeadlock` — same trace machinery, applied at call sites
instead of spawn-thread closure bodies.

**Constraints:**
- ONE file changes: `src/check.rs`. No other Rust file. No wat
  source. No docs. No tests outside `src/check.rs`'s `#[cfg(test)]`.
- The diagnostic substring `channel-pair-deadlock` MUST appear in
  the panic message exactly as spelled (load-bearing — slice 2
  matches against it).
- All existing tests stay green.
- ~200 LOC. If it grows beyond ~300, stop and report — the design
  is wrong; the build doesn't proceed past the spec.

## Read-in-order anchor docs

1. `docs/arc/2026/05/126-channel-pair-deadlock-prevention/DESIGN.md`
   — the rule, the trace algorithm, the Display contract, the
   diagnostic substring lock. Source of truth.
2. `docs/arc/2026/04/117-scope-deadlock-prevention/DESIGN.md` and
   `INSCRIPTION.md` — the precedent. Same trace machinery, sibling
   rule. Read these before touching `src/check.rs`.
3. `src/check.rs` — your worked template lives in this file. Read
   the arc-117 functions FIRST, in this order:
   - `enum CheckError::ScopeDeadlock { ... }` (~line 117) — the
     variant shape you mirror
   - `validate_scope_deadlock` (~line 1327) — entry walker
   - `walk_for_deadlock` (~line 1734) — recursive AST walk
   - `check_let_star_for_scope_deadlock` (~line 1796) — per-let*
     check
   - `parse_binding_for_typed_check` (~line 1853) — name+type+span
     extractor (REUSE — don't duplicate)
   - `type_is_thread_kind` (~line 1876) — type classifier
   - `type_contains_sender_kind` (~line 1908) — type classifier
     (REUSE for arc 126's Sender side)
   - `contains_join_on_thread` (~line 1966) — AST predicate walker
4. `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" + § "Routing
   acks" — the doctrine the diagnostic cites.

## What you're adding

### 1. CheckError variant

Add to the `enum CheckError` block (after `ScopeDeadlock`):

```rust
/// Arc 126 — a function call passes two arguments that trace
/// back to the same `:wat::kernel::make-bounded-channel` /
/// `make-unbounded-channel` pair-anchor. One arg is a `Sender<T>`;
/// the other is a `Receiver<T>`; both are halves of one channel.
/// Holding both ends in one role deadlocks any recv on the
/// Receiver — the caller's Sender clone keeps the channel alive
/// even if the receiving thread dies.
ChannelPairDeadlock {
    callee: String,
    sender_arg: String,
    receiver_arg: String,
    pair_anchor: String,
    span: Span,
},
```

### 2. Display arm

Add to the `impl Display for CheckError` block. **The substring
`channel-pair-deadlock` MUST appear verbatim** (slice 2 matches
against it):

```
channel-pair-deadlock at <span>: function call '<callee>' receives
two halves of the same channel pair. Argument '<sender_arg>' is a
Sender<T> and argument '<receiver_arg>' is a Receiver<T>; both
trace back to the make-bounded-channel allocation at '<pair_anchor>'.
Holding both ends of one channel in one role deadlocks any recv —
the caller's writer keeps the channel alive even when the receiving
thread dies.

Fix options (per ZERO-MUTEX.md § "Routing acks"):
  1. Pair-by-index via HandlePool — each producer pops one Handle
     holding ONE end of EACH of two distinct channels.
  2. Embedded reply-tx in payload — caller does not bind the
     reply-tx; project the Sender directly into the Request.
```

Mirror arc 117's `ScopeDeadlock` Display arm for indentation, line
breaks, the `at <span>:` prefix, the structural `Fix:` block at
the bottom. Read its arm verbatim and copy the shape.

### 3. Diagnostic mapping

Add a match arm to `to_diagnostic` or whatever surface
`Diagnostic::new` is called from. Mirror arc 117's
`ScopeDeadlock` mapping. Use the kind string `"ChannelPairDeadlock"`.

### 4. Walker functions

Add four functions (named to mirror arc 117's quartet):

```rust
fn validate_channel_pair_deadlock(
    node: &WatAST,
    types: &TypeEnv,
    errors: &mut Vec<CheckError>,
) {
    walk_for_pair_deadlock(node, types, &Vec::new(), errors);
}

fn walk_for_pair_deadlock(
    node: &WatAST,
    types: &TypeEnv,
    binding_scope: &[(String, WatAST)],  // accumulated let* bindings
    errors: &mut Vec<CheckError>,
) {
    // 1. If node is a let*: extend binding_scope with this let*'s
    //    bindings (name -> rhs); recurse into RHSes + body forms
    //    with the extended scope.
    // 2. If node is a function-call form (List with a keyword head
    //    that's not a kernel comm primitive): run
    //    check_call_for_pair_deadlock on it.
    // 3. Else: descend into children.
}

fn check_call_for_pair_deadlock(
    call_form: &WatAST,
    binding_scope: &[(String, WatAST)],
    types: &TypeEnv,
    errors: &mut Vec<CheckError>,
) {
    // For each argument expression in the call:
    //   - resolve its type (annotation in scope, or Type-by-position)
    //   - if Sender<T> kind: trace to pair-anchor; record
    //   - if Receiver<T> kind: trace to pair-anchor; record
    // Group by pair-anchor; if any anchor has both sender_arg AND
    // receiver_arg, emit CheckError::ChannelPairDeadlock.
}

fn trace_to_pair_anchor(
    name: &str,
    binding_scope: &[(String, WatAST)],
) -> Option<(String, Span)> {
    // Look up `name` in binding_scope.
    // If RHS is `(:wat::core::first <inner>)` or `... second ...`:
    //   recurse on <inner> if it's a Symbol, else return None.
    // If RHS is `(:wat::kernel::make-bounded-channel ...)` or
    //          `(:wat::kernel::make-unbounded-channel ...)`:
    //   return Some((name, span)).
    // Else: return None.
}

fn type_is_receiver_kind(ty: &TypeExpr, types: &TypeEnv) -> bool {
    // Mirror of `type_contains_sender_kind` returning a bool for
    // the Receiver side. Match `:wat::kernel::Receiver` / `Channel`
    // (Channel resolves to (Sender, Receiver) — tuple recursion via
    // the existing infrastructure handles it). Use `expand_alias`
    // for one-level peel on unknown heads.
}
```

### 5. Integration

Call `validate_channel_pair_deadlock` from `check_program`
alongside `validate_scope_deadlock`. Find the existing call site
(grep for `validate_scope_deadlock(` in `check_program`); add the
new call adjacent. Same iteration shape over `func.body` and
top-level forms.

### 6. Unit tests

Add to `src/check.rs`'s `#[cfg(test)] mod tests {}` block. Mirror
arc 117's test patterns:

- **`channel_pair_deadlock_fires_on_canonical_anti_pattern`** —
  hand-craft a let* with `(make-bounded-channel)` + `(first)` /
  `(second)` projections + a call passing both. Assert
  `ChannelPairDeadlock` fires.
- **`channel_pair_deadlock_silent_on_two_different_pairs`** —
  same shape but with two SEPARATE `make-bounded-channel` calls.
  Assert NO error.
- **`channel_pair_deadlock_silent_on_canonical_handle_pop`** — a
  HandlePool/pop returning a (Tx, AckRx) where Tx and AckRx are
  ends of DIFFERENT channels (allocated inside Service/spawn).
  Assert NO error.
- **`channel_pair_deadlock_diagnostic_substring`** — fire the
  rule on the anti-pattern; assert the panic-message string
  contains the substring `"channel-pair-deadlock"` exactly.

Read existing arc-117 tests for their probe-construction pattern.
Reuse the test harness (parse + freeze + assert errors) as-is.

## Verification

Run after each substantial change:

```bash
cargo test --release -p wat --lib check 2>&1 | tail -20
```

This runs `src/check.rs`'s unit tests (the new ones + the existing
arc-117 tests). All must pass.

After the unit tests pass, run:

```bash
cargo test --release --workspace 2>&1 | tail -30
```

Expected outcome — workspace stays GREEN. The 6 ignored deadlock
test sites are STILL ignored (slice 2 unignores them, NOT slice 1).
Slice 1 only adds the check; the existing `:ignore` annotations
keep them off the runtime path. If you see new failures from any
substrate-shipped wat file (Console, telemetry, services), STOP —
the rule is too aggressive; the substrate's pair-by-index pattern
should pass cleanly.

## What NOT to do

- Do NOT touch any `.wat` file. Slice 2 handles the `:ignore` →
  `:should-panic` conversion. Slice 3 handles the doc updates.
- Do NOT modify arc 117's existing functions. Reuse them as-is.
- Do NOT add a config flag, env var, or feature toggle. The check
  always runs.
- Do NOT introduce new public API. The walker is `fn` (private).
- Do NOT change the `expand_alias` machinery. Use it as-is.
- Do NOT add tracing prints / debug logging. The diagnostic IS
  the user surface.
- Do NOT touch `crates/*` — the slice 1 check fires from the wat-rs
  workspace member's `src/check.rs` and reaches every crate's wat
  via the freeze-time check uniformly.
- Do NOT delete the WITHDRAWN arc 125 or arc 127 docs. Honest
  history.

## Reporting back

Report (target ~150 words):

1. The exact lines added in `src/check.rs` (file:line refs of the
   new variant + Display arm + each function's signature).
2. Unit test count: how many added; all passing.
3. Workspace test result: passed / failed / ignored counts.
4. Any tightened or loosened false-negative caveat from the
   DESIGN's § "False-negative caveats" — if your implementation
   covers a case the DESIGN said it wouldn't, name it; if it
   misses a case the DESIGN said it would catch, surface that.
5. The exact panic message your `Display` impl produces on the
   canonical anti-pattern. Include enough context that slice 2 can
   choose the right `:should-panic` substring.

## What this brief is testing (meta)

This brief is the substrate-as-teacher discipline applied to a
compile-time check. The DESIGN names the rule; arc 117 is the
worked code template; the docs cite the doctrine. If a sonnet
reading these can ship slice 1 with the diagnostic substring
matching, the discipline is intact. If not, surface the gap.

Working directory: `/home/watmin/work/holon/wat-rs/`.
