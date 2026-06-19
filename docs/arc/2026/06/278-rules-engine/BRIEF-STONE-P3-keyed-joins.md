# BRIEF — Stone P3: keyed hash-joins (the O(N²)→O(match) bend)

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A RUST
stone, **one function**: rewrite the hash-join cross in `src/rete/kernel.rs::hash_join_pass`. Build, run the
named tests, report verbatim. Another agent weighs.

## The work
The native hash-join (P2) cross-products every left token × every right element — O(tokens·elements). Replace
that nested cross (`src/rete/kernel.rs:573-589`) with a **keyed index + probe**: group the right elements by
their shared-variable values once, then probe with each left token's shared-variable values, touching ONLY the
matching bucket. This is the real RETE hash join. It is **behavior-preserving** — the SAME compatible pairs →
the SAME extended tokens → the SAME derived facts. The only change is complexity (O(N²) → O(N)). Nothing else
in the kernel changes.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P3-keyed-joins.md` — the keyed-join algorithm, the join-key
   definition, the empty-shared-keys edge, the contract (observationally identical to the cross), out-of-scope.
2. `src/rete/kernel.rs:539-592` — `hash_join_pass` (what you rewrite — the cross is lines 573-589) and its
   helpers immediately above it: `alpha_feeding` (`:478`), `token_element_compatible` (`:504`), `extend_token`
   (`:523`). Also `element_fact_bindings` (`:327`) and `token_matches_bindings` (`:342`) — the destructurers
   you already call.
3. `wat/rete.wat:657-728` — the oracle's `token-element-compatible?` + `extend-token` + `cross-join-node`. The
   keyed path must produce IDENTICAL results to this cross (read it to confirm the compat semantics: shared-var
   AGREEMENT — a var present on only ONE side never conflicts). **Do NOT change the oracle.**
4. `tests/probe_arc278_P2_native_fire_once.rs` — the differential net (already live, currently 4/4 GREEN). This
   is the correctness gate; it must STAY 4/4. **Do not modify it.**

## The algorithm (rewrite of lines 573-589 ONLY)
Inside the existing `for child_id { ... }` loop, after `let elements = ...` (the right-side elements) and with
`tokens` (the left side) in hand, replace the nested `for tok { for el { if compatible { extend } } }` with:

1. **Compute the join key = the sorted shared variable names.** The shared vars = (the keys present in the
   tokens' bindings) ∩ (the keys present in the elements' bindings). All tokens at a beta node share a binding
   key-set and all elements at an alpha share one, so this intersection is fixed for the (node, child) pair —
   compute it ONCE before the probe, not per pair. Use a **sorted `Vec<Value>`** of the shared key names
   (canonical order, so a token's key-tuple and an element's key-tuple line up positionally). Derive it from
   one sample token's bindings ∩ one sample element's bindings (guard the empty-tokens / empty-elements cases —
   they already `continue` above, so you have ≥1 of each here).
2. **Index the RIGHT (elements) by join-key-value tuple.** Build a `HashMap<Vec<Value>, Vec<usize>>` (or
   `Vec<&Value>`) mapping each element's `[bindings.get(k) for k in join_keys]` tuple → the elements with that
   tuple. (`Value: Hash + Eq` holds — it is already an rpds map key in this file: `m.get(&Value::i64(..))`,
   `pm.insert(Value::i64(..), ..)`.) O(elements).
3. **Probe with the LEFT (tokens).** For each token, compute its `[bindings.get(k) for k in join_keys]` tuple,
   look up the bucket, and `extend_token` with ONLY those elements. O(tokens + matches).
4. **The bucket IS the compatible set — no per-pair re-check.** Keying on ALL shared vars means the matching
   bucket = exactly the elements that agree on every shared var = exactly what `token_element_compatible`
   returns true for. So call `extend_token` directly inside the bucket loop; do NOT call
   `token_element_compatible` in the inner loop. (Leave the `token_element_compatible` fn in place — it is still
   referenced conceptually / may be used elsewhere; do not delete it.)

**Edge — empty shared keys** (`join_keys` is empty, e.g. a cartesian join with no shared var): every element's
key-tuple is `vec![]` → one bucket; every token's key-tuple is `vec![]` → probes that one bucket → effectively
the full cross. That is the CORRECT behavior for a no-shared-var join (rare). Handle it naturally — an empty
`join_keys` vec falls out of the same code (every tuple is `[]`), no special branch needed.

A token must produce the same extended tokens, in an order that yields the same final derived-fact SET. The
differential compares derived facts (query count + content), so per-bucket iteration order is fine as long as
every compatible element is hit exactly once per token.

## Builder directive: build missing deps, never hack around
Everything you need exists (`extend_token`, the destructurers, `std::collections::HashMap`, `Value: Hash+Eq`).
**If something is genuinely missing → STOP + name it.** Do NOT change the wat oracle. Do NOT touch any other
pass (alpha / root-join / production). Do NOT change `WorkingMemory`'s shape (the index is a transient built
per (node,child) during the pass, NOT a change to the stored memory — keying the STORED memory is P4).

## STOP triggers
1. The keyed result diverges from the cross (P2 differential goes red) and the only fix you see touches the wat
   ORACLE or another pass → STOP, describe the divergence (which token/element, which shared var). The oracle is
   the reference; the native impl conforms to it.
2. You reach for: keying the STORED memory, a fixpoint, delta/incremental insert, retraction, the public
   `fire`, or a Clara bench → that is P4/P5; STOP.
3. The join-key intersection is ambiguous (tokens/elements at the same node carry DIFFERENT key-sets) → STOP and
   describe; do not guess a tie-break.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored   # 4/4 GREEN — the correctness net (MUST stay 4/4: 2×2 == exactly 2, no cross-loc leakage)
cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored           # 4/4 (the wat oracle join still green — you didn't touch it)
cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored      # 4/4
cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 | grep result  # 1/1
cargo test --release -p wat --lib rete 2>&1 | grep "test result"                            # kernel/matcher unit tests green
cargo test --release -p wat --lib 2>&1 | grep "test result"                                 # 935/36 (the 36 pre-existing UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                  # 264/1 (UNCHANGED)
cargo test --release --test test_stdlib_load_order | grep result                            # 1/0
cargo build --release 2>&1 | tail -2                                                         # Finished; no NEW warnings
```
Optional (perf, NOT a gate — run if quick): the join-scaling bench shows the bend.
```
cargo test --release -p wat --test perf_arc278_fire_baseline native_fire_once_join_scaling -- --ignored --nocapture
# P2/cross baseline (climbing): us/fact 19.3 → 24.0 → 29.9 → 37.7 → 62.3 for N = 100..1600
# P3/keyed target: us/fact stays ~flat. If it still climbs steeply, the keying didn't take — report it.
```
Report: the rewritten `hash_join_pass` (full fn body); all test outputs verbatim; the bench numbers if you ran
them; any STOP hit. No git.

## Blast radius
`src/rete/kernel.rs` — `hash_join_pass` only (the cross at lines 573-589 → keyed index+probe; the surrounding
node/child loop structure stays). NO other pass. NO wat changes. NO oracle change. NO `WorkingMemory` shape
change. No new public surface. No git.
