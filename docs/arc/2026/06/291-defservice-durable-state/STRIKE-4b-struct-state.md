# Arc 291 — Strike 4b: State is a struct; the soul wears a body (the prophecy, manifested)

**Status: STRIKE-READY (4b-i).** R8's manifestation made concrete. The firm law (builder): **a struct shall
never cross the wire — categorically, by kind; only records (EDN) cross.** This is what lets defservice host
a *resource* State (cache/socket/fd) and `hibernate` honestly: the struct is the body (in-locus resources),
the EDN Snapshot is the soul (crosses). Recon verdict (`is_portable_type(Struct) → false`, `-p wat`): **one
real break** (`service-template`'s old ship-State pattern); the resource crates (`wat-lru`/`wat-holon-lru`)
already comply (struct State in-locus, records/enums on the wire). Minimal and more correct.

## Staging
- **4b-i — the substrate law (this strike).** `is_portable_type(Struct) → false` (categorical, reversing
  254.1's field-recursion). Migrate the one break. The type-level wall.
- **4b-i-b — value-level audit (follow-on).** Once the type-gate is up, audit whether any struct can still
  reach `closure_extract::encode_struct` at runtime; if a non-type-gated path exists, make `encode_struct`
  refuse ("structs are not wire-serializable; use a record"). Belt to the type-gate's suspenders.
- **4b-ii — defservice State → struct.** `:state` mints a `defstruct` (named accessors preserved). The four
  callbacks become the law: `:init`/`:resume` build the struct from EDN *in*; `:stop`/`:hibernate` project
  the struct to EDN *out*. Revises 4a (counter's whole-State hibernate → `i64` projection). No default for
  `:hibernate` (struct can't cross → forces the projection → the gate, automatic).
- **4b-iii — the RED probe: a soul wearing a body.** A defservice whose `:state` holds a genuinely non-EDN
  field; compiles only with a `:hibernate` projection; hibernate→kill→resume sheds and rebuilds the resource.
- **Then:** R1 → full PROBATUM EST · the pause (builder's) · the 291 INSCRIPTION.

## 4b-i — the brief (substrate law)

**The work.** Make a struct categorically non-portable, so a struct can never be a channel/wire payload.

**The changes (exact):**
1. `src/check.rs:13044` — `is_portable_type` Struct arm: `Some(TypeDef::Struct(s)) => s.fields.iter().all(...)`
   → `Some(TypeDef::Struct(_)) => false`. (Keep the comment honest: struct ↛ wire by kind; arc 291 4b
   supersedes 254.1's field-recursion. A struct holds resources + EDN; only records cross.)
2. **Migrate the one break** — `wat-tests/service-template.wat`: `:svc::State` is a `defstruct` shipped over
   channels (`Sender<svc::State>`, `DriverOut`, `Thread<nil,svc::State>`, the final-state-on-disconnect). It
   is **all-EDN data** (`push-count`/`ack-count` i64s) → under the law it **is a record**. Change
   `(:wat::core::defstruct :svc::State [...])` → `(:wat::Record::def :svc::State [...])`; fix any
   `:svc::State/new` / struct-field sites that the record form changes (records have named accessors too;
   the constructor may shift `State/new` → the record ctor). Keep the test's behavior identical.
3. **Un-ignore** the RED probe — `tests/nursery/probe_arc254_channel_payload_portable.rs`:
   `channel_of_all_edn_struct_must_be_rejected` (remove the `#[ignore]`).

**STOP triggers:**
1. STOP if `is_portable_type(Struct) → false` breaks MORE than `service-template` + the known floor — the
   recon said exactly one real break. If the resource crates (`wat-lru`/`wat-holon-lru`) or anything else go
   red, STOP and report the site list (the design says they already comply).
2. STOP if migrating `:svc::State` to a record needs more than a def-form + accessor swap (e.g. it holds a
   genuinely non-EDN field) — surface it; the recon classified it as all-EDN.
3. Do NOT touch `closure_extract::encode_struct` in this strike (that's the 4b-i-b audit follow-on).

**Blast radius:** `src/check.rs` + `wat-tests/service-template.wat` + the nursery probe un-ignore. Do NOT
touch defservice (`wat/service.wat`) — State→struct is 4b-ii.

**Verify (run yourself, report exact output):**
- `cargo test -p wat --test nursery channel_of_all_edn_struct` → 1 passed (the firm rule now rejects it)
- `cargo test -p wat --test nursery` → all green (no over-rejection of portable payloads)
- `cargo test -p wat --no-fail-fast` → orchestrator weighs SET-diff vs HEAD (expect ⊆ {service-template
  fixed → green} ∪ floor; net ∅ new after the migration)
- the resource crates: `cargo test -p wat-lru -p wat-holon-lru` → stay green (they already comply)

Runtime: 15–30 min. Trap-door: the `service-template` record migration (the one break) — everything else is
the 1-line `is_portable_type` flip + the un-ignore.
