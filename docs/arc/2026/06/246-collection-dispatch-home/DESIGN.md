# Arc 246 — collection-dispatch home (`src/collection/`) — DESIGN

**Opened 2026-06-04**, the first forward-arc after 237's death. Graduates `STUB.md`. The home where the clause-vs-intrinsic doctrine (just inscribed in comments by 237.8d) lives permanently in self-verifying, re-castable code. Builder: *"I want this collection dispatch to be a warded namespace — I never want to deal with these thoughts again."*

## Mission

Lift the **container-polymorphic collection intrinsic dispatch** out of the flat `src/check.rs` + `src/runtime.rs` into a warded `src/collection/` home, redirect the central routing arms into it, ward it to L1+L2=0 (vigilatum stamp), and **inscribe the partition doctrine in the home's module-doc** so the "why isn't this a clause?" question is answered structurally, forever. Doctrine home: `docs/OP-PLACEMENT.md` (the prose); this arc is the **code** that proves it.

## The name — `src/collection/` (intueri-cast, 2026-06-04)

A spawned `intueri` cast settled the name (verdict grounded on the live tree + `docs/OP-PLACEMENT.md`):

- **`src/collection/`** — names the DOMAIN; matches the noun-by-domain precedent (`function/`, `check/`, `types/`, `remedy/`, `comms/`, `rust_deps/`); makes no promise it can't keep.
- **`src/dispatch/` REJECTED** — it would **lie**: it names itself after `dispatch_keyword_head`/`_value` (runtime.rs ~5295/5318, the *central* keyword dispatch that STAYS OUTSIDE this home and merely redirects in). A home named for the one mechanism it does not contain is an active cold-read lie — the worst Level-1 failure. Its only honest place is *demoted to a child module* under `collection/`, where the parent noun kills the collision.
- The scope tension (does the broad noun pull in all collection ops?) resolves **one level down**, in the submodule + the doctrine word `intrinsic` — not in the directory name.

## Scope — NARROW: the container-polymorphic dispatch core (grounded on disk 2026-06-04)

**IN** — the ops that dispatch over {Vector, HashMap, HashSet, List} and need type-level computation (the projective intrinsics):

- **Check-side — 4 inference intrinsics** (`src/check.rs`): `infer_contains` (10371), `infer_conj` (10451), `infer_get` (10538), `infer_assoc` (12203). *(No separate `infer_length`/`infer_empty` exist — inferred inline; confirm whether their inline arms move or stay at lift.)*
- **Runtime-side — ~30 per-Type impls** (`src/runtime.rs`): `eval_<vector|hashmap|hashset|list>_<length|empty_q|contains_q|get|conj|assoc|dissoc|keys|values|concat>` + the constructors `eval_vector_ctor` / `eval_hashmap_ctor` / `eval_hashset_ctor`.

**STAYS PUT, redirects** — the **110** `:wat::core::(Vector|HashMap|HashSet|List)/<op>` routing arms in `dispatch_keyword_head_value`. The central dispatch is NOT this home (its own future home is the 109-level `runtime.rs` reorg). Each arm becomes a mechanical `collection::eval_*` call.

**OUT** — explicitly excluded, to keep the home the *dispatch* core:
- **`dispatch_rust_scheme`** (check.rs 12900) — the `:rust::` shim dispatch; a different concern (rust-deps), not collections.
- **~12 Vector/List-specific utility ops** — the seq-HOFs (`eval_vec_map`/`filter`/`foldl`/`foldr`/`sort_by`, `eval_list_map_with_index`) and helpers (`eval_vec_reverse`/`range`/`take`/`drop`/`last`/`rest`/`find_last_index`, `eval_list_zip`/`window`/`remove_at`). They are collection ops but NOT container-polymorphic dispatch. If they ever want a home it is a **sibling** module (`collection/transform.rs`), a separate decision — NOT this arc.

## Internal layout — intueri-cast confirmed (2026-06-04)

A spawned `intueri` cast named the submodules, grounded on the **`src/function/` precedent** (the structural twin: same lift origin `runtime.rs`+`check.rs`, same check-side/runtime-side shape, same words `function/` already chose — `eval.rs` + `infer.rs`):

```
src/collection/
  mod.rs        — home root; the module-doc inscribes the clause-vs-intrinsic partition
                  doctrine (the word "intrinsic" lives HERE, in prose — `get` as the
                  worked proof, cites docs/OP-PLACEMENT.md); the vigilatum stamp; any shared
                  helper (cf. `function/mod.rs`'s `FN_HEAD`).
  infer.rs      — the 4 check-side inference intrinsics (CheckEnv/InferCtx/Subst/TypeExpr).
  eval.rs       — the ~30 runtime per-Type impls + 3 constructors (Value/Environment/…).
  transform.rs  — CONDITIONAL (only if the ~12 utilities are swept — see Scope): the
                  seq-HOFs + helpers. (`sequence.rs` the alternative if accessors dominate;
                  weigh at the sweep.)
```

**Why by-side, not a single `intrinsic.rs` (cast verdict):** the two sides have disjoint imports (check vs runtime types), so a by-side split keeps each file's import-world cohesive (intueri's "too many import-worlds = a mumbling file"); and `intrinsic` is the *doctrine* word — it belongs in the `mod.rs` prose where the OP-PLACEMENT.md doctrine lives, NOT a filename (a filename must show the home's shape on `ls`; `infer`/`eval` do, `intrinsic.rs` hides it). Mirrors `function/`'s `{eval,infer,parse,metadata}.rs`. A shared helper, if one emerges, lands in `mod.rs` (not a third mumbling file).

## Difficulty — moderate, bounded (a homes-walk sibling)

~34 standalone fns over one cohesive domain + ~110 mechanical redirects. The fns are already standalone (cut/paste-liftable). Comparable scale to the `function/` and `check/` lifts already walked. **Not a `runtime.rs` excavation — a known, bounded walk.** Substrate-as-teacher cascade to green after the lift; the compiler names every redirect.

## Slicing

1. **246.0 — DESIGN** (this doc). Name cast ✓, scope verified ✓, fn set grounded ✓. *Done when this commits.*
2. **246.1 — LIFT + REDIRECT.** Create `src/collection/`; move the 4 `infer_*` + ~30 `eval_*` in; `pub(crate)` as needed; redirect the 110 central-match arms to `collection::*`; cascade to green (lib 895/0/1, build clean). FM-2-bis probe: a disconfirming contract that the collection ops still dispatch correctly post-lift + the home module exists.
3. **246.2 — WARD** (vigilia 8-spell → L1+L2=0). Annihilate the failure classes; earn the `vigilatum` stamp. (Clippy-in-home = L2 — the home is held to the warded bar the flat files are not.)
4. **246.3 — INSCRIBE + INSCRIPTION.** The module-doc states the discriminant with `get` as the worked proof, citing `docs/OP-PLACEMENT.md`; the home *answers* the question structurally. INSCRIPTION closes the arc.

## Gates (per stone)

- Lib `cargo test --release --lib -p wat` → 895/0/1 (no regression).
- `cargo build --release --tests --workspace` → clean.
- Ward stones: vigilia L1+L2=0; clippy-clean *in the home*.
- No `holon-rs`. No touching the central `dispatch_keyword_head` beyond the redirect arms.

## Enabled-by / blocks

Enabled by 237's closure (done). Independent of arc 245 (disjoint: Rust `src/` vs `wat/` corpus). Does not block 245. After 246 + 245, wind back to whoever 237 blocked.
