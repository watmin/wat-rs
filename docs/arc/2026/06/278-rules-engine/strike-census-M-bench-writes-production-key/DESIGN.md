# DESIGN — census M: a bench replica writes a production census key

Census audit section M (`../vigilia-2026-09-05/recon/census-name-audit.md:129`), re-grounded at
HEAD 2026-09-06. **The last row of the census, and the only one that is a hazard rather than a
name.**

## The finding

`src/rete/kernel/tests/node_share_cost.rs:476` and `:500` — inside arms J and K of a timing
reconstruction:

```rust
super::census_count("filter:test-reuse");
```

A synthetic loop that evaluates no predicate and pushes no token writes a **production** census key,
once per candidate per token, per timing sample.

**And the same file reads that key for a real gate.** `:286`:

```rust
let fire_reuse = counted("filter:test-reuse");
```

feeding the non-vacuity assertion at `:296` (`fire_reuse > 0 && fire_evals == 0`). One file, one
key, written synthetically and read as a measurement — separated by nothing but scope.

## Two corrections to the audit

1. **Only `filter:test-reuse` is written**, at two sites. The audit also names `filter:test-pass`;
   `grep` finds no such call in that file. Second audit overstatement in two sections — see census
   K/L, where *"neither can ever emit a 0 row"* was likewise half wrong.
2. **The bump is deliberate and correct.** Production calls `census_count` at the same point
   (`fire/mod.rs:2286`), so the replica must pay that cost or it under-reconstructs the branch it
   is timing. **Deleting it would be wrong** — this is not stray instrumentation.

## Why it is harmless today, and why that is not a defence

The census window is a **tight closure** — `with_count_census(|| eval_in_frozen(…))` at `:274-278`,
closing 200 lines before the bench arms. Nothing is polluted at HEAD.

But the separation is incidental. It rests on the window's *lexical extent*, not on any rule, and
the file that would be corrupted is the one doing the writing. **Widen that closure by one line and
a synthetic loop starts feeding a live non-vacuity gate** — in the direction that makes it pass.

## THE ONE CONTRACT DECISION

**Give the replica its own key. Do not delete the call.**

`super::census_count("bench:filter-reuse")` — same call, same shape, same cost, and a name that no
production reader can collide with. The timing reconstruction is preserved exactly; only the
namespace moves.

⚠ **Report the J/K arm timings before and after.** The key is a `&'static str` hashed per call;
`"filter:test-reuse"` is 17 bytes and `"bench:filter-reuse"` is 18, chosen to keep them comparable.
On a ~75-80 ns call this should be noise — **but this is a timing reconstruction, so measure it
rather than assume.** A material move is a finding.

## And a gate, because the situation is what should not exist

`census_count` appears in `src/rete/kernel/tests/` at **exactly these two sites** — every other
match in that tree is `census_counted(||`, the window helper. So after the rename the rule has an
**empty exemption list**:

> A `census_count` call under `src/rete/kernel/tests/` may name only a `bench:`-prefixed key.

Same shape as `no_raw_network_keys_in_oracle.rs` and `no_raw_factbag_access.rs`. It removes the
*situation* — a replica able to write a production key — rather than relying on a closure staying
narrow.

## Out of scope = REJECTED

- **Deleting the bumps.** They carry the reconstruction's fidelity.
- Widening or narrowing the census window at `:274`.
- The `emitted ⇒ ever read` lint deferred by census G. `bench:filter-reuse` will be an emitted-never-read
  key — an unread *truth*, which census G established is fine.

## STOP triggers

1. The J/K arm timings move materially → STOP and report both. The reconstruction's fidelity is the
   thing being protected.
2. The gate needs any exemption → STOP; the "exactly two sites" measurement is wrong.
3. Any production census key is still writable from `tests/` after the change → STOP.
