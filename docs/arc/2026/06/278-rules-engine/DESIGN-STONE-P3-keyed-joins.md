# DESIGN — Stone P3: keyed hash-joins (the O(N²)→O(match) bend)

Where the curve turns. The native hash-join (P2) still cross-products every token × every element
(`kernel.rs:573-589`) — O(tokens·elements). P3 keys the join: group the right elements by their shared-var
values, then probe with each left token's shared-var values, touching ONLY the matching bucket. The real RETE
hash join; closes 3b's deferred join-bindings index. **Behavior-preserving** — same derived facts — so the P2
differential is the correctness net; the win shows on the bench.

## This is a PERF stone — verification is shaped accordingly
P3 changes NO observable behavior (same joined tokens → same derived facts). So:
- **Correctness net (asserted, CI):** the P2 differential probe (`probe_arc278_P2_native_fire_once`) stays
  4/4 — native `fire-once'` == wat `fire-once`. If P3 changes a conclusion, it goes red.
- **The win (measured, on-demand):** `tests/perf_arc278_fire_baseline.rs::native_fire_once_join_scaling`
  (N distinct locs → N joins of N×N candidate pairs). P2/cross baseline (captured at HEAD):
  `19.3 → 24.0 → 29.9 → 37.7 → 62.3 us/fact` for N = 100…1600 (199ms at N=1600) — climbing (the O(N²) creep).
  P3/keyed target: us/fact stays ~flat (~constant) — the join is now O(N). Verified by re-running the bench +
  reading the keyed-join diff (no flaky CI timing assertion).

## What P3 changes (Rust — `src/rete/kernel/fire/mod.rs`, the hash-join cross only)
Replace the nested cross in `hash_join_pass` with a keyed probe. For each HashJoinNode child
J with LEFT tokens + RIGHT elements:
1. **Join key = the shared var names** = (a token's binding keys) ∩ (an element's binding keys), computed once
   per (node, child) — all tokens at a beta node share a key set; all elements at an alpha share a key set, so
   the intersection is fixed. Use a **sorted** `Vec<Value>` of the shared key names (canonical order so token
   and element key-tuples line up).
2. **Index the RIGHT:** `HashMap<Vec<Value>, Vec<usize>>` (or `Vec<&Element>`) mapping each element's
   join-key-value tuple → the elements with that tuple. (`Value` is `Hash + Eq` — confirm; if not, hash a
   stable rendering.) O(elements).
3. **Probe with the LEFT:** for each token, compute its join-key-value tuple, look up the bucket → extend the
   token with ONLY those elements. O(tokens + matches).
4. **The bucket IS the compatible set.** `token_element_compatible?` requires agreement on ALL shared keys;
   keying on all shared keys means the matching bucket = exactly the compatible elements → call `extend_token`
   directly, no per-pair compat re-check needed. (Keep `token_element_compatible` available, but the keyed path
   doesn't call it in the inner loop.)

Edge: **empty shared keys** (a join with no shared var) → every element in one bucket (key = `vec![]`), every
token probes it → effectively the cross (correct for a cartesian join; rare). **Empty left/right** → skip
(unchanged). Everything else in the kernel (alpha/root-join/production passes, extend_token, alpha-feeding)
UNCHANGED. The `WorkingMemory` stays flat (`HashMap<i64, Vec<Value>>`) — the index is a transient built per
node during the pass, not a change to the stored memory shape (keying the *stored* memory for incremental
delta is P4's concern).

## The one contract decision (pinned)
The keyed join is observationally identical to the cross (same compatible pairs → same extended tokens → same
derived facts); the only change is complexity (O(N²) → O(N)). The P2 differential is the proof of identity;
the bench is the proof of the bend.

## Files touched
- `src/rete/kernel/fire/mod.rs` — rewrite `hash_join_pass`'s cross as a keyed index+probe. Nothing else.
- (bench already extended: `tests/perf_arc278_fire_baseline.rs::native_fire_once_join_scaling`.)

## Verify
- `probe_arc278_P2_native_fire_once -- --include-ignored` → **4/4** (differential preserved — the canary).
- The full chain + floors at baseline (no regression).
- `perf_arc278_fire_baseline native_fire_once_join_scaling -- --ignored --nocapture` → us/fact stays ~flat
  across N (vs the captured climbing P2 baseline) — the bend.

## Out of scope = REJECTED
- **Keying the STORED memory** (`node-id → {join-key → [...]}`) for incremental insert — P4 (delta). P3's
  index is transient per-pass.
- **Delta / fixpoint / retraction / public `fire` / Clara bench** — P4/P5.
- No change to alpha/root-join/production passes, the oracle, or observable behavior.
