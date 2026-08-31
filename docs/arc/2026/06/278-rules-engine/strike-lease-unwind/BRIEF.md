# BRIEF — make `with-network`'s lease survive an unwind, by giving it an owner

Give the rete arm lease a Rust owner whose `Drop` releases it, so `with-network` needs no release
call and no unwind can skip one. Today the release sits in a `(do (release-session base) result)`
after the body; a wat error and a host panic each skip it, and both are **driven RED already** —
the two probes are in the tree, uncommitted, and are the gate you must turn green. The sibling
form `with-open-file` (`wat/io.wat:40`) has the identical `let`+`do` shape and is safe because its
resource is a Rust value with a `Drop`; you are making `with-network` earn the same parity its own
doc already claims. Read `DESIGN.md` beside this file first — its ★ section pins the one contract
decision, and its "out of scope" section names three shapes that are already rejected with reasons.

## Read in order

1. `wat/rete/syntax.wat:295-311` — `with-network`. The `let` + `do` is the site; the release call
   on `:310` is the one you delete.
2. `wat/rete/syntax.wat:312-345` — `with-overlay`, built ON `with-network` (one release site, not
   two). It inherits the cure; row 4 of the scorecard is where you prove that rather than assume it.
3. `src/rete/kernel/arm.rs:655-740` — `InternEntry`, `ARM_TABLE`, `rete_arm_intern`,
   `rete_arm_release`, and the two runes above the `thread_local!`. The guard and the `try_with`
   change both land here.
4. `src/rete/kernel/arm.rs:1216-1268` — `eval_release_session`. Copy its shape for the new
   primitive: it is the worked example for reaching a Session's network identity
   (`session_network` → `network_identity`) and for the two `TypeMismatch` refusals when either
   step fails.
5. `src/rust_deps/marshal.rs:322-346` — `RustOpaqueInner` + `make_rust_opaque`. The doc there says
   dedicated `Value` variants are discouraged; this is the supported shape for the guard.
6. `src/io.rs:1278` and `src/io.rs:1318` — two live `#[restricted_to(op, prefix)]` uses, the shape
   to copy for fencing the new primitive to `:wat::rete::`.
7. `src/rete/kernel/tests/arm_lease.rs` (tail) — the two RED probes, and row 3 above them
   (`scoped_work_with_network_releases_the_lease_it_takes`), which is the normal-return control
   that must stay green.

## Sketch

```rust
// arm.rs — the owner. ADOPT: takes no new lease; assumes the one compile-all took.
pub(crate) struct ArmLease { id: u64 }
impl Drop for ArmLease {
    fn drop(&mut self) { rete_arm_release(self.id); }
}

// arm.rs — the table must survive TLS teardown order.
pub(crate) fn rete_arm_release(id: u64) {
    let _ = ARM_TABLE.try_with(|t| { /* body unchanged */ });
}

// arm.rs — the primitive, shaped like eval_release_session.
#[restricted_to(":wat::rete::adopt-session-lease", ":wat::rete::")]
pub(crate) fn eval_adopt_session_lease(...) -> Result<Value, EvalBreak> {
    // … session_network → network_identity → make_rust_opaque(":rust::rete::ArmLease", ArmLease { id })
}
```

```
;; syntax.wat — the guard is bound; there is no release call left.
(:wat::core::let [base   (:wat::core::match (:wat::rete::compile-all rules queries) …)
                  lease  (:wat::rete::adopt-session-lease base)
                  result (body-fn base)]
  result)
```

## Blast radius

`src/rete/kernel/arm.rs`, `src/runtime.rs` (dispatch), `src/check.rs` (TypeScheme),
`src/rete/purity.rs` (op list — `release-session` is already there at `:2380`),
`wat/rete/syntax.wat`, `src/rete/kernel/tests/arm_lease.rs`. No Session field, no new `Value`
variant, no `unsafe`.

## Traps named in advance — each with its step

1. **`wat/**/*.wat` is `include_str!`-embedded** (`src/stdlib.rs:416`). A `syntax.wat` edit does
   NOT take effect until you rebuild. **Step:** `cargo build --release` after touching wat, before
   drawing any conclusion from a run. A "the fix didn't work" from a stale binary is the most
   expensive minute in this strike.
2. **The `let` may reject an unused binding.** **Step:** drive it. If the checker refuses `lease`
   as unused, use `(:wat::core::do lease result)` as the body — the guard drops at frame teardown
   either way, so this is a checker accommodation, not a lifetime change.
3. **TLS destruction order.** A guard alive at thread exit drops during teardown, and
   `ARM_TABLE.with()` **panics** if the table went first. **Step:** `try_with`, discard the `Err`.
   The semantics are already "missing id is a no-op", so nothing else changes.
4. **Adopt, do not acquire.** If the primitive takes a *new* lease, the count goes to 2 and the
   guard's single release leaves `compile-all`'s original held forever — the leak survives its own
   cure wearing a green test. **Step:** the guard constructor must not call `rete_arm_intern`.
5. **Exactly one adopt site.** Over-release is a no-op on a missing id, so a double-adopt degrades
   to early eviction (a rebuild) rather than corruption — but it is still wrong. **Step:**
   `grep -n 'adopt-session-lease' wat/` must return exactly one call.
6. **`with-overlay` inherits, but prove it.** It is built on `with-network`, so the cure should
   reach it for free. **Step:** its own probe (scorecard row 4), not an argument from the call graph.

## STOP triggers

- **STOP-1** — if `make_rust_opaque` cannot carry a payload with a `Drop` that runs when the wat
  binding dies, STOP and report it. The whole design rests on that; do not substitute a second
  release call in an error branch.
- **STOP-2** — if deleting the `do` turns any currently-green test red, STOP and report which.
  That is a test asserting the old lifetime, and whether it is the test or the change that is
  wrong is the orchestrator's call, not a thing to quietly edit around.
- **STOP-3** — if `#[restricted_to]` cannot fence the new op to `:wat::rete::`, ship the primitive
  unfenced and **say so by name in the report**. An unfenced mouth is a finding, not a failure.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-silent-zero/DESIGN.md` and the A2 strike's
`EXPECTATIONS.md` — same arc, same cadence: probe RED first, one mutation per arm, and a report
that states per-arm **proven / reachable-but-not-driven / not-reachable-and-why**.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Nine riders before you each returned one prescription of
mine that did not survive contact — an impossible mutation, a `collect()` on a hot path, a
counter-proof that could not fail, a gate spec blind to its own flagship defect. Every one surfaced
because the rider said so; none by a scorecard. If a step here is wrong, that is the finding.
