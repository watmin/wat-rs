# BRIEF — Arc 230 Stone 230.1 — Substrate variant retirement (Symbol/Keyword/Tag/Nil → pure Bind compositions)

**Stone scope:** Retire 4 convenience variants from `HolonAST` (holon-rs) + cascade the ripple across wat-rs + caller sweep. Atomic combined stone: holon-rs Phase A + wat-rs Phase B + cascade Phase C in one sonnet flight. **Substrate-as-teacher cascade methodology** per FM 15. **LARGEST refactor in arc 170+'s history.**

**Type:** Sonnet Mode A.
**Time budget:** 240-420 min target; 480 min STOP (8 hours).
**Depends on:** Stone 228.1 SHIPPED (`29cc984`); classifier-wrap pattern established for collections (provides the precedent encoding for the retired variants).
**Calibration:** Closest precedents — Stone 225.1 v3 (~68 min for 150-200 sites) + Stone 228.1 (~36 min for ~100 sites). This stone is ~2-3x scope (touches BOTH holon-rs and wat-rs; ~300-500 sites estimated). Calibration trend favors faster-than-target.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`** (primary)
- **ALSO TOUCHES: `/home/watmin/work/holon/holon-rs/`** — substrate enum changes
- Branch: `arc-170-gap-j-v5-deadlock-state` (wat-rs); holon-rs branch should also align (verify before changes)
- Linux only; no `--no-verify`
- DO NOT commit. Orchestrator commits BOTH repos atomically when workspace green per atomic-commit pattern (recovery doc § 7).
- DO NOT touch wat-edn (wire format unaffected; bytes change at structural-encoding level only)
- **HARD CUT — no aliases / no deprecation wrappers.** Variants DELETED entirely.

## BASH DISCIPLINE

- ONE cargo/git command at a time, foreground
- NO piping through `| grep` / `| tail`
- NO concurrent background cargo runs
- `cargo test --release --lib -p wat` has 5 known signal-handler test hangs (task #413). Skip per Verification.

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Variants to retire (`/home/watmin/work/holon/holon-rs/src/kernel/holon_ast.rs`)

| Variant | Line | Storage |
|---|---|---|
| `Symbol(Arc<str>)` | 66 | name string |
| `Keyword(Arc<str>)` | 100 | name string (no leading colon per arc 221 doctrine) |
| `Nil` | 108 | unit |
| `Tag(Arc<str>)` | 119 | name string (no leading hash) |

### Retained primitives (12; the true substrate algebra)

| Category | Variants |
|---|---|
| Composers | Atom, Bind, Bundle, Permute |
| Carriers | I64, F64, Bool, Char, String |
| Encoders | Thermometer, Blend |
| Sentinel | SlotMarker |

### Replacement encoding (per typed-entities doctrine)

```
Symbol("foo")  →  Bind(Atom(String("Symbol")), Atom(String("foo")))
Keyword("foo") →  Bind(Atom(String("Keyword")), Atom(String("foo")))
Tag("foo")     →  Bind(Atom(String("Tag")), Atom(String("foo")))
Nil            →  Bind(Atom(String("Symbol")), Atom(String("nil")))  ; per user 2026-05-22 articulation
```

Each retired variant becomes a `Bind(Atom, Atom)` composition. The classifier name is the OUTER atom; the carrier (string) is the INNER atom. **Recursion stops at raw String at the bottom.**

### Substrate sites affected (estimates)

- **holon-rs**: `kernel/holon_ast.rs` enum variants + cascade arms (`as_bytes`, `canonical_bytes`, traversal helpers, Display impl, test fixtures, `PRIM_TAG_*` constants for the 4 retired)
- **wat-rs**: ~30-50 `HolonAST::Symbol|Keyword|Tag|Nil` match arms in `runtime.rs` + `check.rs` + `freeze.rs` + `lower.rs`
- **wat-rs tests**: ~20-40 sites asserting on variant shapes
- **wat-side**: minimal (most wat code goes through verbs not raw variant matching)
- **Total estimated**: ~300-500 touch points

## Your scope (sonnet)

### Phase A — holon-rs variant retirement

cwd: `/home/watmin/work/holon/holon-rs/`

1. **Remove** `HolonAST::Symbol`, `HolonAST::Keyword`, `HolonAST::Nil`, `HolonAST::Tag` variant declarations from `src/kernel/holon_ast.rs` (lines 66, 100, 108, 119)
2. **Remove** the corresponding cascade arms (every `match` arm referencing these 4 variants — should be in `canonical_bytes`, `as_bytes`, Display, traversal helpers, debug helpers)
3. **Remove** the constructor helpers (e.g., `HolonAST::symbol`, `HolonAST::keyword`, `HolonAST::tag`, `HolonAST::Nil` direct construction) — provide replacements that produce the Bind-Atom-Atom composition instead. Naming: keep the lowercase constructor fns (`symbol(s)`, `keyword(s)`, `tag(s)`, `nil()`) but have them PRODUCE the composition rather than the bare variant.
4. **Remove** `PRIM_TAG_SYMBOL`, `PRIM_TAG_KEYWORD`, `PRIM_TAG_TAG`, `PRIM_TAG_NIL` constants if they exist (substrate distinction now comes from Bind structure encoding, not seed tags)
5. **Update** holon-rs tests asserting on `HolonAST::Symbol|Keyword|Tag|Nil` variant shapes — they now assert on the Bind composition
6. **Verify**: `cd /home/watmin/work/holon/holon-rs/ && cargo build --release && cargo test --release` — holon-rs internally green

### Phase B — wat-rs ripple

cwd: `/home/watmin/work/holon/wat-rs/`

7. **Update every** `HolonAST::Symbol|Keyword|Tag|Nil` match arm in:
   - `src/runtime.rs` — match arms read these via the helper-fn output now (Bind composition); arms that CREATE these should use the updated constructor fns (which produce Bind compositions)
   - `src/check.rs` — same pattern; TypeScheme registrations may need adjustment
   - `src/freeze.rs` — same pattern
   - `src/lower.rs` — same pattern (the algebra-tier `Atom` lowerer per arc 225 Delta 3)
8. **The `extract_classifier` helper from arc 228** (`src/runtime.rs`) — verify it correctly recognizes the new Symbol/Keyword/Tag/Nil compositions (their outermost form is `Bind(Atom(String("Symbol")), _)` etc. — the helper should already handle these since the shape matches the collection classifier pattern)
9. **`to_holon_inner` arms** for `Value::wat__core__keyword`, `Value::Unit` — these previously produced `HolonAST::Keyword(...)` / `HolonAST::Nil` directly; now they produce Bind compositions via the updated helper-fn
10. **`from_holon_item` / `eval_holon_from_holon`** — recognize the new compositions; dispatch by classifier ("Symbol" / "Keyword" / "Tag" "Nil") in addition to the existing collection classifiers

### Phase C — Substrate-as-teacher cascade

11. From `wat-rs/`: `cargo build --release -p wat` — many errors expected
12. **Iterate per FM 15**: read errors, apply rule, rerun until green
13. **Run wat-side test fixtures** — `wat/**/*.wat` + `wat-tests/**/*.wat` cascade typically minimal (most go through verbs); fix as cascade surfaces

### Phase D — Doc refresh as discovered

14. Touch adjacent doc comments referencing the retired variants — note arc 230's retirement; the variants are now CONVENIENCES served by helper-fn-produced Bind compositions

### Verification (after all green)

Run each ONE AT A TIME, foreground, no pipes:

```
# In holon-rs:
cd /home/watmin/work/holon/holon-rs/ && cargo build --release
cd /home/watmin/work/holon/holon-rs/ && cargo test --release
cd /home/watmin/work/holon/holon-rs/ && cargo clippy --release -- -D warnings

