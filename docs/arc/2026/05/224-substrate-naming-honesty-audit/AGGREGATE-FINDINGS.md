# Stone 224.4 — Aggregate Findings + Fix-Arc Plan

**Date:** 2026-05-23 morning
**Author:** orchestrator (claude-opus-4-7), orchestrator-direct per `feedback_sonnet_no_realization_voice`
**Inputs:** FINDINGS-INTUERI-HOLON-AST.md (224.1) + FINDINGS-INTUERI-RUNTIME.md (224.2) + FINDINGS-INTUERI-CHECK.md (224.3)

## The layered honesty pattern (the load-bearing observation)

Three casts surfaced a clean layering:

| Layer | File | L1 lies | L2 mumbles | Spark verdict |
|---|---|---|---|---|
| **Algebra primitives** | `holon-rs/src/kernel/holon_ast.rs` | **0** | 4 | lives — substrate is honest |
| **Verb dispatchers** | `wat-rs/src/runtime.rs` (28,916 lines) | **3** | 8 | mumbles + family pattern |
| **Type checker** | `wat-rs/src/check.rs` (18,361 lines) | **3** | 7 | dims at pre-arc-109 legacy boundary |

**The substrate algebra IS honest** — the 16 HolonAST variants speak truth. **The wat-rs surface above lies** — verb names borrow variant names + overload polymorphically; type-checker docs went stale when arc 109 K.kernel-channel renamed vocabulary; closure-local names mislabel types. The fixes are at the surface layer, not the algebra.

The doctrine-dialogue prediction held: `:wat::holon::Atom` IS the worked example of a Level 1 lie. Intueri found two more in runtime.rs + three more in check.rs.

## All 6 Level 1 lies — categorized for fix-arc planning

### Group A — Small in-arc fixes (Stone 224.5 candidate)

Each is bite-sized (single function + adjacent doc, OR doc-only) and bundles into ONE sonnet stone (~60-120 min).

| # | Site | Fix | Estimated touch |
|---|---|---|---|
| L1-runtime-2 | `runtime.rs:1105-1106` (`Value::type_name()` Sender/Receiver) | Return `"wat::kernel::Sender"` / `"wat::kernel::Receiver"`; update 5 `expected:` string call sites at lines 18160, 18252, 18320, 18406, 18821 | 7 sites (2 + 5) |
| L1-runtime-3 | `runtime.rs:13605-13610` (`holon_item_to_value` error op) | Thread `op: &str` through helper signature; update all callers to pass their own op name | 1 fn signature + ~3-5 call sites |
| L1-check-A | `check.rs:3675-3699` (`type_contains_sender_kind` doc) | Rewrite doc to canonical names; rename function to `sender_kind_in_type` (Rust `find_` convention for Option-returning search) | 1 fn doc + adjacent rename refs |
| L1-check-B | `check.rs:143` (`ScopeDeadlock` variant doc) | Drop QueueSender/QueuePair refs; use canonical `Sender`/`Channel`/`HandlePool` | doc-only |
| L1-check-C | `check.rs:15624` (`symbol_ty` closure → keyword type) | Rename closure to `keyword_ty`; update 4 TypeScheme citation lines 15633/15642/15651/15700 + adjacent comment | 1 closure + 4 citation sites + 1 comment |

**Calibration:** Group A fits one sonnet stone, 60-120 min Mode A. Mechanical; cascade test risk low (no functional behavior change except L1-runtime-2 which improves user error messages).

### Group B — Substrate-wide rename (Arc 225 spawn child of arc 224)

| # | Site | Fix | Estimated touch |
|---|---|---|---|
| L1-runtime-1 + family pattern | `runtime.rs:13820` (`:wat::holon::Atom` verb) + `eval_atom_value` (`:wat::core::atom-value` inverse) | Rename verb-pair to `:wat::holon::atomize` / `:wat::holon::materialize`; update `eval_*` Rust function names; update all wat caller sites; update doc comments; update interop/integration tests; update wat-edn references if any; cross-language interop implications | 50-100+ sites across wat-rs + holon-rs + lab + tests + docs |

**Calibration:** Substrate-wide consumer sweep. Per arc 159's precedent (~951 sites for `let*` retirement) this is medium-sized. Expected ~3-5 hours sonnet work across 2-3 stones. Deserves its own arc with INSCRIPTION cycle.

**Arc 225 spawns from arc 224.** Per `feedback_spawn_block_winding`: the audit (arc 224) cannot honestly close until the fix-arc (arc 225) closes. Arc 224 INSCRIPTION (Stone 224.7) blocked on arc 225.

## Level 2 mumbles — summary (not exhaustive enumeration)

Across all three casts, 19 L2 mumbles found. Pattern categories:

**Stale-vocabulary docs** (5 mumbles) — arc 109 K.kernel-channel + arc 220 List<T> minting + arc 162 lambda→fn rename all left adjacent doc stragglers. Mechanical fix; could fold into Group A OR defer to a general doc-refresh stone.

**Naming-convention drifts** (~7 mumbles) — `eval_list_*` operating on Vector (5 fns), `value_to_holon` vs `value_to_atom`, `eval_form_step` inversion, `dispatchs` count noun. Cosmetic; defer or bundle.

**Helper-function asymmetries** (4 mumbles) — `require_vec` vs `require_vector`, `type_is_thread_kind` (bool) vs `type_contains_sender_kind` (Option), `eval_config_noise_floor_default_shim`, `infer_list_constructor`. Cosmetic.

