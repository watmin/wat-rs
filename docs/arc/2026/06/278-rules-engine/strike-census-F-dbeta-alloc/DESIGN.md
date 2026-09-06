# DESIGN — census F: `dbeta:alloc` is a non-empty flag, not an allocation count

Census audit section F (`../vigilia-2026-09-05/recon/census-name-audit.md:86-95`), re-grounded at
HEAD 2026-09-06.

## ⚠ Severity — LIVE, unlike D and E

D and E were latent: no number was wrong. **This one is wrong in front of a human, and a
load-bearing identity depends on the name being ignored.**

## The finding

One bump, `src/rete/kernel/fire/mod.rs:1068`:

```rust
census_count_n("dbeta:alloc", u64::from(!out.is_empty()));
```

`u64::from(bool)` — **0 or 1 per call**. The quantity is *"calls whose result was non-empty"*. It
can never exceed `dbeta:calls`, and it is not even an upper bound on allocations: `out` is grown by
repeated `out.extend(...)` in the loop above, so a single call can allocate several times. Its three
siblings (`dbeta:calls`, `dbeta:tokens`, `dbeta:multi`) are all correctly named.

## Three consumers, and the name is already disbelieved by one

1. **`node_share_cost.rs:288`** — `let fire_gathers = counted("dbeta:alloc");`. The consumer
   **renames it in code** to what it actually is, then asserts a real arithmetic identity at `:314`:

   ```rust
   fire_gathers > 0 && fire_gather_tokens == fire_gathers * tokens.len() as u64
   ```

   ★ **And arm L's timing reconstruction is scaled by it** — *"arm L replays {fire_gathers} clones
   of a {}-token vector"*. The usage is correct **because the name is ignored.**

2. **`gather_probe_cost.rs:1229`** — feeds a printed table whose column header is
   **`allocating`** (`:1245`). A reader sees *"allocating 100"* and believes 100 allocations
   happened. **This is the mislabel rendered to a human.**

3. **`accum_cost.rs:141-144`** — the exact-equality NAMES list; a rename must update it.

## ★ The danger is concrete, not stylistic

The name invites a future hand to "fix" the counter so it counts *real* allocations. That number
would exceed `dbeta:calls`, break the identity at `node_share_cost.rs:314`, and **silently
mis-scale arm L's reconstruction** — a timing result derived from a replay count that no longer
matches the thing being reconstructed. The comment at `:314` would still read plausibly.

A counter whose correct use depends on disbelieving its own name is one refactor from a wrong
number.

## THE ONE CONTRACT DECISION

**Rename to `dbeta:nonempty`. Do NOT change what is counted.**

- `dbeta:nonempty` states the population exactly: calls whose result was non-empty, 0 or 1 per call.
- Not `dbeta:gathers` — `dbeta:gather` is already the phase mark at `:1071` and the two would be
  confusable in a census listing.
- **Counting real allocations is affirmatively rejected**: nothing wants that number, and producing
  it would break the live identity above. Record the rejection at the bump site.

## Out of scope = REJECTED

- Changing the counted quantity (above).
- The three correctly-named siblings.
- Census G–M. One section per strike.

## Mutation proof — and unlike D and E, a real one exists

`node_share_cost.rs:314` asserts `fire_gathers > 0`. **Delete the bump → that assertion REDs.**
This is the first of the census strikes where the counter is genuinely gated, so the proof is a
live red rather than an honest statement that none is available. Drive the live test; quote it.

## STOP triggers

1. Any measured value changes → STOP. This is a rename; the quantity is identical.
2. The mutation does not RED → STOP and report. `node_share_cost.rs:314` is supposed to gate this
   counter; if it does not, the DESIGN's severity claim is wrong and F is latent after all.
3. You are tempted to make it count real allocations → STOP. Rejected above, with the reason.
