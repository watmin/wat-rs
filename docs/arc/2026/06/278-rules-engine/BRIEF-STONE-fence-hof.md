# BRIEF — fence-HOF: the 6a purity fence handles higher-order fold fns + fn-literals

**Single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `cargo wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)
The 6a fence (`src/rete/purity.rs`) rejects every higher-order fold fn — `foldl`/`foldr`/`map`/`filter`/
`reduce` aren't recognized, and a `:wat::core::fn` literal is treated as an unknown call head. Teach the fence
these, with **conditional purity** (a HOF is pure∧det iff its fn-arg is — which falls out of the existing
arg-recursion). `src/rete/purity.rs` ONLY. Contract: `DESIGN-STONE-fence-hof.md`.

## Read in order (the rooms — all in `src/rete/purity.rs`)
1. `DESIGN-STONE-fence-hof.md` — the contract (conditional purity, NOT blanket-allow).
2. **`intrinsic_meta` (`:66`–`:163`)** — the `pure_det` `matches!` set (`:76`). Add `:wat::core::foldl`,
   `:wat::core::foldr`, `:wat::core::map`, `:wat::core::filter`, `:wat::core::reduce`.
3. **`head_ok` (`:169`) + `classify_fn` (`:265`)** — note `head_ok:171` checks `sym.functions.contains_key`
   FIRST, so a native fn registered in `sym.functions` reaches `classify_fn` before `intrinsic_meta`. `foldl`
   is exactly that (native, registered). So change `classify_fn`'s `FunctionBody::Native => false` (`:279`) to
   **consult `intrinsic_meta`**: `FunctionBody::Native => intrinsic_meta(fqdn).is_some_and(|m| match axis {
   Axis::Pure => m.pure, Axis::Deterministic => m.deterministic })`. (This is load-bearing — without it, step 2
   is never reached for foldl.)
4. **`classify_expr` (`:192`–`:261`)** — add a `:wat::core::fn` lambda arm (before the general-list arm at
   `:243`): for `(:wat::core::fn [params] -> :ret body…)`, classify the **body** forms (after the `-> :ret`),
   skipping the param vector + ret-type. Mirror the `match`-arm's `->`-locating logic (`:224`–`:239`) to find
   the body. A fn-literal with no `->` (untyped) → classify everything after the param vector, OR deny if
   malformed — match the project's fn-literal grammar (read how `:wat::core::fn` parses).
5. `tests/probe_arc278_fence_hof.rs` — the contract, RED now (4 tests: pure fold/map pure∧det + the impure
   guard). Do NOT weaken it; the impure-guard MUST stay (conditional purity, not blanket-allow).

## STOP triggers
1. If marking the HOFs pure∧det does NOT make `pure?` accept the pure fold (e.g. the fn-arg recursion doesn't
   reach the fn-literal body) — STOP, report what the classify path does for `(foldl (fn …) 0 xs)`.
2. If the impure-fold guard test goes green-then-the-impure-one-also-passes (blanket-allow leak) — STOP, you've
   over-allowed; the fn-arg's impurity must propagate.
3. If greening needs anything beyond `src/rete/purity.rs` — STOP.

## Done = green
`cargo test --release -p wat --test probe_arc278_fence_hof` → 4/4. AND no regressions: `--test
probe_arc278_6b_ii_a_where_oracle` (where-fence intact) → its existing count ; `cargo build --release` clean +
`cargo test --release -p wat --lib -- --test-threads=1 | grep result` → 941/36. (8-custom's probe stays RED —
expected; it greens in the next stone.)

## Report back
The exact `purity.rs` diff (the 3 parts), the test counts (verbatim), and any STOP. Final message is all I see.