**Missing intueri runes** (3 mumbles) — 4 `#[allow(dead_code)]` retired walkers in check.rs lack proper runes.

**Doc cleanup** (rest) — duplicate fragment on Value::wat__std__HashMap doc, three-block absence-doc in check_program, arc 117 dead-code section header.

**Disposition:** L2 mumbles can be addressed:
- **Bundled into Group A stone** if cheap to do alongside the L1 fixes (especially the stale-vocabulary ones, since they're in the same files being edited)
- **Deferred to a future maintenance arc** for the cosmetic / convention-drift ones
- **Skipped** if the cost exceeds the readability benefit

The aggregate recommendation: include stale-vocabulary mumbles in Group A (Stone 224.5) since they're in-file; defer the cosmetic ones explicitly with affirmative-out-of-scope language per `feedback_no_known_defect_left_unfixed` (i.e., NOT "deferred to future arc when X surfaces" but "out of scope for arc 224; tracked as low-priority polish if a substrate-maintenance arc opens").

## Fix-arc plan

### Pivot sequence (spawn-block honest)

```
Stone 224.4 — aggregate (THIS DOC; orchestrator-direct)  ✓ SHIPPED HERE
Stone 224.5 — Group A small L1 fixes (one sonnet stone)
   ├─ runtime.rs: 2 fixes (type_name + holon_item_to_value op)
   ├─ check.rs: 3 fixes (sender_kind doc, ScopeDeadlock doc, symbol_ty rename)
   └─ Stale-vocabulary L2 mumbles bundled
Arc 225 spawned — atomize/materialize substrate-wide rename
   └─ Per spawn-block discipline: arc 224 INSCRIPTION blocked on arc 225
Stone 224.6 — L2 cosmetic disposition (defer-with-note vs in-arc)
Stone 224.7 — INSCRIPTION (arc 224 CLOSES)
   ← unblocks arc 221 INSCRIPTION when 222 + 223 also close
```

### Calibration estimates

| Stone | Scope | Predicted | Notes |
|---|---|---|---|
| 224.4 | aggregate paperwork | ~30 min orchestrator-direct | SHIPPED HERE |
| 224.5 | Group A + stale-doc L2s | 90-150 min sonnet Mode A | mostly mechanical; cascade test risk LOW |
| 224.6 | L2 cosmetic disposition | 30-45 min orchestrator-direct | decide defer-with-note or fold |
| 224.7 | INSCRIPTION | 30 min paperwork | blocked on arc 225 |
| **arc 225** | atomize/materialize rename | 3-5 hours across 2-3 stones | substrate-wide; sonnet + cascade fixes |

### Spawn-block accounting

After this stone, the spawn tree depth at arc 224:

```
arc 220 (waits)
  └→ arc 221 (waits)
       ├→ arc 222 (pending)
       ├→ arc 223 (pending)
       └→ arc 224 (active head)
            └→ arc 225 (atomize/materialize rename) — spawns after Stone 224.5
```

Arc 221's INSCRIPTION (Stone 221.6) waits for {222, 223, 224} closure. Arc 224's INSCRIPTION (Stone 224.7) waits for arc 225 closure. The chain is now 4-5 levels deep at the deepest point.

Per `feedback_spawn_block_winding`: wind forward depth-first. Close arc 225 fully (its own INSCRIPTION). Return to close arc 224. Then move to arc 223 or arc 222 (siblings). Then arc 221. Then arc 220.

## Honest deltas + recommendation

**Recommendation: proceed with the A/B split as articulated above** (Group A in-arc as Stone 224.5; Group B as spawned arc 225).

Rationale:
- Group A fixes are bite-sized, mechanical, low-risk; cascade tests minimal; cluster well as one stone
- Group B is substrate-wide, needs its own BRIEF/EXPECTATIONS/SCORE/INSCRIPTION discipline; folding it into arc 224 would conflate "audit + small fixes" with "substrate-doctrine-rename arc" — different scope shapes
- Spawn-block discipline is cleanly honest with this split: arc 224 closes substrate naming honesty audit; arc 225 closes the atomize/materialize rename; both contribute to closing arc 221's spawn tree

**Open question for user before opening arc 225 BRIEF:**
- Naming check: `atomize` / `materialize` is the proposed pair per intueri's family-pattern finding. ANY pushback on those specific names? `:wat::holon::atomize` reads cleanly (verb-action, namespace-prefixed). `:wat::holon::materialize` is the symmetric inverse. Alternatives if these don't land: `lift` / `lower`, `encode` / `decode`, `to-holon` / `from-holon` (the latter conflicts with `:wat::holon::from-watast` namespace shape so probably no).

## Cross-references

- arc 224 DESIGN.md — full audit scope
- FINDINGS-INTUERI-HOLON-AST.md — 224.1 cast
- FINDINGS-INTUERI-RUNTIME.md — 224.2 cast (the L1-runtime-1 family pattern source)
- FINDINGS-INTUERI-CHECK.md — 224.3 cast (the L1-check-A doc lie source)
- arc 225 DESIGN.md — atomize/materialize rename arc (drafted alongside this aggregate)
- `feedback_spawn_block_winding` — arc 225 spawn parentage discipline
- [[atom-is-holder]] memory — the substrate doctrine that surfaced this audit
- INTERSTITIAL § 2026-05-22 very-late → 2026-05-23 — the realization arc
