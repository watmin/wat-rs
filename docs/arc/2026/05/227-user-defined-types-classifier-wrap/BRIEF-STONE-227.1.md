# BRIEF — Arc 227 Stone 227.1 — User-defined types via `:wat::holon::defclass` macro

**Stone scope:** Mint `(:wat::holon::defclass <fqdn>)` defmacro that expands to constructor + predicate auto-generation in the USER-DECLARED namespace. Pure macro expansion using arc 226/228 primitives — NO new substrate primitives. **The LAST stone of the typed-entities chain.**

**Type:** Sonnet Mode A.
**Time budget:** 60-120 min target; 180 min STOP.
**Depends on:** Stone 226.1 SHIPPED (`e7ba909`); `:wat::holon::is?` + `:wat::holon::Bind` + `:wat::holon::Atom` all live.
**Calibration:** Closest precedents — Stone 226.1 (~11 min), Stone 230.1 (~30 min). This stone is similar scope.

## v3 supersedes v1 (orchestrator stop+reframe)

The original v1 BRIEF had two honest violations the user caught before sonnet shipped:

1. **Namespace insertion** — v1 had `(defclass Voltage)` → `(defn :user::Voltage ...)`. This INSERTS into `:user::*` namespace on the user's behalf — a violation. Users must declare their own FQDN namespace.
2. **Wrong namespace for defclass itself** — v1 had `:wat::core::defclass`. Typed entities are holon-tier; defclass mints classifier-wrapped holons. Correct namespace: `:wat::holon::defclass`.

v3 corrects both. The macro REQUIRES user-declared FQDN; lives in `:wat::holon::*`.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`**
- Branch: `arc-170-gap-j-v5-deadlock-state`
- DO NOT commit. DO NOT touch holon-rs. DO NOT touch wat-edn.
- **HARD CUT** discipline.

## BASH DISCIPLINE

- ONE cargo command at a time, foreground; no piping; no concurrent runs
- 5 known signal-handler test hangs (task #413) — skip per Verification

## Doctrine — users declare; substrate stays uninvolved

Per [[typed-entities-doctrine]] + `feedback_fqdn_is_the_namespace`:
- Every typed value at user-surface = `(Bind (Atom <ClassifierString>) (Atom <data>))`
- FQDN IS the namespace — users declare full paths; no shortcuts
- The substrate has 12 true primitives; everything else (including user types) is composition
- Arc 227 mints the user-facing surface — `:wat::holon::defclass` lives in the holon namespace because it manages holon-algebra typed entities

## Design — `:wat::holon::defclass` defmacro

**Input:** a FQDN keyword (user's choice; user declares their own namespace).

```
(:wat::holon::defclass :myapp::Voltage)
```

**Output (expanded by macro):**

```
(:wat::core::do
  (:wat::core::defn :myapp::Voltage [v]
    (:wat::holon::Bind
      (:wat::holon::Atom "myapp::Voltage")
      (:wat::holon::Atom v)))
  (:wat::core::defn :myapp::is-Voltage? [v]
    (:wat::holon::is? v "myapp::Voltage")))
```

**Naming rules:**

| Input | Constructor verb | Predicate verb | Classifier string |
|---|---|---|---|
| `:myapp::Voltage` | `:myapp::Voltage` | `:myapp::is-Voltage?` | `"myapp::Voltage"` |
| `:awesome::lib::Sensor` | `:awesome::lib::Sensor` | `:awesome::lib::is-Sensor?` | `"awesome::lib::Sensor"` |
| `:test::Foo` | `:test::Foo` | `:test::is-Foo?` | `"test::Foo"` |

**Classifier string = FQDN without leading colon.** This makes user types collision-free across applications:
- `:appA::Voltage` produces classifier `"appA::Voltage"`
- `:appB::Voltage` produces classifier `"appB::Voltage"`
- Instances are distinct; `:appA::is-Voltage?` and `:appB::is-Voltage?` discriminate honestly

**Predicate name = parent-namespace + `is-` + basename + `?`.** The is- prefix attaches to the basename (last `::` segment), keeping the namespace structure intact.

**STONE 227.1 SHIPS ONLY THE SIMPLE FORM (single-arg defclass).** Inheritance via classifier-chain (`(defclass :myapp::U8 :wat::core::Int)`) is Stone 227.2 territory.

## Your scope (sonnet)

### Phase 1 — Author the `:wat::holon::defclass` defmacro

Locate the appropriate stdlib wat path:
- Check existing pattern: `grep -rn "defmacro" wat/holon/` for holon-tier defmacro precedents
- If `wat/holon/` lacks a defmacro precedent, check `wat/core/` (e.g., the defn defmacro template)
- Mint `wat/holon/defclass.wat` (preferred — defclass is holon-tier)

The defmacro shape (sketched; sonnet refines syntax + computes the symbol-manipulation):

```
(:wat::core::defmacro :wat::holon::defclass [fqdn]
  (:wat::core::quasiquote
    (:wat::core::do
      (:wat::core::defn ~fqdn [v]
        (:wat::holon::Bind
          (:wat::holon::Atom ~(classifier-string-from fqdn))
          (:wat::holon::Atom v)))
      (:wat::core::defn ~(predicate-fqdn-from fqdn) [v]
        (:wat::holon::is? v ~(classifier-string-from fqdn))))))
