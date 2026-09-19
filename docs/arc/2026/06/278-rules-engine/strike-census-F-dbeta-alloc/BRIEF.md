# BRIEF — census F: `dbeta:alloc` → `dbeta:nonempty`

Read `DESIGN.md` first. **This one is LIVE** — the mislabel is printed to a human as a column
header, and a timing reconstruction depends on the name being ignored. A rename plus one printed
label plus one recorded rejection. **The counted quantity does not change.**

## Read in order

1. `src/rete/kernel/fire/mod.rs:1058-1071` — the bump. `u64::from(!out.is_empty())` is 0-or-1 per
   call; `out` is grown by repeated `extend` in the loop just above, which is why this is not even
   an upper bound on allocations. Its three siblings are correct — leave them.
2. `src/rete/kernel/tests/node_share_cost.rs:286-320` — consumer 1. `fire_gathers` already renames
   it in code, and `:314` asserts `fire_gathers > 0 && fire_gather_tokens == fire_gathers *
   tokens.len()`. Read the arm-L comment underneath: the replay count is this number.
3. `src/rete/kernel/tests/gather_probe_cost.rs:1225-1250` — consumer 2. The `Gather` struct's
   `alloc` field and the `row` formatter's **`allocating`** column header. Both need the true word.
   The struct field is internal to that test; rename it too so the table and the field agree.
4. `src/rete/kernel/tests/accum_cost.rs:138-146` — the exact-equality NAMES list. `"dbeta:alloc"`
   is in it; the list is sorted, so keep it sorted after the rename.
5. `tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` — your safety net. A name read by a
   cost test but emitted nowhere is a build failure, so a missed reader cannot become a silent zero.

## Where the rejection gets recorded

At the bump site, write down that counting *real* allocations was considered and rejected: nothing
wants that number, it would exceed `dbeta:calls`, and it would break the identity at
`node_share_cost.rs:314` and silently mis-scale arm L's reconstruction. A future reader should find
the reasoning, not the temptation.

## Mutation proof — a real red is available here

Unlike census D and E, this counter **is** gated. Delete `census_count_n("dbeta:nonempty", …)` and
`node_share_cost.rs:314`'s `fire_gathers > 0` must RED. Drive the LIVE test, quote the failure
verbatim, restore, confirm green. If it does not red, that is STOP-2 and the severity claim in the
DESIGN is wrong.

## Blast radius

`src/rete/kernel/fire/mod.rs` (one literal + the rejection comment) ·
`src/rete/kernel/tests/node_share_cost.rs` (one read, and the doc at `:78-79` names the key) ·
`src/rete/kernel/tests/gather_probe_cost.rs` (one read, the struct field, the column header) ·
`src/rete/kernel/tests/accum_cost.rs` (the NAMES list). **No engine behaviour. No new counter.**

## STOP triggers

1. Any measured value changes → STOP; the quantity is identical, only the name moves.
2. The mutation does not RED → STOP and report; F would be latent and the DESIGN wrong.
3. You are tempted to count real allocations → STOP. Rejected in DESIGN.
4. A sibling (`dbeta:calls`/`tokens`/`multi`) changes → STOP; they are correct.

## Prior result to copy for shape

`../strike-census-B-compiled-calls/SCORE.md` — starred rows, the live mutation quoted verbatim, and
the honest delta named.