# In wat-rs (return to wat-rs cwd):
cd /home/watmin/work/holon/wat-rs/ && cargo build --release -p wat
cd /home/watmin/work/holon/wat-rs/ && cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2
cd /home/watmin/work/holon/wat-rs/ && cargo test --release --test probe_arc216_stone1_hashset_roundtrip
cd /home/watmin/work/holon/wat-rs/ && cargo test --release --test probe_arc216_stone2_vector_roundtrip
cd /home/watmin/work/holon/wat-rs/ && cargo test --release --test probe_arc216_stone3_hashmap_roundtrip
cd /home/watmin/work/holon/wat-rs/ && cargo test --release --test probe_arc216_stone7_tuple_roundtrip
cd /home/watmin/work/holon/wat-rs/ && cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cd /home/watmin/work/holon/wat-rs/ && cargo test --release --test wat_arc143_manipulation
cd /home/watmin/work/holon/wat-rs/ && cargo test --release --test mvp_end_to_end
cd /home/watmin/work/holon/wat-rs/ && cargo test --release -p wat-edn
cd /home/watmin/work/holon/wat-rs/ && cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

All must complete cleanly.

**Write `wat-rs/docs/arc/2026/05/230-substrate-variant-retirement/SCORE-STONE-230.1.md`** mirroring SCORE-STONE-228.1.md shape.

