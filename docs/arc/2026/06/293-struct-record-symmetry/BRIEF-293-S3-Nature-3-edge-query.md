# BRIEF — 293 S3-Nature-3: `assignable` queries the full-args extend-type edge (closes Gap B, round-trips the peer)

> **Executor: one sonnet SHADOWDANCER.** A **tiny, surgical Rust** strike (2-3 lines in `assignable`). Additive — every
> existing match path is unchanged. Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first; `.claude/worktrees/`
> illegal). `cargo build`; `./target/release/cargo-wat <f>`; `cargo nextest run --release` (NEVER `cargo test`).
> **Commit NOTHING.** Runs on S3-Nature-2 (`23e8c16f`). This is the LAST substrate gap in the peer-as-satisfier chain
> (S3a `93e936b3` + S3-Nature-2 `23e8c16f` are done); when the RED probe round-trips, "a service is a surface at a
> coordinate" (278 R32) is true in the checker.

## The gap (grounded — the orchestrator verified each string)

A **full-args** parametric `extend-type` — `(:wat::core::extend-type :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>
:probe::Kv …)` — registers its subtype edge under the receiver's **raw keyword string**, verbatim:
`register_subtype(":wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>", ":probe::Kv")` (`src/types.rs:1985`; the key is
`items.get(1)` as-written). But `assignable`'s Parametric→Path branch (`src/check.rs:14930`) queries **head-only**:
`is_subtype(&format!(":{head}"), ep, types)` = `is_subtype(":wat::kernel::Peer'", ":probe::Kv")` — which has no edge (the
edge lives under the full-args key). So a dialed `Peer'<Kv::Op,Kv::Reply>` is never recognized as satisfying `:probe::Kv`
→ `":probe::Kv/put: parameter #1 (receiver) expects :probe::Kv; got Peer'<…>"`. The head-only query was arc-267's, for
**constructor-based** edges (`extend-type :wat::kernel::Peer' :Proto` → any peer); it simply doesn't cover the full-args
form.

## The fix (the whole change — `src/check.rs`, the Parametric→Path branch ~14929)

Query the **full-args key first** (protocol-specific), then fall back to head-only (arc-267). `format_type` (`check.rs:15311`,
same file, `pub`) reproduces the registration key EXACTLY — for `Parametric{head, args}` it emits `:{head}<{args}>` with
inner args colon-stripped: `format_type(Peer'<probe::Kv::Op,probe::Kv::Reply>)` == `":wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>"`
== the registered key (orchestrator verified).

```rust
// BEFORE (check.rs ~14929):
if let (TypeExpr::Parametric { head, .. }, TypeExpr::Path(ep)) = (&a, &e) {
    if crate::types::is_subtype(&format!(":{head}"), ep, types) {
        return nature_floor_ok(&a, ep, types);
    }
}

// AFTER:
if let (TypeExpr::Parametric { head, .. }, TypeExpr::Path(ep)) = (&a, &e) {
    // Full-args edge (a full-parametric extend-type, e.g. Peer'<Op,Reply> <: :S — PROTOCOL-SPECIFIC)
    // OR the arc-267 head-only edge (a constructor-based extend-type, e.g. Vector<T> <: :Proto).
    if crate::types::is_subtype(&format_type(&a), ep, types)
        || crate::types::is_subtype(&format!(":{head}"), ep, types) {
        return nature_floor_ok(&a, ep, types);
    }
}
```

That is the entire change. **Additive:** for a candidate WITHOUT a full-args edge, `is_subtype(&format_type(&a), ep)`
returns false and the head-only check runs exactly as before → byte-identical for every existing extend-type. It also
delivers protocol-specificity for free: a `Peer'<WrongOp,WrongReply>` reconstructs to a key with no registered edge and
no head-only edge → correctly does NOT satisfy `:S`.

## Read the rooms, in order
1. `src/check.rs:14920-14934` — the `assignable` Parametric→Path branch (the change site).
2. `src/check.rs:15311-15339` — `format_type` (confirm it emits `:{head}<{colon-stripped args}>`).
3. `src/types.rs:1958-1986` — the `extend-type` registration (`register_subtype(&type_name, …)`, `type_name` = the raw keyword; confirm the key format the query must match).
4. `scratchpad/s3-probe-peer-satisfies.wat` — the RED probe (the full round-trip). Fails today at the receiver check;
   must print the two round-trip lines after.

## STOP triggers (halt + report, do NOT hack)
1. **STOP-FOURTH-GAP:** if the probe clears the receiver check (assignable now finds the edge) but then fails at a LATER
   point (runtime dispatch of `:probe::Kv/put` through the peer — the send'/recv' at RUNTIME), STOP and report the exact
   new `file:line` + error — that is a distinct (runtime) stone, not something to patch in `assignable`.
2. **STOP-REGRESSION:** the whole floor must stay byte-identical (the change is additive). If any pre-existing test
   changes behavior, STOP and report — the OR was wired wrong or `format_type` doesn't match the key.
3. **STOP-NOCP:** do NOT change `register_subtype`, `is_subtype`, `format_type`, `nature_floor_ok`, or the extend-type
   registration. ONLY the two `is_subtype` calls in `assignable`'s Parametric→Path branch.

## The gate (EXPECTATIONS — the orchestrator re-runs these)
| what | command | expected |
|---|---|---|
| the peer-as-satisfier round-trips | `./target/release/cargo-wat scratchpad/s3-probe-peer-satisfies.wat` | prints `peer-as-Kv put ok = true` / `peer-as-Kv get alpha = one` |
| aggregate + service satisfaction unchanged | `cargo nextest run --release -E 'test(smem_roundtrip) or test(sqlite_store_differential) or test(nature) or test(counter)'` | passed (byte-identical) |
| whole floor | `cargo nextest run --release` | verbatim Summary; `0 failed` modulo the known `no_inlined_wat_in_tests` reminder |

Runtime ~30-45 min (a Rust change + rebuild + the suite).

## Final report (structured): the exact diff (the one branch) · the verbatim gate results (the probe round-trip + the
targeted tests + the whole-floor Summary) · STOP triggers hit or "none" · did the peer round-trip fully, or surface a
4th (runtime) gap · anything that surprised you.

## Prior comparable: S3a (`93e936b3`) + S3-Nature-2 (`23e8c16f`) — the two prior gaps in this same chain; each the
"first full-args-parametric-extend-type consumer walks an untested corner" (`PRIMVS VSVS ANGVLOS PANDIT`).
