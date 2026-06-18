# BRIEF — Stone 3b: `HashJoinNode` (the two-sided equality join — THE HEART)

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A PURE WAT stone
(`wat/rete.wat`) — grow `fire-rules` with the join pass. This is the hardest node; get the join semantics
EXACTLY right. Build, run named tests, report verbatim. Another agent weighs.

## The work
Grow `fire-rules` with a JOIN PASS that runs AFTER root-join seeding (3a): propagate Tokens left→right through
HashJoinNodes, crossing each upstream Token against the Elements of the join's condition-alpha, unifying
compatible pairs into extended Tokens stored in the HashJoinNode's beta-memory. Multi-condition rules now
match end to end. NO production firing / NO cascade (stone 4).

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-3b-hash-join.md` — the join semantics (THE CRUX), the
   traversal, the reverse-lookup of the right-alpha, the dynamic join key, what's deferred (the index +
   binding-keys precompute), the hazard.
2. `docs/arc/2026/06/278-rules-engine/CLARA-REFERENCE.md` §2 — Token extends at each join
   `(->Token (conj matches [fact id]) (conj bindings fact-bindings))`; two memories keyed by join-bindings;
   the left/right cross.
3. `wat/rete.wat` — the EXISTING `fire-rules` + `root-join-pass`/`seed-root-join-children`/`append-token` (3a)
   you extend; `Token`/`Element`/`AlphaNode`/`HashJoinNode` records + accessors; `node-kind-label`; the
   `match (Some pv) (None …)` idiom (`append-token`, :561) + the foldl-over-keys idiom.
4. `tests/probe_arc278_3b_hash_join.rs` — remove the 3 `#[ignore]`s. It is your contract.

## The join semantics — implement EXACTLY (the hazard zone)
For a `HashJoinNode J`:
- **RIGHT** = `alpha-memory[alpha-feeding(J)]` where `alpha-feeding(J)` = the AlphaNode whose `children`
  contains J (reverse-lookup over the network — the real edge 1b wired).
- **LEFT** = `beta-memory[B]` where B is the upstream beta node reached by forward traversal (B's `children`
  contains J; you arrive at J from B).
- for each `(token, element)` in LEFT × RIGHT:
  - **compatible?** fold `element.bindings`: for each `(k, v)`, if `token.bindings` has `k` with a value `≠ v`
    → INCOMPATIBLE. (A var in only one side never conflicts.) All agree → compatible. This dynamic shared-var
    agreement IS the equality join (do NOT use `binding-keys` — it's empty; dynamic is correct).
  - **extend** (compatible only): new Token =
    - `matches`  = `(:wat::core::PersistentVector/conj token.matches (:wat::core::Tuple element.fact <alpha-id>))`
    - `bindings` = fold `element.bindings` into `token.bindings` (`PersistentMap/assoc` each entry)
    - append to `beta-memory[J]`.
- propagate: J's new tokens flow to J's children → repeat to a monotone fixpoint (a stable-iteration loop, or a
  topological pass over the join chain). For the v1 (linear per-rule chains) one ordered pass after root-join
  suffices; if you need a general fixpoint loop, write it (it terminates — finite, monotone).

⚠ **HAZARD #1**: LEFT=tokens, RIGHT=elements; compatibility = shared-var AGREEMENT (not "all keys present", not
"any key present"); a var on only one side is FINE. Mis-cross or mis-key → silent drops/dups. The probe's
no-match case is the canary.

## Builder directive: build missing deps, never hack around
Deps SHOULD all exist (fold, PersistentMap/keys/get/assoc/contains, PersistentVector/conj, Tuple, Token/Element
accessors, node-kind-label, `=`). **If a core primitive is genuinely missing → STOP + name it.** Do NOT hack.

## Engine-source bar (DOGFOOD)
LINT-CLEAN — `format`/`interpolate`, `cond`/`contains?` not nested-`if`. ONLY below-bar spot = the EXISTING
`render-dag` fixture — do NOT touch it.

## STOP triggers
1. A needed core primitive is missing → STOP, name it.
2. The traversal ordering is non-trivial beyond a single chain / a simple fixpoint (e.g. you can't determine a
   correct order) → STOP, describe what you found (do not guess an order that might drop matches).
3. You reach for production firing / cascade / the join-bindings index → that's stone 4 / deferred; STOP.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored        # 3/3 GREEN (2 match + 1 no-match guard)
cargo test --release -p wat --test probe_arc278_3a_root_join -- --include-ignored         # 3/3 (root-join still green)
cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored       # 3/3
cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored          # 1/1
cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored             # 2/2
cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored           # 3/3
cargo test --release --test test_stdlib_load_order | grep result                            # 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                                 # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                  # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                         # clean
```
Report: the join-pass source + helpers (`alpha-feeding`, compatibility, extend, the cross + propagation), all
outputs verbatim, any STOP hit. Un-ignore the 3 probe tests. No git.

## Blast radius
`wat/rete.wat` (`fire-rules` grows a join pass + helpers) + the probe (un-ignore). NO 1a/1b record change. NO
Rust unless a sub-dep is missing → STOP. No git.
