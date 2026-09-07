# BRIEF — strike the false purity claim in the userfn probe's ★ paragraph

## The work

One comment-only correction in `tests/rete/probe_arc278_then_user_forms_userfn.wat`. The ★
paragraph at `:10-21` asserts that a user fn constructing a record is refused as impure. It is not,
and the tree records that in two places. No code changes.

## Read in order

1. `tests/rete/probe_arc278_then_user_forms_userfn.wat:10-21` — the ★ paragraph as it stands.
2. `src/rete/kernel/stratify.rs:410-428` — **the record that contradicts it**, including the
   diagnosis: three attempts all used a plain `:wat::core::defn`, so Law A refused the FN and
   *"the table measured ONE door three times and read it as three."*
3. `wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat` — the 2026-09-07 driving.
   `:a2::mk-rate` is a `:wat::rete::core::defn` whose body constructs `(:a2::Rate :count k)`, and
   the file prints `COMPILE: Compiled`. Run it: `cargo run --release --bin wat -- <path>`.

## The correction must carry four things

- the claim is **struck, not deleted** — say it was withdrawn and why, so a reader who remembers it
  is not left hunting;
- **why it was believed** — three probes, one door, counted three times;
- **both drivings** — `stratify.rs:418-424` (2026-08-28) and the scratch fixture (2026-09-07,
  behind `21a5f8514`);
- **what actually guards a minting body** — `rete_fn_body_mints`, `probe_arc278_termination_fn_head.wat`.

⚠ Assert nothing new about `purity.rs`'s ratchet. The original paragraph's failure was describing a
fence it never reached; do not repeat that shape while correcting it.

## STOP triggers

1. **If running the scratch fixture does NOT print `COMPILE: Compiled`** — STOP. The correction's
   own evidence would be false, and that outranks the edit.
2. **If `:tf::first-rate` turns out to construct rather than extract** — STOP and report; the
   paragraph would then be wrong in a second way and the fixture's description with it.
3. Comment only. No `:then` rewrite, no `purity.rs`, no gate widening.

## Blast radius

`tests/rete/probe_arc278_then_user_forms_userfn.wat`, comments only. The probe's own tests must
still pass unchanged.
