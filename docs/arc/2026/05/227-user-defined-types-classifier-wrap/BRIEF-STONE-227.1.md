# BRIEF — Arc 227 Stone 227.1 — User-defined types via `defclass` macro

**Stone scope:** Mint `(:wat::core::defclass Name)` defmacro that expands to constructor + predicate auto-generation. Pure macro expansion using arc 226/228 primitives — NO new substrate primitives needed. **The LAST stone of the typed-entities chain.**

**Type:** Sonnet Mode A.
**Time budget:** 60-120 min target; 180 min STOP.
**Depends on:** Stone 226.1 SHIPPED (`e7ba909`); `:wat::holon::is?` + `:wat::holon::Bind` + `:wat::holon::Atom` all live; uniform classifier-encoding established.
**Calibration:** Closest precedents — Stone 226.1 (~11 min), Stone 230.1 (~30 min), Stone 228.1 (~36 min). This stone is even smaller scope (single macro + tests; no encoding cascade; no caller sweep). Pattern locked.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`**
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- Linux only; no `--no-verify`
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch holon-rs (substrate is settled post-arc-230)
- DO NOT touch wat-edn
- **HARD CUT** discipline

## BASH DISCIPLINE

- ONE cargo command at a time, foreground
- NO piping through `| grep` / `| tail`
- 5 known signal-handler test hangs (task #413) — skip per Verification

## The doctrine — user types are unlimited via classifier-wrap

Per [[typed-entities-doctrine]]:
- Every typed value at user-surface = `(Bind (Atom <ClassName>) (Atom <data>))`
- The substrate has 12 true primitives; everything else is composition
- Users invent classifier names; the substrate doesn't need to know about them
- `(is-X? x)` queryability follows automatically via arc 226's polymorphic `is?` machinery

Arc 227 mints the user-facing surface — `(defclass MyType)` declares a new type; substrate auto-generates the constructor + predicate.

## Design — wat-level defmacro (pure expansion)

**Path chosen: wat-level defmacro** (no new substrate primitives). The defclass form expands at macro time to two defns: one constructor + one predicate.

Example (the doctrine in code):

```
(:wat::core::defclass Voltage)
```

Expands to:

```
(:wat::core::defn :user::Voltage [v -> :wat::holon::HolonAST]
  (:wat::holon::Bind (:wat::holon::Atom "Voltage") (:wat::holon::Atom v)))

(:wat::core::defn :user::is-Voltage? [v -> :wat::core::bool]
  (:wat::holon::is? v "Voltage"))
```

Users then call `(:user::Voltage 5.0)` to construct a Voltage instance + `(:user::is-Voltage? x)` to query.

**Optional inheritance** (Stone 227.1 OR deferred to 227.2):

```
(:wat::core::defclass U8 Int)
```

Expands to constructor that wraps in classifier-chain: `Bind(Atom("U8"), Bind(Atom("Int"), Atom(value)))`. Predicate `is-U8?` matches outer; `is-Int?` (from arc 226) matches inner.

**STONE 227.1 SHIPS ONLY THE SIMPLE FORM (no inheritance).** Inheritance is Stone 227.2 territory if needed. Single-arg defclass first; verify the pattern; extend later.

## Your scope (sonnet)

### Phase 1 — Author the defclass defmacro

Locate the appropriate stdlib wat file (likely `wat/core/` or `wat/holon/`):
- Check existing pattern: `grep -r "defmacro" wat/` to find precedents like defservice, defn-restricted, struct-restricted
- Mint `wat/core/defclass.wat` (or appropriate namespace) containing the defmacro definition

The defmacro shape (sketched; sonnet refines syntax):

```
(:wat::core::defmacro :wat::core::defclass [name]
  (:wat::core::quasiquote
    (:wat::core::do
      (:wat::core::defn (:wat::core::unquote (build-constructor-name name)) [v]
        (:wat::holon::Bind
          (:wat::holon::Atom (:wat::core::unquote (name-as-string name)))
          (:wat::holon::Atom v)))
      (:wat::core::defn (:wat::core::unquote (build-predicate-name name)) [v]
        (:wat::holon::is? v (:wat::core::unquote (name-as-string name)))))))