## STOP triggers

- **STOP-1 (compile error UNEXPECTED):** cargo errors NOT from the cascade — STOP
- **STOP-2 (test failure beyond cascade consequences):** STOP + diagnose + frame per Stone 221.3 Delta 1a honest framing
- **STOP-3 (480 min elapsed):** wall-clock STOP
- **STOP-4 (wat-edn touched accidentally):** STOP (wire format must NOT change)
- **STOP-5 (new lie family discovered):** out-of-scope; surface as finding
- **STOP-6 (round-trip semantics break):** if Symbol/Keyword/Tag/Nil round-trip silently corrupts, STOP + diagnose
- **STOP-7 (bash discipline):** cargo hang from accidental pipes; check + simplify
- **STOP-8 (VSA vector identity collision):** the new Bind-encoded Symbol/Keyword/Tag/Nil canonical-bytes must remain distinct from Bind-encoded other-classifiers (per typed-entities doctrine — structural-encoding is the discriminator now). If `Symbol("foo")` vector collides with `Keyword("foo")` vector under the new encoding, STOP + diagnose

## Out-of-scope

- wat-edn wire format changes (wire stays per arc 218/219 doctrine)
- Type predicates `(is-Map? x)` (arc 226 scope; depends on extract_classifier from arc 228 + the unified classifier-encoding from arc 230)
- User-defined types (arc 227)
- EDN-form named constructors at wat-surface (arc 222)
- WatAST primitive-layer honesty (arc 223)
- Quasiquote evaluator changes (arc 229; deferred)
- INSCRIPTION (Stone 230.4 + arc 228 INSCRIPTION blocked on 230 closing per spawn-block)
- Aliases for backwards compatibility (HARD CUT)
- from-holon -> :T type-hint propagation (Task #469; orthogonal future stone)

## Notes on forward-correction of arc 221

Arc 221 Stones 221.3 + 221.5 minted `HolonAST::Symbol`, `Keyword`, `Tag`, `Nil` variants + their PRIM_TAG seed-distinction work. Arc 230 supersedes via the typed-entities doctrine (variants are CONVENIENCES; pure Bind composition is honest). Per `feedback_inscription_immutable`:
- Arc 221's INSCRIPTION (when it ships) records the variant minting as it happened
- Arc 230 SCORE + INSCRIPTION forward-correct the encoding
- The disk records the full arc of understanding — original variant doctrine + composition supersession + reasoning

This is a documentation responsibility, not a code responsibility. Arc 221 in-code comment headers can be updated as you touch them (Phase D) noting "arc 221 minted; arc 230 supersedes with Bind composition."

## Wat-reveals-holon dynamic (4th application)

This is the 4TH time the wat-reveals-holon dynamic has surfaced a substrate gap that holon work alone wouldn't have found:
1. arc 221 (2026-05-22 morning) — wat-surface maturity exposed convention-based-collapse dishonesty in HolonAST::Symbol; variants minted
2. arc 224 (2026-05-22 evening) — intueri audit exposed verb-naming lies; arc 225 fixed
3. arc 228 (2026-05-22 late) — typed-entities doctrine landed; collections classifier-wrapped
4. **arc 230 (NOW)** — doctrine application reveals the variants themselves are conveniences; pure Bind composition is the honest algebra

The 4-month timeline ([[typed-entities-doctrine]] memory) keeps producing convergence-shaped wins. This is the BIG one — the substrate algebra reduces to its true 12 primitives. Holon found itself again, through wat.
