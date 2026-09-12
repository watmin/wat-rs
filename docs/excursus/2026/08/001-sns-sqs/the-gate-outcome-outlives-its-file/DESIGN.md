# DESIGN — the gate outcome outlives its file

Builder: *"the placeholders are the last of our concerns - i think capability is up next?..."*

**Drawn 2026-09-12. NOT STRUCK.** The follow-on to `the-gate-methods-face-an-outcome`, closing the one
tier that stone could not reach.

## Why

`wat/capability.wat` is stdlib manifest position **20**; `wat/service.wat` is **35**
(`src/load/stdlib.rs:156` vs `:341`). `:wat::service::GateOutcome` is declared in `service.wat`, so
**`capability.wat` cannot name it** — and `capability.wat:15` already says so in its own words:

> *grant/revoke return nil (the extend-type faces GateOutcome via require-granted; this file loads
> [before service.wat])*

Consequence: `Capability`/`TypedCapability`'s `grant`/`revoke` keep `-> :wat::core::nil`, and
`grantable-extend` wraps `require-granted`, which **raises**. So the owner methods face an outcome and
**the capability tier still raises** — the last raise on the track, surviving on an ordering technicality.

Tracked at `docs/arc/2026/04/109-kill-std/NOTE-a-surface-cannot-see-a-type-declared-after-it.md`.
⚠ **Zero userland callers** (measured: the only two `(:wat::capability::*/grant|revoke` occurrences in the
corpus are inside `wat/service.wat` itself — the macro's own generated wiring). So this is a
**consistency** fix on an untravelled path, not a live defect. It is small and it is last for that reason.

## The fix: register the enum in Rust, keep its name

Move `GateOutcome` from a `defenum` in `wat/service.wat` to a registration in `src/types.rs`, where
Rust-registered types are visible to **every** `.wat` regardless of load order.

### ⭑ Three things verified before drawing this, not assumed

1. **wat can CONSTRUCT a Rust-registered enum variant.** Proven by existing working code, which is
   stronger than a new probe: `wat/service.wat:2573` constructs the nullary
   `:wat::kernel::RecvOutcome::TimedOut`, and `:2583`/`:2606` construct the tagged
   `(:wat::kernel::RecvOutcome::Malformed cause)`. `RecvOutcome` is registered in `src/types.rs`.
2. **⭑⭑ A namespace may legitimately span Rust and wat — the precedent is already here.**
   `:wat::holon::` has **8** types registered in `src/types.rs` *and* a `wat/holon.wat`;
   `:wat::edn::` has **3** and a `wat/edn.wat`. So `:wat::service::GateOutcome` living in Rust while the
   rest of `:wat::service::` lives in `wat/service.wat` is **the established pattern, not a wart.**
3. **`Purity::Pure` exists in the Rust registration** (20 enums use it), so
   `GateOutcome`'s `:wat::enum::Pure` marker carries over exactly. `Gone`'s payload
   `:wat::kernel::LociDiedError` is itself Rust-registered, so it is nameable from `types.rs`.

## The one contract decision: KEEP THE NAME `:wat::service::GateOutcome`

The alternative is renaming to `:wat::kernel::GateOutcome` for tidiness, since the other Rust-registered
outcome enums live under `:wat::kernel::`. **Rejected.** It would churn **28** references for no gain,
and finding (2) shows a namespace spanning both homes is normal here. **The name is the contract; only
the declaration site moves.**

## What changes

| where | change |
|---|---|
| `src/types.rs` | register `:wat::service::GateOutcome` — `Applied` (unit), `Gone [cause <- LociDiedError]`, `GaveUp [waited-ms <- i64, last <- String]`, `Purity::Pure`, no type params |
| `wat/service.wat` | **delete** the `defenum` at `:3868`. Everything else that names it is unchanged. |
| `wat/capability.wat` | `Capability`/`TypedCapability` `grant`/`revoke` features: `-> :wat::core::nil` becomes `-> :wat::service::GateOutcome` |
| `wat/service.wat` `grantable-extend` (`:3688`) / `typedcap-extend` | stop wrapping `require-granted`; return the outcome |

★ `require-granted` and `gate-faced` **stay** — they become what they always should have been: a
*caller's* choice, not the surface's.

## Out of scope = REJECTED

- **Moving `CallOutcome` and `StopOutcome` too.** The 109 NOTE argues it generalizes, and it does — both
  are declared in `service.wat` and any earlier file needing them hits the same wall. **But no file needs
  them today**, so moving them is speculative, and this campaign has a rule against fixing what nothing
  has asked for. ⚠ If the strike finds a *second* file that needs one, that changes the answer — report
  it rather than bundling.
- **Renaming to `:wat::kernel::`.** Above.
- **The 61 live `Malformed` placeholder arms.** Explicitly last, per the builder.

## Trap-doors named up front

1. **`capability.wat` is 70 lines and loads at position 20.** If anything it declares is needed *before*
   position 20, moving a type into Rust does not help — check what `capability.wat` itself depends on.
2. **The `defenum` deletion and the Rust registration must land together.** Between them nothing
   type-checks; that is the ordinary cascade, not a red.
3. **`cargo build --release` does not compile tests**; `wat/*.wat` is frozen into the binary at build
   time, so a stdlib change means a rebuild before any `.wat` runs.
4. **Clippy is at 0** (`551ed1a41`) and adding to `src/types.rs` risks the `large_enum_variant` family —
   `GateOutcome`'s biggest variant is 2 fields, so it should be clear, but verify rather than assume.
5. **Zero userland callers means the floor may not exercise the changed surface at all.** A green floor is
   therefore weak evidence here; the strike needs a *positive* demonstration that a `Capability`-tier
   grant now returns a value (see EXPECTATIONS row 5).