```

Where `classifier-string-from` and `predicate-fqdn-from` are SUBSTRATE-LEVEL keyword-manipulation helpers. **Investigate what's available**:
- `grep -rn "keyword/to-string\|keyword-name\|keyword/from-string\|symbol-rename" src/` for the available symbol-manipulation primitives
- Existing defmacros that rename/compose keywords are precedent

If the substrate lacks needed keyword-manipulation helpers, propose them as STOP-5b finding — but try to use existing machinery first (the substrate has reflection per arc 201; the helpers likely exist).

### Phase 2 — Auto-register the defmacro

If the macro file is in a stdlib-scanned path, no Rust change. Otherwise add to `register_stdlib_defmacros` in src/macros.rs.

Verify path: `grep -rn "wat/holon\|wat/core\|stdlib_path" src/macros.rs`.

### Phase 3 — Tests

New test file `tests/probe_arc227_stone1_defclass.rs`:

**Test 1 — Single FQDN defclass + construct + query**:
```
(:wat::holon::defclass :test::Voltage)
(:test::Voltage 5.0)                       ; constructs an instance
(:test::is-Voltage? instance)              ; returns true
(:test::is-Voltage? "random string")       ; false (not a Voltage holon)
```

**Test 2 — Cross-namespace independence (collision-free verification)**:
```
(:wat::holon::defclass :appA::Voltage)
(:wat::holon::defclass :appB::Voltage)
; instance from A is NOT B; predicate discrimination works
; (:appA::is-Voltage? appA-instance) → true
; (:appA::is-Voltage? appB-instance) → false  (classifier "appA::Voltage" vs "appB::Voltage")
```

**Test 3 — Multiple distinct user types in same namespace**:
```
(:wat::holon::defclass :test::Celsius)
(:wat::holon::defclass :test::Kelvin)
; cross-discriminate
```

**Test 4 — User type vs built-in type**:
```
(:wat::holon::defclass :test::MyMap)
; (:test::is-MyMap? instance) true; (:wat::holon::is-Map? instance) false
; built-in (:wat::holon::Map ...) still works
```

**Test 5 — Polymorphic is? works on user-defined**:
```
(:wat::holon::is? user-instance "test::Voltage") ; returns true
(:wat::holon::is? user-instance "Voltage")        ; returns false (no FQDN-vs-basename collision)
```

**Test 6 — Constructor errors on non-atomizable**:
```
(:test::Voltage <some-fn>)  ; errors at check time per arc 225 narrow Atom
```

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

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` empty.

**Write `wat-rs/docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.1.md`** mirroring SCORE-STONE-226.1.md shape.

## STOP triggers

- **STOP-1 (compile error UNEXPECTED):** STOP and report
- **STOP-2 (test failure beyond new probe):** STOP + diagnose
- **STOP-3 (180 min elapsed):** wall-clock STOP
- **STOP-4 (holon-rs touched):** STOP and report
- **STOP-5 (substrate-primitive route taken when wat-defmacro works):** STOP — defclass should be PURE macro expansion using existing primitives
- **STOP-5b (substrate lacks keyword-manipulation helpers):** if classifier-string-from / predicate-fqdn-from CANNOT be built from existing substrate, STOP and surface as finding — orchestrator decides whether to mint substrate helpers (would expand arc 227 scope) OR defer
- **STOP-6 (inheritance scope creep):** Stone 227.1 ships single-arg defclass ONLY; multi-arg with base-type is Stone 227.2
- **STOP-7 (bash discipline):** cargo hang from pipes
- **STOP-8 (namespace insertion violation):** if you find yourself generating `:user::*` or any namespace the user didn't declare, STOP — users declare their own FQDN; the macro extracts + uses what user gave

## Out-of-scope

- Class inheritance via classifier-chain (Stone 227.2)
- Multimethod dispatch integration with arc 146/147 (Stone 227.3+)
- VSA similarity scoring (Stone 226.2 — different arc)
- USER-GUIDE chapter (Stone 227.4 closure paperwork)
- INSCRIPTION (Stone 227.4 — closes arc 227; cascades up the chain)
- holon-rs / wat-edn changes
- Aliases (HARD CUT)

## Doctrine context — the closing arc + the FQDN doctrine reaffirmed

Stone 227.1 ships the LAST piece of the typed-entities doctrine + reaffirms `feedback_fqdn_is_the_namespace`:

```
arc 225 ✓ — bridge naming family (substrate verbs honest)
arc 228 ✓ — collection classifier-wrap
arc 230 ✓ — variant retirement (substrate 16 → 12 primitives)
arc 226 ✓ — type predicates (substrate IS the type system)
arc 227 (THIS STONE) — user-defined types in USER-DECLARED namespaces
```

Users invent classifier names in their own namespaces. Substrate doesn't insert into `:user::*` or any pre-declared shortcut. The doctrine: full FQDN; user owns their namespace; substrate provides the algebra; type-checking emerges.

The duck has a measurable shape; users name new ducks in namespaces they own.

## Implementation hints

- Investigate `wat/holon/` and `wat/core/` for defmacro precedents (defservice, defn-restricted, defn template, etc.)
- The `:wat::core::defmacro` form is the substrate primitive
- `:wat::core::quasiquote` + `:wat::core::unquote` for template construction
- Keyword manipulation: investigate what's available — likely `:wat::core::keyword/to-string`, `:wat::core::string/concat`, `:wat::core::keyword/from-string` or similar; arc 201 reflection layer may provide
- The constructor side uses `:wat::holon::Bind` + `:wat::holon::Atom` (both narrow constructors post-arc-225)
- The predicate side uses `:wat::holon::is?` from arc 226
