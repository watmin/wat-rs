# BRIEF — 293 S3-Nature-2: the `:Peer` nature + peer satisfaction (closes Gap B)

> **Executor: one sonnet SHADOWDANCER.** A focused **Rust** strike (add the fourth `Nature` variant + its satisfaction).
> Additive — existing aggregate satisfaction stays byte-identical. Work ONLY in `/home/watmin/work/holon/wat-rs/`
> (`pwd` first; `.claude/worktrees/` illegal). `cargo build`; `./target/release/cargo-wat <f>`; `cargo nextest run
> --release` (NEVER `cargo test`). **Commit NOTHING.** Runs on the `Holder`→`Nature` rename (`4b9a6d7f`).
> Motivation: 278 R32 — a service is a surface whose nature is `:Peer` at a coordinate. A dialed `Peer'` must satisfy a
> `:nature :Peer` surface. `:Peer` is OFF the aggregate rank ladder — an EXACT match, not a floor.

## The work (one paragraph)

Add `Peer` as the fourth `Nature` variant, and teach the surface-satisfaction check that a `Peer'<S::Op,S::Reply>`
extend-typed to a `:nature :Peer` surface `:S` satisfies it. `:Peer` does NOT participate in the contravariant rank
ladder (Struct −1 < Record 0 < HolonRecord +1) — a `:nature :Peer` surface requires the candidate's nature to *be*
`:Peer` **exactly** (an aggregate does not satisfy a peer surface, nor vice-versa). The RED probe
`scratchpad/s3-probe-peer-satisfies.wat` fails today on the unknown nature-root and must round-trip after.

## The exact changes

**`src/types.rs` — the `Nature` enum + methods (~130-174):**
1. Add the `Peer` variant to `enum Nature { Struct, Record, HolonRecord, Peer }`.
2. `is_pure()`: `Nature::Peer => false` — a peer holds a live channel (crosses no comms; only its address does; the
   circuit / 293.W `:ephemeral`-only rule). It joins `Struct` on the impure side.
3. `rank()`: `Nature::Peer => i8::MIN` — an **off-ladder sentinel**. Document WHY: `:Peer` is not on the aggregate
   contravariant ladder; the exact-match branch in `nature_floor_ok` handles `:Peer` surfaces, and `MIN` ensures a peer
   candidate can never clear an *aggregate* surface's rank floor (`MIN >= any aggregate rank` is false). It is never the
   deciding value for a `:Peer` surface (that path branches before rank).
4. `root_keyword()`: `Nature::Peer => ":wat::kernel::Peer'"`.
5. `from_root_keyword()`: add `":wat::kernel::Peer'" => Some(Nature::Peer)` (this is what makes the clause
   `:nature :wat::kernel::Peer'` parse).

**`src/types/surface.rs` — the `:nature` clause error string (~348):** extend the "nature value must be a nature-root
symbol (…)" message to include `:wat::kernel::Peer'`.

**`src/check.rs` — the derivation + the floor:**
6. `derived_nature` (~14854): BEFORE the `is_holon_or_vector`/`is_pure_type`/`Struct` fallthrough, add a branch — if the
   (already reduced) `t` is a `TypeExpr::Parametric { head, .. }` with `head == "wat::kernel::Peer'"`, return
   `Nature::Peer`. (A `Peer'<…>` must derive to `Peer`, not fall through to `Struct`.)
7. `nature_floor_ok` (~14875): change the check to branch on the required nature —
   ```rust
   if let Some(req) = surf.nature {
       let d = derived_nature(actual, types);
       return if req == Nature::Peer { d == Nature::Peer } else { d.rank() >= req.rank() };
   }
   ```
   For every EXISTING (aggregate-nature) surface the `else` branch is the *unchanged* rank floor → byte-identical
   behavior. Only a `:nature :Peer` surface takes the exact-match branch. An aggregate candidate (`d != Peer`) fails a
   `:Peer` surface; a peer candidate (`d == Peer`, `rank == MIN`) fails an aggregate surface.

That is the whole change. The extend-type edge (`:wat::kernel::Peer' <: :S`) is registered by the ordinary
`extend-type` machinery + S3a (`93e936b3`, the parametric self); `assignable` (check.rs:14914) already routes a
parametric `Peer'` → surface via that edge + `nature_floor_ok` — so once the floor accepts `:Peer`, satisfaction lands.

## Read the rooms, in order
1. `src/types.rs:129-174` — the `Nature` enum + the four methods (changes 1-5).
2. `src/types/surface.rs:~333-390` — the `:nature` clause parse + the error string (change; it already calls
   `from_root_keyword`, so parsing follows automatically).
3. `src/check.rs:14854-14882` — `derived_nature` + `nature_floor_ok` (changes 6-7).
4. `src/check.rs:14896-14919` — `assignable` (the Parametric→Path edge path that reaches `nature_floor_ok`; READ, do
   not change — confirm the flow).
5. `scratchpad/s3-probe-peer-satisfies.wat` — the RED probe (the full round-trip). Fails today on the unknown
   nature-root; must print the two round-trip lines after.

## STOP triggers (halt + report, do NOT hack)
1. **STOP-SATISFIES-ON-PEER-SURFACE:** if `:satisfies` (the S2 defservice) breaks when its surface is `:nature :Peer`
   (S1 synthesis or the S2 macro objects to a peer-nature surface), STOP and report the exact error — it may be a
   follow-on stone, not something to hack here.
2. **STOP-REGRESSION:** existing aggregate satisfaction MUST stay byte-identical (the `else` branch unchanged). If any
   pre-existing test changes behavior, STOP and report — the floor branch was altered wrongly.
3. **STOP-NOCP:** do NOT touch S1 synthesis, the S2/`defservice` macro, or `assignable`'s edge logic. This stone is ONLY
   the `Nature::Peer` variant + `derived_nature`/`nature_floor_ok`.
4. Surface (do not fix here) if you notice: the extend-type edge is keyed by the `Peer'` HEAD (`format!(":{head}")`,
   check.rs:14915), so a *wrong-protocol* `Peer'<OtherOp,OtherReply>` could match the head-only edge — note it as a
   possible follow-on precision concern; the correct-protocol case is what this stone proves.

## The gate (EXPECTATIONS — the orchestrator re-runs these)
| what | command | expected |
|---|---|---|
| the peer-as-satisfier round-trips | `./target/release/cargo-wat scratchpad/s3-probe-peer-satisfies.wat` | prints `peer-as-Kv put ok = true` / `peer-as-Kv get alpha = one` |
| aggregate satisfaction unchanged | `cargo nextest run --release -E 'test(smem_roundtrip) or test(sqlite_store_differential) or test(nature) or test(holder)'` | passed (byte-identical) |
| a `:nature :Struct` surface still works | `cargo wat` on a small struct-satisfies-`:nature :Struct` probe (author one) | type-checks |
| whole floor | `cargo nextest run --release` | verbatim Summary; `0 failed` modulo the known `no_inlined_wat_in_tests` reminder |

Runtime ~30-45 min (a Rust change + a full rebuild + the suite).

## Final report (structured): the exact diff (the ~7 sites) · the verbatim gate results (the probe round-trip + the
targeted tests + the whole-floor Summary) · STOP triggers hit or "none" · did `:satisfies` on a `:nature :Peer` surface
work end-to-end, or surface a follow-on · anything that surprised you.

## Prior comparable: S3a (`93e936b3`, the parametric extend-type self) + the `Holder`→`Nature` rename (`4b9a6d7f`, this
stone's foundation). The `nature_floor_ok`/`derived_nature`/rank-ladder machinery is arc 293 K1a/K1b.
