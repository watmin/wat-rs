# SCORE — Stone 254.1: channel-payload portability gate

Scored against an INDEPENDENT orchestrator re-run (not the agent's report).

## Scorecard

| # | what | result |
|---|---|---|
| 1 | **load-bearing**: struct-with-`Sender`-field payload rejected (un-ignored probe) | **PASS** — `channel_of_struct_with_opaque_field_must_be_rejected` passes (rejects) |
| 2 | portable payload still accepted | PASS (i64 control green) |
| 3 | parse-gate finding unaffected | PASS (bare-Sender still parse-rejected) |
| 4 | lib baseline preserved | PASS — **940/0/1** (my own re-run) |
| 5 | no over-rejection | PASS — but see Deferral A (enum) for how it was kept green |
| 6 | clippy clean | PASS (no new warnings on check.rs) |

## What shipped

`src/check.rs`: `fn is_portable_type(&TypeExpr, &TypeEnv) -> bool` (after `is_holon_or_record`) + a gate on `make-bounded-channel`'s payload (`infer_make_queue`, the `Ok(t)` arm). `reduce`-canonicalizes first. Record→portable; Struct→all-fields-portable (recurse); scalars/Uuid/char→portable; portable-containers<portable>→portable; Sender/Receiver/ProgramHandle/HandlePool/ChildHandle/IO/ML→non-portable; Fn/Var→non-portable. Probe un-ignored.

## Honest deferrals (GROUNDED, tracked — not buried)

**Deferral A — `TypeDef::Enum → true` (enum payload portability NOT enforced).**
STOP-2 fired: recursing into enum variants reddened 66 lib checks. Cause (verified on disk): `wat/kernel/services/{stdout,stdin,stderr}.wat` define `*Service::Event` defenums carrying `data-rx <- :wat::kernel::Receiver<…>` fields, and the services `make-bounded-channel` those enums. **This is the stdlib's own service-control pattern sending `Receiver`-carrying enums through channels — a uniform-portability CONTRACT VIOLATION inside the stdlib.** It is exactly the non-uniform leak the DESIGN said the migration would surface (the thread-tier-carries-handles tension). Enum portability cannot be enforced until that pattern is redesigned. **Tracked: the stdlib service-control channel redesign is a migration concern (254.2/254.3 — where the thread/process tier ownership of control channels is settled); enum-portability enforcement lands with it.** Cost of NOT deferring: bundling a stdlib redesign into a checker-gate stone (scope creep). The load-bearing 254.1 target (struct payloads) is fully caught.

**Deferral B — `Path None → true` (formal type parameter `:T` treated portable).**
`wat/stream.wat` uses `make-bounded-channel :T 1` (T a formal param). At the abstract body check `:T` resolves to nothing, so it's treated portable by convention. This RESTS ON the assumption that instantiation sites re-check the body with the concrete T (catching a non-portable concrete payload there). **UNVERIFIED — track: confirm parametric-channel instantiation re-checks the payload type, or `(make-bounded-channel :T 1)` instantiated with a non-portable T is a hole.** Follow-up (254.x).

## Calibration

Predicted 15–25 min; actual ~23 min (1 STOP-2 mid-build, resolved). Mode A-ish (one grounded deviation). Agent diagnosis was honest + disk-grounded; orchestrator re-run confirmed; the two deferrals are real, not green-washing.
