# BRIEF — 293 S3a: a parametric `extend-type` self is a proper `Parametric` (unblocks the peer-as-satisfier)

> **Executor: one sonnet SHADOWDANCER.** A tiny, surgical **Rust** strike (one type-representation fix). The
> orchestrator scouted the root and grounded the fix against an existing helper + a RED probe that already fails on
> exactly the gap. Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first; `.claude/worktrees/` illegal).
> `cargo build --release`; `./target/release/cargo-wat <f>`; `cargo nextest run --release` (NEVER `cargo test`).
> **Commit NOTHING.**

## The work (one paragraph)

The `extend-type` body-check types the impl's `self` (`fixed_params[0]`) as a **flat `TypeExpr::Path(ed.type_name)`**
(`src/runtime.rs:709`, in `register_extend_type_surface_impls`). For a **nominal** receiver (`:probe::MemStore`) that
is correct. For a **parametric** receiver — `:wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>` — the whole string is
wrapped in a bare `Path` and **never decomposed** into `TypeExpr::Parametric { head: "wat::kernel::Peer'", args: […] }`.
So any check that pattern-matches `TypeExpr::Parametric` — notably `project_peer_io` (`src/check.rs:11456`, the
`send'`/`recv'` peer-union membership) — misses it and rejects `self` with "expects peer (Thread' | Process' | Peer' |
ThreadSelfPeer')". **Fix:** parse `ed.type_name` into a proper `TypeExpr` with the existing
`crate::types::parse_type_expr` (`src/types.rs:2640` — yields `Parametric` for `<…>`, `Path` otherwise), falling back
to the flat `Path` if parsing errors. That decomposes a parametric receiver so `project_peer_io` recognizes it; a
nominal receiver parses to the identical `Path`, so nothing else changes.

## The exact site (`src/runtime.rs` ~709, inside the `SurfaceMember::Method` arm of `register_extend_type_surface_impls`)

```rust
// TODAY:
if i == 0 {
    crate::types::TypeExpr::Path(ed.type_name.clone())
} else { … }

// AFTER (parse a parametric receiver into a real Parametric; nominal → identical Path):
if i == 0 {
    crate::types::parse_type_expr(&ed.type_name)
        .unwrap_or_else(|_| crate::types::TypeExpr::Path(ed.type_name.clone()))
} else { … }
```

That is the ENTIRE change expected. If a second gap surfaces (see STOP triggers), report it — do not paper over it.

## Why this is the whole gap (grounded — the orchestrator verified each)

- **The symptom is isolated.** `scratchpad/s3-probe-send-isolation.wat` proves `send'` accepts a `Peer'<Kv::Op,Kv::Reply>`
  as a **plain fn param** (it type-checks + prints), because a fn-param annotation parses to `Parametric`. Only the
  **extend-type self** position fails — the one place `self` is a bare `Path` (runtime.rs:709).
- **The union match is `Parametric`-only.** `project_peer_io` (check.rs:11453–11470) `reduce`s the peer type and matches
  `TypeExpr::Parametric { head ∈ {Thread'/Process'/Peer'/ThreadSelfPeer'}, args.len()==2 }`; else the `other` arm errors.
  A `Path("…Peer'<…>")` never matches, and `format_type` prints it as `Peer'<…>` — exactly the observed "got".
- **The helper exists.** `parse_type_expr` (types.rs:2640) is the standard string→TypeExpr parser used across the type
  layer. `parse_type_expr(":probe::MemStore")` → `Path(":probe::MemStore")` (unchanged); `parse_type_expr(
  ":wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>")` → the decomposed `Parametric`.

## Read the rooms, in order
1. `src/runtime.rs:647–716` — `register_extend_type_surface_impls`; the `self`-typing at ~709 (the fix site) + how
   `ed.type_name` / `member_args` feed the impl sig.
2. `src/types.rs:2640` — `parse_type_expr` (the helper); confirm its Path-vs-Parametric behavior on a nominal vs a
   `<…>` keyword.
3. `src/check.rs:11437–11484` — `project_peer_io` (WHY a `Path` self is rejected; do NOT change this — the union stays
   tight).
4. `scratchpad/s3-probe-peer-satisfies.wat` — the RED probe (the full round-trip: a `Peer'` extend-type-satisfies
   `:probe::Kv`, then `(:probe::Kv/put peer req)` forwards over the wire). It fails today on `project_peer_io`.

## The RED probe (already on disk — the gate)
`scratchpad/s3-probe-peer-satisfies.wat` — a real `:probe::kv-store'` service satisfying `:probe::Kv`, an
`extend-type :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply> :probe::Kv` whose bodies `send'`/`recv'` the Op/Reply,
then `main` dials the store and calls `(:probe::Kv/put peer req)` + `(:probe::Kv/get peer req)`. **Expected after the
fix:** prints `peer-as-Kv put ok = true` and `peer-as-Kv get alpha = one`.

## STOP triggers (halt + report, do NOT hack)
1. **STOP-SECOND-GAP:** if the probe clears `project_peer_io` but then fails at a LATER point (runtime dispatch of
   `:probe::Kv/put` through the peer satisfier, the surface-satisfaction edge for a parametric receiver, `match`
   exhaustiveness on the `Reply` in the body), STOP and report the exact new `file:line` + error — that is a second
   stone, not something to patch here.
2. **STOP-UNION-LOOSENED:** do NOT change `project_peer_io` to accept `Path` forms or non-peer types — the union must
   stay tight (only genuine peer parametrics). The fix is at the `self`-typing site, not the union check.
3. **STOP-NOCP:** do NOT change `parse_type_expr`, `defsurface`/S1, or the `defservice`/S2 macro. S3a is ONLY the
   extend-type-self representation.

## The gate (EXPECTATIONS — the orchestrator re-runs these)
| what | command | expected |
|---|---|---|
| the peer-as-satisfier round-trips | `./target/release/cargo-wat scratchpad/s3-probe-peer-satisfies.wat` | prints `peer-as-Kv put ok = true` / `peer-as-Kv get alpha = one` |
| plain-param peer still works (no regression) | `./target/release/cargo-wat scratchpad/s3-probe-send-isolation.wat` | prints `send' on a plain Peer' param type-checks` |
| nominal-receiver extend-types unaffected | `cargo nextest run --release -E 'test(smem_roundtrip) or test(sqlite_store_differential) or test(counter)'` | passed (unchanged) |
| whole floor | `cargo nextest run --release` | verbatim Summary; `0 failed` modulo the known `no_inlined_wat_in_tests` reminder |

Runtime ~30–45 min (a Rust change forces a rebuild + the full suite).

## Final report (structured): the exact diff (the one site) · the verbatim gate results (both probes + the targeted
tests + the whole-floor Summary) · STOP triggers hit or "none" · anything that surprised you (a second gap, a
`parse_type_expr` edge on some existing extend-type receiver, etc.).

## Prior comparable: the `PRIMVS VSVS ANGVLOS PANDIT` fix (`b441c6bf`) — a sibling "the first parametric/real consumer
of a baked-context path walks a never-exercised corner"; and the extend-user-checked strike (`fa8bbcb9`, the R28 honesty
strike) that made user extend-type bodies checked in the first place — this is a representation gap that strike exposed.
