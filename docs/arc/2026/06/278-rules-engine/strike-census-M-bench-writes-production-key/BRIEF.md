# BRIEF — census M: move the bench replica off a production key, and gate it

Read `DESIGN.md` first. **Do not delete the bumps** — they carry the timing reconstruction's
fidelity. They move to a `bench:` key, and a gate makes the old shape unwritable.

## Read in order

1. `src/rete/kernel/tests/node_share_cost.rs:462-505` — arms J and K. Each has
   `super::census_count("filter:test-reuse")` inside the hot loop, mirroring production's own call.
2. `src/rete/kernel/fire/mod.rs:2286` — the production call the replica is reconstructing. This is
   why the bump exists and must stay.
3. `src/rete/kernel/tests/node_share_cost.rs:274-278` — the census window. A tight closure around
   one `eval_in_frozen`. This is the only thing separating the synthetic writes from the real read.
4. `src/rete/kernel/tests/node_share_cost.rs:286` and `:296` — `counted("filter:test-reuse")`
   feeding `fire_reuse > 0`. The gate that would be fed synthetic data if the window ever widened.
5. `tests/lint/no_raw_factbag_access.rs` — copy this for the new gate: same shape, same empty
   exemption list, same "has no form" header language.

## What ships

- Both bumps → `super::census_count("bench:filter-reuse")`. Nothing else in those arms changes.
- A new `tests/lint/` gate: a `census_count` call under `src/rete/kernel/tests/` may name only a
  `bench:`-prefixed key. Empty exemption list. Note in its header that `census_counted(||` is the
  window helper and is not this rule's business.
- One comment at the bump saying why the call is there (production parity for the reconstruction)
  and why the key is bench-scoped (it must never reach a production reader).

## Measure the arms

Run the test **before and after** and report arm J and arm K timings both ways. The key is hashed
per call and the two names differ by one byte, chosen to keep them comparable. Expected: noise. A
material move is STOP-1 and a finding — this is a timing reconstruction, so it gets measured rather
than assumed.

## Mutation proof — one rule, one mutation

Put `census_count("filter:test-reuse")` back at one of the two sites and drive the LIVE gate. It
must RED and name the file and key. Quote it, restore, confirm green.

## Blast radius

`src/rete/kernel/tests/node_share_cost.rs` (two literals + one comment) · one new `tests/lint/`
file. **No production code. No behaviour. No window change.**

## STOP triggers

1. Arm J or K timings move materially → STOP, report both sets.
2. The gate needs an exemption → STOP; the "exactly two sites" measurement is wrong.
3. A production key is still writable from `tests/` after the change → STOP.
4. You are tempted to delete the bumps or widen the window → STOP; both rejected in DESIGN.

## Prior result to copy for shape

`../strike-census-F-dbeta-alloc/SCORE.md` — starred rows, live mutation quoted verbatim, before/after
numbers reported rather than described.
