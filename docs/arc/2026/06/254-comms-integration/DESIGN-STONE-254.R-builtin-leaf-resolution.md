# DESIGN — Stone 254.R: resolve-time builtin-leaf validation (annihilate the undefined-func-reaches-runtime class)

**The failure domain, named.** A call head under a reserved prefix
(`:wat::core::`, `:wat::kernel::`, `:wat::holon::`, …) with a WRONG LEAF
(`i64::+'2`, `Bogus`) passes resolve AND check, then dies at RUNTIME as "unknown
function." Behind an `(Err _)` swallow in a spawned thread, that cost a 30-minute
crawl in wat-lru. **Greedy stance: annihilate the class — an undefined func is
ALWAYS a check-time error, never a runtime surprise.**

## Root cause (grounded)

`src/resolve.rs` `is_resolvable_call_head`:
```rust
if is_reserved_prefix(head) {
    return true;   // validates the NAMESPACE, blanket-accepts the LEAF
}
```
The comment hands leaf-validation to "the type checker's concern" — but the
checker doesn't validate builtin leaves either. So leaf-validation for
reserved-prefix builtins is **owned by no one**; the wrong leaf falls through both
gates to runtime. This is the `make-*-queue` phantom class **generalized**: the
set of call-heads the front-end ACCEPTS (any reserved-prefix leaf) is strictly
larger than the set the runtime can DISPATCH. Every name in the gap is a phantom.

resolve already validates non-reserved heads correctly (`sym.get` for registered
fns + defclauses, `unit_variants`, `macros`, single-segment accessors). **The
only hole is the reserved-prefix blanket-accept** — for reserved-prefix names that
are HARDCODED builtins (not in `sym`), there is no membership check.

## The fix (✅✅✅ — single source of truth)

1. **`fn is_dispatchable_builtin(name: &str) -> bool`** — the membership predicate
   listing every hardcoded builtin call-head, derived from the runtime
   keyword-head dispatch (the `":wat::..." =>` arms in `runtime.rs` — find ALL
   dispatch sites; mirror any guard/prefix-pattern arms, not just exact strings).
   This is the source of truth for "is this a dispatchable builtin."
2. **resolve.rs** — replace the blanket `is_reserved_prefix → return true` with:
   a reserved-prefix head is resolvable iff `is_dispatchable_builtin(canonical)`
   OR it already resolves via the existing paths (`sym`, variants, macros,
   accessor). A wrong leaf under a real namespace → **unresolved-reference at
   resolve time** (earliest, clearest gate).
3. **Drift gate** (makes it SSOT, not two hand-synced lists): a test asserting
   every name `is_dispatchable_builtin` accepts actually DISPATCHES (call with
   placeholder args; assert the result is not the "unknown function" error) → no
   phantoms. The existing green corpus catches the dual (a missing real builtin →
   over-rejection → real code fails resolve → add it). Together: the accepted set
   ≡ the dispatchable set, enforced.
4. **Defense in depth — the swallow:** sweep test death-sentinels that `(Err _)`-
   discard a chain → bind and render it (the `:upstream-chain` field exists for
   this). So if anything ever reaches runtime, it stays legible. (May split to a
   sibling stone.)

## Risk + mitigation

Over-rejection (the enumeration misses a real builtin) → real code fails resolve.
This is **self-correcting via the test suite** (substrate-as-teacher): build, run
the full corpus + lib, every over-rejection names the missing builtin; add it.
Iterate to green. Under-rejection (a phantom slips the list) → caught by the drift
gate.

## Probe (RED at HEAD → GREEN after)

`tests/nursery/probe_undefined_builtin_resolves.rs` (committed, RED-verified):
- `wrong_operator_leaf_is_a_check_error` — `(:wat::core::i64::+'2 ...)` must be a
  check error (RED: freezes clean today).
- `bogus_leaf_under_known_namespace_is_a_check_error` — `(:wat::core::Bogus ...)`
  (RED: freezes clean today).
- `valid_operator_still_resolves` — `(:wat::core::i64::+ ...)` stays OK (control,
  green at HEAD; must STAY green — no over-rejection).

## Gates

probe 3/3; lib green; full corpus green (no over-rejection regressions); drift
gate green (no phantoms). Then: an undefined func is a resolve-time
unresolved-reference, always — the 30-minute crawl becomes structurally
impossible, and every undefined-func bug in the sqlite_Db + 170-race fixes that
follow is instantly obvious. It compounds.
