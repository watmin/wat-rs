# DESIGN — STONE 255.1c-guard: hoist the registry above the literal arms, and finally read the hot-path number

## The defect, proven by differential (not argued)

`runtime.rs`'s dispatch is a `match` on the head string. The registry is consulted by a **guard arm
at `:5608`**, sitting partway down the table. Rust matches top-to-bottom, so **any literal arm above
`:5608` wins over registration.**

Proven this session by construction — one literal arm inserted at `:5036` for an already-registered
name, nothing else changed:

```
baseline (registry path)      →  "ff"
+ literal arm before guard    →  "SHADOWED"
restored                      →  "ff"
```

**Measured spread:** of **540** dispatch arms, **373 sit BEFORE the guard** and only **167 after**.

| family | arms before the guard |
|---|---|
| `core` | **231** |
| `runtime` | **13 (all of them)** |
| `std` | 4 · `edn` 4 · `kernel` 3 · `config` 2 |

`:wat::time::` — home #2 — was the **anomaly**: all 41 arms sat *after* the guard. Every family that
remains is upstream of it.

## Why this blocks a multi-day carve

The position silently changes what a missed deletion means:

| arm position | rider registers a name but misses deleting its arm |
|---|---|
| **after** the guard (`time` only) | guard wins → new path runs → stale arm is dead code → **clippy catches it** |
| **before** the guard (**everything left**) | **old arm wins → the new handler NEVER RUNS.** `metadata-of` answers, reflection works, the floor is green, dispatch is unchanged |

The second row is **a carve that looks finished and did nothing**, and it is the shape of all 373
remaining arms. Today the only thing standing between us and that is a per-brief gate I have to
write correctly ~30 more times — and home #2's version of that gate was imprecise on its first
draft. **This stone removes the class instead of re-catching the case** (extirpare: climb from
"a check that fires" to "the wrong state has no form").

## The one contract decision, pinned

**The registry is consulted BEFORE the match is entered — not as its first arm.**

```rust
// at the top of dispatch_keyword_head_value, before `match head {`
if let Some(handler) = crate::intrinsic::registry().lookup(head) {
    return handler(args, list_span, env, sym);
}
match head { … }          // the literal table, unchanged
```

A first-*arm* guard would still be inside the match's ordering and would read as one-arm-among-many;
a pre-match check states the law plainly — **registered wins, always** — and cannot be reordered by
a later edit that inserts an arm above it.

## ★ This stone is OBSERVATIONALLY INERT — which is what makes the number readable

Measured: of the 6 registered production names (`Bytes::to-hex`, `Bytes::from-hex`, `show-source`,
`render-doc`, and the special forms), **none has a literal dispatch arm.** So hoisting changes no
behaviour today — every currently-registered name already reaches the registry, and every unregistered
name still falls to the same literal table.

**The entire deliverable is therefore ONE NUMBER: what the hoist costs the arithmetic hot path.**
Behaviour is fixed; only the cost moves. That is the cleanest possible reading of the design's
long-deferred gate.

## The perf question, stated honestly

`IntrinsicRegistry` is `HashMap<&'static str, IntrinsicEntry>` (`intrinsic/mod.rs:353`) — **std
SipHash**, so a lookup hashes the whole head string. Today `:wat::core::i64::+` dispatches at
`:5036`, **above** the guard, and has therefore **never paid a registry lookup**. After the hoist it
pays one on every single call.

The design gates this arc on *"no regression on the arithmetic hot path"* (`DESIGN.md` § Perf) and
that gate has never been measured, because **no bench exists**: no `benches/`, no `[[bench]]`, no
criterion. `wat-scripts/perf/` holds wat-level harnesses (grid, matrix, clara) whose per-iteration
cost is interpreter-dominated.

**The stone therefore builds the instrument first and captures the baseline BEFORE the hoist.** A
delta measured against a number taken after the change is not a delta.

### The instrument

An in-crate `#[test] #[ignore]` timing harness that drives `dispatch_keyword_head_value` directly in
a loop and reports ns/op. Rationale, each a rejection of a worse option:

- **In-crate**, because the fn is private — no visibility widened for a measurement.
- **`#[ignore]`**, so it never runs in the floor and never becomes a flaky timing test. Run
  explicitly for the number. This matches the repo's existing manual-perf convention.
- **No criterion**, no new dev-dependency, for a two-point comparison.
- **Direct dispatch, not a wat program**, because a wat loop's per-iteration cost is microseconds of
  interpreter against nanoseconds of dispatch — the signal would drown. Naming this in advance
  because it is the trap door that would make the number meaningless.

## Out of scope — affirmatively cut, not deferred

- **No family is carved.** Not `kernel`, not `core`, not anything. This stone deletes zero arms.
- **The blanket-accept** (`resolve/walk.rs:257`) is untouched — `255.1b-iv`.
- **No hash-function or `phf` change.** If the number says the lookup costs too much, the response is
  a ruling by the builder on a measurement, not a fallback this design pre-authorises. The stone's
  deliverable is the number; what to do about it is not this stone's decision to make.

## What lands after it

With registration structurally winning, every remaining family becomes carveable in any order and a
missed deletion becomes dead code that clippy names, exactly as it did for `time`. The ordering
hazard stops being a per-brief hazard.

The next home after that is **`:wat::kernel::`** on its own evidence: every registered row today is
**48 `Pure` / 2 `Preserving` / 0 `Effectful`**, so the purity axis is precisely where determinism was
before `time` — a set that cannot falsify the contract (R59). `kernel` is the IO tier, and
`pure_declared_matches_is_effectful_op` (`intrinsic/mod.rs:601`) is the cross-check that has never
yet seen an effectful row.