```

(Sketched; sonnet picks the right combinators — the substrate has tools for this. Investigate the defservice macro pattern; mirror its shape.)

### Phase 2 — Register the defmacro in stdlib

If the macro lives in a new wat file, ensure it's loaded via `register_stdlib_defmacros` in src/macros.rs. Pattern: existing defmacros are auto-registered from the stdlib paths.

If the new wat file is in a path already scanned, no Rust change needed. Verify via `grep -rn "stdlib_paths\|stdlib_defmacros" src/`.

### Phase 3 — Tests

New test file `tests/probe_arc227_stone1_defclass.rs`:

**Test 1 — Simple defclass + constructor + predicate**:
- `(defclass Voltage)` declares
- `(Voltage 5.0)` constructs an instance
- `(is-Voltage? instance)` returns true
- `(is-Voltage? "random string")` returns false
- `(is-Voltage? (Map ...))` returns false (different classifier)

**Test 2 — Multiple user types**:
- `(defclass Celsius)` + `(defclass Kelvin)`
- Cross-discrimination: Celsius instance is NOT Kelvin and vice-versa
- Both queryable via polymorphic `is?`

**Test 3 — User types vs built-in types**:
- `(defclass MyMap)` — distinct from built-in Map
- `(is-MyMap? instance)` true; `(is-Map? instance)` false (different classifier strings)
- Built-in `(Map ...)` still works independently

**Test 4 — Edge cases**:
- `(defclass A)` then `(defclass A)` — sonnet decides: error (re-declaration) OR idempotent. Pick honest behavior; document.
- Constructor invoked with non-atomizable arg — should error per arc 225's `:wat::holon::Atom` narrow constructor

### Phase 4 — Verification

Run each ONE AT A TIME, foreground:

```
cargo build --release -p wat
cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2
cargo test --release --test probe_arc227_stone1_defclass
cargo test --release --test probe_arc226_stone1_type_predicates
cargo test --release --test probe_arc216_stone1_hashset_roundtrip
cargo test --release --test probe_arc216_stone2_vector_roundtrip
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip
cargo test --release --test probe_arc216_stone4_predicate_composition
cargo test --release --test probe_arc216_stone7_tuple_roundtrip
cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cargo test --release --test wat_arc143_manipulation
cargo test --release --test mvp_end_to_end
cargo test --release -p wat-edn
cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

All must complete cleanly.

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` must be empty.

**Write `wat-rs/docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.1.md`** mirroring SCORE-STONE-226.1.md shape.

## STOP triggers

- **STOP-1 (compile error UNEXPECTED):** STOP and report
- **STOP-2 (test failure beyond new probe):** STOP + diagnose
- **STOP-3 (180 min elapsed):** wall-clock STOP
- **STOP-4 (holon-rs touched):** STOP and report
- **STOP-5 (substrate-primitive route taken when wat-defmacro works):** if you find yourself adding a new Rust-level macro handler instead of using `defmacro` + existing primitives, STOP and reconsider — the doctrine says "users invent types; substrate doesn't need to know"; arc 227 should honor that with macro expansion only
- **STOP-6 (inheritance scope creep):** Stone 227.1 ships single-arg defclass ONLY; multi-arg with base-type inheritance is Stone 227.2 territory
- **STOP-7 (bash discipline):** cargo hang from pipes

## Out-of-scope

- Class inheritance via classifier-chain (Stone 227.2)
- Multimethod dispatch integration with arc 146/147 (Stone 227.3 or future arc)
- VSA similarity scoring for fuzzy class membership (Stone 226.2 — different arc)
- USER-GUIDE chapter (Stone 227.4 closure paperwork)
- INSCRIPTION (Stone 227.4; this is the LAST chain stone — its INSCRIPTION unblocks arc 226 → 228 → 225 → 224 → 221 → 220 INSCRIPTION cascade)
- holon-rs changes
- wat-edn changes
- Aliases (HARD CUT)

## Doctrine context — the closing arc

Stone 227.1 ships the LAST piece of the typed-entities doctrine implementation:

```
arc 225 ✓ — bridge naming family (substrate verbs honest)
arc 228 ✓ — collection classifier-wrap (Map/Set/Vector/List/Tuple → (Bind (Atom name) (Bundle...)))
arc 230 ✓ — variant retirement (substrate 16 → 12 primitives; Symbol/Keyword/Tag/Nil → Bind compositions)
arc 226 ✓ — type predicates (substrate IS the type system)
arc 227 (THIS STONE) — user-defined types (the type system EXTENDS unboundedly without substrate changes)
```

Once arc 227 closes, the whole chain unwinds. The substrate is impeccable: 12 true primitives + classifier-wrap doctrine + queryable types + user-extensible type universe. The wat-reveals-holon dynamic completes its 5th application this chain.

The duck has a measurable shape AND users can name new ducks.

## Implementation hints

- defservice (arc 209) is the closest precedent — search `wat/std/service/Console.wat` (historical) or `wat/console/` for the pattern
- defn-restricted (arc 198) is another precedent — `wat/core/defn-restricted.wat` likely
- The `:wat::core::defmacro` form is the substrate primitive that mints macros
- `:wat::core::quasiquote` + `:wat::core::unquote` are used for template construction inside defmacro bodies
- `:wat::holon::is?` was minted in Stone 226.1 — the predicate side uses it directly
- The constructor side uses `:wat::holon::Bind` + `:wat::holon::Atom` (both narrow constructors post-arc-225)
