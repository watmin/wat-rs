# BRIEF — Arc 228 Stone 228.1 — Substrate collection classifier-wrap + Pascal-Case constructor verbs

**Stone scope:** Mint 5 Pascal-Case constructor verbs (Map/Set/Vector/List/Tuple) at substrate; update `to_holon_inner` collection arms to produce classifier-wrapped `(Bind (Atom "ClassName") (Bundle ...))` compositions; update `eval_holon_from_holon` to dispatch by classifier-atom name. **Substrate-as-teacher cascade methodology** per FM 15.

**Type:** Sonnet Mode A.
**Time budget:** 120-240 min target; 300 min STOP.
**Depends on:** Arc 225 Stone 225.1 v3 SHIPPED (commit `189b033`); bridge family clean.
**Calibration:** Closest precedent — Stone 225.1 v3 (~68 min for 150-200 site rename + 5 deliverables). This stone is similar substrate-surface work but more focused (less consumer sweep since arc 216 probes already touch the encoding; mostly substrate-internal change). Pattern locked.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`** (NOT holon-rs!)
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- Linux only; no `--no-verify`
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch holon-rs (arc 230 retires variants later; not this arc)
- DO NOT touch wat-edn (wire format unaffected)
- **HARD CUT — no aliases.** Per arc 225 precedent + user's "fractal of correctness" principle.

## BASH DISCIPLINE

- ONE cargo/git command at a time, foreground
- NO piping cargo output through `| grep` / `| tail`
- NO concurrent background cargo runs
- `cargo test --release --lib -p wat` has 5 known signal-handler test hangs (task #413). Use targeted-skip command in Verification.

## Pre-flight grep verified (orchestrator-grep'd 2026-05-22)

### Current state of `to_holon_inner` collection arms (`src/runtime.rs:13906+`)

After arc 225 Stone 225.1 v3, `to_holon_inner` has these collection arms — ALL produce BARE Bundle (no classifier-wrap):

| Input Value type | Current output (arc 225 baseline) | Lines |
|---|---|---|
| `Value::wat__std__HashSet` | `Bundle(bare items)` | 13957-13967 |
| `Value::Vec` | `Bundle(positional Bind pairs)` | 13972-13984 |
| `Value::Tuple` | `Bundle(positional Bind pairs)` — **IDENTICAL to Vec** | 13991-14003 |
| `Value::wat__std__HashMap` | `Bundle(K-V Bind pairs)` | 14011-14025+ |
| `Value::wat__core__List` | (need fresh grep — likely bare Bundle) | ? |

**The arc 216 doctrine** (per current comments) said Vec/Tuple have IDENTICAL substrate encoding; the consumer's declared TYPE discriminates on the reverse trip. Per arc 228's typed-entities doctrine — **this is forward-corrected.** The encoding MUST carry the classifier so type is recoverable from the data alone.

### Doctrine map for arc 228 (the changes)

| Input Value | NEW output (post arc 228) |
|---|---|
| `Value::wat__std__HashSet(s)` | `Bind(Atom("Set"), Bundle(bare items))` |
| `Value::Vec(v)` | `Bind(Atom("Vector"), Bundle(positional Bind pairs))` |
| `Value::Tuple(t)` | `Bind(Atom("Tuple"), Bundle(positional Bind pairs))` — NOW DISTINCT from Vec |
| `Value::wat__std__HashMap(m)` | `Bind(Atom("Map"), Bundle(K-V Bind pairs))` |
| `Value::wat__core__List(l)` | `Bind(Atom("List"), Bundle(sequential items))` |

Each instance carries its CLASSIFIER as substrate data. Vec/Tuple are now distinct at substrate (was conflated). Reverse trip dispatches by extracting the classifier atom.

### NEW Pascal-Case constructor verbs (substrate)

Mirroring the existing algebra-primitive verb family (`:wat::holon::Bundle`, `Bind`, `Permute`, `Atom`, `Thermometer`, `Blend`, `Tag`):

| New verb | Input | Output |
|---|---|---|
| `:wat::holon::Map` | `Vec<HolonAST>` where items are Bind(k,v) pairs | `Bind(Atom("Map"), Bundle(items))` |
| `:wat::holon::Set` | `Vec<HolonAST>` | `Bind(Atom("Set"), Bundle(items))` |
| `:wat::holon::Vector` | `Vec<HolonAST>` | `Bind(Atom("Vector"), Bundle(positional Bind(I64(i), item)))` |
| `:wat::holon::List` | `Vec<HolonAST>` | `Bind(Atom("List"), Bundle(sequential items))` |
| `:wat::holon::Tuple` | `Vec<HolonAST>` | `Bind(Atom("Tuple"), Bundle(positional Bind(I64(i), item)))` |

Note: Vector and Tuple have IDENTICAL Bundle internals (positional Binds); the OUTER classifier atom (`"Vector"` vs `"Tuple"`) is what distinguishes them. List has different internals (sequential bare items, NOT positional Binds).

### `eval_holon_from_holon` dispatch update

Current (arc 225 baseline): dispatches by Bind-key shape (heuristic three-way recognition of HashSet/HashMap/Vec from raw Bundle).

NEW: dispatch by EXTRACTING the classifier-atom from the outermost Bind, then dispatching by name ("Map" → HashMap; "Set" → HashSet; "Vector" → Vec; "List" → List; "Tuple" → Tuple). Cleaner; no heuristic.

NEW helper: `extract_classifier(holon: &HolonAST) -> Option<&str>` — returns the classifier name if outermost form is `Bind(Atom(<String>), _)`; None otherwise.

## Your scope (sonnet)

### Phase 1 — Mint 5 new Pascal-Case constructor verbs

In `src/runtime.rs`:
- New Rust fn `eval_algebra_map` (or `eval_holon_map_constructor`) — takes a Vec of HolonAST items (each a Bind), produces `Bind(Atom("Map"), Bundle(items))`. Single-input shape; matches sibling `eval_algebra_bundle` pattern.
- Same for `eval_algebra_set` / `Set`
- Same for `eval_algebra_vector` / `Vector` (the substrate auto-applies positional Bind keys to the input items — input is bare items, output wraps them as positional Binds)
- Same for `eval_algebra_list` / `List`
- Same for `eval_algebra_tuple` / `Tuple` (same as Vector internals; different outer classifier)
- Register all 5 in the dispatch table

In `src/check.rs`:
- TypeScheme registrations for each new verb
- `infer_list` special-case handlers if needed (mirror existing Bundle/Bind/Permute special-cases)

In `src/freeze.rs`:
- Freeze-phase registration if needed

### Phase 2 — Update `to_holon_inner` collection arms (classifier-wrap)

For each of the 5 collection arms (HashSet/Vec/Tuple/HashMap/List), wrap the existing Bundle composition in `Bind(Atom("ClassName"), Bundle(...))`:

```rust
// Example for HashSet:
Value::wat__std__HashSet(s) => {
    let mut items: Vec<HolonAST> = Vec::with_capacity(s.len());
    for elem in s.iter() {
        let holon_val = to_holon_inner(elem.clone(), arg_span)?;
        match holon_val {
            Value::holon__HolonAST(h) => items.push((*h).clone()),
            _ => unreachable!("..."),
        }
    }
    let inner_bundle = HolonAST::bundle(items);
    let classified = HolonAST::bind(
        HolonAST::Atom(Arc::new(HolonAST::string("Set"))),  // OR check for HolonAST::Atom wrapping helper
        inner_bundle,
    );
    return Ok(Value::holon__HolonAST(Arc::new(classified)));
}
```

Mirror for Vec/Tuple/HashMap/List. **Vec and Tuple now have DISTINCT outer classifiers** ("Vector" vs "Tuple") — Bundle internals stay identical (positional Binds) but the OUTER Bind discriminates.

Forward-correct the arc 216 comment headers: note that the original arc 216 encoding had bare Bundles; arc 228 wraps with classifier per typed-entities doctrine; arc 216 INSCRIPTION (when it ships) records the original; arc 228 supersedes.

### Phase 3 — Update `eval_holon_from_holon` dispatch

In `src/runtime.rs:13643+`:
- NEW helper `extract_classifier(&HolonAST) -> Option<String>` — returns classifier name if outermost form is `Bind(Atom(String(name)), _)`; None otherwise
- Refactor the Bundle three-way dispatch in `eval_holon_from_holon` to first try `extract_classifier`; if present, dispatch by name ("Map" → HashMap; "Set" → HashSet; "Vector" → Vec; "List" → List; "Tuple" → Tuple)
- If no classifier (bare Bundle), fall back to the existing heuristic (or error if classifier is REQUIRED per the new doctrine — decide based on backward compatibility)

**Recommendation:** require classifier presence for collection decode (no bare-Bundle fallback). Per HARD CUT discipline — the substrate refuses to decode unclassified collections; consumers must produce classifier-wrapped forms. Bare Bundle that's not classifier-wrapped errors with helpful diagnostic.

### Phase 4 — Substrate-as-teacher cascade

After Phase 1-3 land, `cargo build --release -p wat` will fail with errors from:
- Tests asserting on bare-Bundle shapes (arc 216 probe_arc216_stone1/2/3/7 may need updates)
- Any wat-side caller that constructed bare Bundles for collections

**Iterate per FM 15:** read errors, apply rule, rerun until green.

### Phase 5 — Wat-side caller sweep

The `wat/` and `wat-tests/` sources may use any of the existing algebra constructors (Bundle/Bind) directly for collection construction. Under arc 228, these should migrate to the new Pascal-Case constructors (Map/Set/Vector/List/Tuple) where the intent is collection-of-shape.

Sweep:
- `wat/**/*.wat` — substrate-bundled wat files
- `wat-tests/**/*.wat` — test fixtures
- `tests/*.rs` — Rust integration tests with embedded wat strings

Don't blindly replace all Bundle uses — only the ones that semantically construct a Map/Set/Vector/List/Tuple. Bundle-as-algebra-primitive stays as-is.

### Phase 6 — Doc-comment refresh as discovered

While sweeping, refresh adjacent doc comments that describe the bare-Bundle encoding. Note arc 228's classifier-wrap supersession. Fix what you touch; no global hunt.

### Verification (after all green)

Run each command DIRECTLY (no pipes, foreground, one at a time):

```
cargo build --release -p wat
cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2
cargo test --release --test probe_arc216_stone1_hashset_roundtrip
cargo test --release --test probe_arc216_stone2_vector_roundtrip
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip
cargo test --release --test probe_arc216_stone7_tuple_roundtrip
cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cargo test --release --test wat_arc143_manipulation
cargo test --release --test mvp_end_to_end
cargo test --release -p wat-edn
cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

All must complete cleanly (signal-handler hangs explicitly skipped per task #413).

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` must be empty.

**Write `wat-rs/docs/arc/2026/05/228-collection-classifier-wrap/SCORE-STONE-228.1.md`** mirroring SCORE-STONE-225.1.md shape.

## STOP triggers

- **STOP-1 (substrate compile error UNEXPECTED):** cargo errors NOT from the cascade — STOP and report
- **STOP-2 (test failure beyond cascade consequences):** STOP + diagnose + frame per Stone 221.3 Delta 1a discipline (broken-by-this-stone honest framing; do NOT call "pre-existing")
- **STOP-3 (300 min elapsed):** wall-clock STOP
- **STOP-4 (holon-rs touched accidentally):** STOP and report
- **STOP-5 (new lie family discovered):** if other substrate sites exhibit polymorphic-name patterns beyond this stone's scope, surface as finding; do NOT auto-extend
- **STOP-6 (round-trip semantics break):** if classifier-wrap breaks the round-trip property for any collection type (e.g., HashMap round-trip silently corrupts), STOP + diagnose
- **STOP-7 (bash discipline):** cargo hang from accidental pipes; check + simplify

## Out-of-scope

- holon-rs changes (arc 230 retires variants; not this arc)
- wat-edn changes
- Type predicates `(is-Map? x)` (arc 226's scope; arc 228 provides the classifier-extraction helper that arc 226 will consume)
- User-defined types (arc 227)
- EDN-form named constructors at wat-surface like `{...}` parser-mint (arc 222's scope)
- WatAST primitive-layer honesty (arc 223)
- Quasiquote evaluator changes (arc 229; deferred)
- INSCRIPTION (Stone 228.4; blocked on arc 230 closing per spawn-block)
- Aliases for backwards compatibility (HARD CUT)

## Notes on forward-correction of arc 216

Arc 216 Stones 1/2/3/7 inscribed the bare-Bundle encoding per the encoding doctrine at the time. Arc 228 supersedes via the typed-entities doctrine (every typed value at user-surface = `(Bind (Atom class) (Atom data))`). Per `feedback_inscription_immutable`:
- Arc 216 INSCRIPTION (when it ships) records the original bare-Bundle encoding as historical
- Arc 228 forward-corrects via SCORE + INSCRIPTION
- The disk records the full arc of understanding — original doctrine + correction + reasoning

This is a documentation responsibility, not a code responsibility. The arc 216 in-code comment headers can be updated as you touch them (Phase 6 — doc refresh as discovered) noting "arc 216 baseline; arc 228 supersedes with classifier-wrap."
