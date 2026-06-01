# SCORE — Stone 243.6b — `check_program` walker fusion + `collect_hints` triage

## Triage

### Rune A — `collect_hints` (`check.rs:611`): LEAVE

Four-questions verdict:

- **Obvious**: `collect_hints` is called from two alternative render paths (Display `fmt_with_span` and `diagnostic()`). These are alternative-per-error, not both-per-error, so the "double-compute" premise is already weak.
- **Simple**: The proposed fix (a cached `hints` field on the `CheckError` outer struct) requires a 459-site construction cascade — fails Simple for an unproven cold micro-op.
- **Honest**: The function runs only on the error-render path — cold (errors are rendered rarely, never in steady-state eval). 9 cheap string-match fns on a cold path is not a confirmed defect.
- **Good UX**: No user-visible change from either direction. The speculative optimization has no demonstrated need (`feedback_let_need_reveal_through_work`).

**Resolution**: Deleted the `rune:temperare(deferred-stone-243.6)` comment block (4 lines) above `collect_hints`. `collect_hints` function body and all 4 call sites byte-unchanged. No defending comment added (`feedback_dont_document_non_fixes`). This rune is a *reclassified deferral* — the fold was triaged out, not deferred.

---

### Rune B — walker fusion (`check.rs:716`): FIX

9 independent pre-inference per-body validator passes confirmed order-independent by sequi (accumulator-drains: each appends to `&mut errors`, no pass reads another's output). Real (non-cold) structural cleanup: traversal count collapses from 9× function-bodies + 9× forms = 18 traversals to 1× function-bodies + 1× forms = 2 traversals.

---

## Fusion — 9→1 pre-inference passes

The 9 validators fused (in their preserved relative order):

1. `validate_comm_positions` — Arc 110: kernel-comm call position (CommCtx::Forbidden)
2. `validate_channel_pair_deadlock` — Arc 126: refuse both halves of a channel pair at one call site
3. `validate_sandbox_scope_leak` — Arc 140: sandbox-scope leak prevention
4. `validate_bare_legacy_primitives` — Arc 109 slice 1c: bare primitive type tokens rejected
5. `walk_for_legacy_stream` — Arc 109 slice 9d: legacy `:wat::std::stream::*` prefix rejected
6. `walk_for_legacy_telemetry_service` — Arc 109 slice K.telemetry: legacy `:wat::telemetry::Service::*` rejected
7. `walk_for_legacy_lru_cache_service` — Arc 109 slice K.lru: legacy `:wat::lru::CacheService::*` rejected
8. `walk_for_legacy_kernel_queue` — Arc 109 slice K.kernel-channel: legacy `:wat::kernel::Queue*` rejected
9. `walk_for_bare_legacy_console` — Arc 109 § kill-std / Arc 170 slice 1f-η: retired `:wat::console::*` namespace rejected

Fused loop shape:
```rust
for func in sym.functions.values() {
    validate_comm_positions(&func.body, CommCtx::Forbidden, &mut errors);
    validate_channel_pair_deadlock(&func.body, types, &mut errors);
    validate_sandbox_scope_leak(&func.body, sym, &mut errors);
    validate_bare_legacy_primitives(&func.body, &mut errors);
    walk_for_legacy_stream(&func.body, &mut errors);
    walk_for_legacy_telemetry_service(&func.body, &mut errors);
    walk_for_legacy_lru_cache_service(&func.body, &mut errors);
    walk_for_legacy_kernel_queue(&func.body, &mut errors);
    walk_for_bare_legacy_console(&func.body, &mut errors);
}
for form in forms {
    validate_comm_positions(form, CommCtx::Forbidden, &mut errors);
    validate_channel_pair_deadlock(form, types, &mut errors);
    validate_sandbox_scope_leak(form, sym, &mut errors);
    validate_bare_legacy_primitives(form, &mut errors);
    walk_for_legacy_stream(form, &mut errors);
    walk_for_legacy_telemetry_service(form, &mut errors);
    walk_for_legacy_lru_cache_service(form, &mut errors);
    walk_for_legacy_kernel_queue(form, &mut errors);
    walk_for_bare_legacy_console(form, &mut errors);
}
```

Note: `walk_for_restricted_call` (Stone 241.14) is NOT in the fused 9 — it is a separate pass using `sym.functions.iter()` (requires the function name), untouched.

---

## Verify

- `cargo test --release --lib -p wat` → **895 passed; 0 failed; 1 ignored** (behavioral parity confirmed)
- `cargo build --release --tests` → **clean** (1 pre-existing dead_code warning in `probe_arc241_stone15_zombie_purge.rs` — not this stone)
- `grep -n "deferred-stone-243.6" src/check.rs` → **0** (both rune comments removed)

---

## Runes Closed

- **Rune A** (`check.rs:611`): closed by triage (LEAVE — reclassified deferral); comment deleted, code unchanged
- **Rune B** (`check.rs:716`): closed by fusion; rune comment deleted, 9 passes collapsed to 1 per-body traversal

---

## Line Delta

- Removed: 4-line Rune A comment block + 4-line Rune B comment block + 18 individual `for` loops (9 func + 9 form, ~54 lines of loop boilerplate) + 8 inter-pass Arc comment blocks (retired-walker and context comments, ~80 lines)
- Added: 1 fused `for func` block (11 lines) + 1 fused `for form` block (11 lines) + unified header comment (33 lines)
- Net: approximately −100 lines (gross reduction in boilerplate; header consolidates all 9 Arc attributions)

---

Stone 243.6b complete. Tree left dirty per BRIEF.
