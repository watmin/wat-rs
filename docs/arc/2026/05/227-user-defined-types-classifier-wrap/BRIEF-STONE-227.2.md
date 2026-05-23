# BRIEF — Arc 227 Stone 227.2 — Multi-field defrecord + auto-accessors

**Stone scope:** Extend `:wat::holon::defrecord` to accept a field-list `[name <- :Type ...]` as optional second argument. Auto-generates constructor with N typed args + N accessors `:ns::Type/<field>` + the predicate (unchanged). Single-arg form (Stone 227.1b) stays — backward compatible.

**Type:** Sonnet Mode A.
**Time budget:** 60-120 min target; 180 min STOP.
**Depends on:** Stone 227.1b SHIPPED (commit `aa2b9f1`); defrecord macro exists at `wat/holon/defrecord.wat`; arc 226 `:wat::holon::is?` + arc 228 `extract_classifier` available; arc 230 classifier-wrap encoding established.
**Calibration:** Closest precedent — Stone 227.1 v3 (~18 min for original defclass mint); Stone 227.1b (~5 min for rename). This stone is more substantial (field-list parsing + N-arg constructor synthesis + N accessor synthesis) but builds on the existing macro shape.

## Doctrine context (per `project_defrecord_defservice_doctrine`)

defrecord wraps immutable data. Multi-field defrecords are STRUCTS — named fields, type-checked, accessed via auto-generated accessors. Methods stay separate defns (per `STONE-227.2-NOTES.md`); defrecord does NOT bundle methods.

The (s, d) -> (s, D) monadic shape applies to defrecord methods (caller-owned threading), but the macro doesn't enforce it — convention only.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`**
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch holon-rs (substrate settled).
- DO NOT touch wat-edn.
- **HARD CUT** discipline — no aliases of any prior shape.

## BASH DISCIPLINE

- ONE cargo command at a time, foreground; no piping; no concurrent runs
- 5 known signal-handler test hangs (task #413) — skip per Verification

## Pre-flight verified (orchestrator-grep'd 2026-05-22 night)

### Existing macro (`wat/holon/defrecord.wat`)

Reviewed in full. Current shape (single-arg):

```
(:wat::core::defmacro
  (:wat::holon::defrecord
    (fqdn :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     (:wat::core::defn ~fqdn [v <- :wat::holon::HolonAST] -> :wat::holon::HolonAST
       (:wat::holon::Bind
         (:wat::holon::Atom (:wat::holon::to-holon ~(:wat::core::keyword/to-string fqdn)))
         (:wat::holon::Atom v)))
     (:wat::core::defn ~(... derived predicate fqdn ...) [v <- :wat::holon::HolonAST] -> :wat::core::bool
       (:wat::holon::is? v ~(:wat::core::keyword/to-string fqdn)))))
```

### Substrate primitives available

- `:wat::core::defmacro` with optional `& rest` for variadic params (arc 150)
- `:wat::core::quasiquote` + `:wat::core::unquote` + `:wat::core::splice` (`~@`)
- `:wat::core::keyword/to-string` + `:wat::core::keyword/from-string`
- `:wat::core::string::split` + `string::join` + `string::concat`
- `:wat::core::Vector/length` / `last` / `take` / `nth` / `map`
- `:wat::core::Option/expect`
- arc 228 `:wat::holon::Bind` / `:wat::holon::Bundle` / `:wat::holon::Atom`
- arc 230 classifier-wrap composition: instance shape `Bind(Atom("ClassName"), Bundle(Bind(Atom("field1"), Atom(val1)), ...))`
- arc 228 `extract_classifier` + `extract_classifier_inner_bundle` (substrate helpers; accessor extraction may need to walk Bundle for the matching field-name Bind)

### Surface design (locked per dialogue)

**Two coexisting forms:**

```
;; Form 1 — single-data (Stone 227.1b shipped; UNCHANGED)
(:wat::holon::defrecord :myapp::Foo)
  ;; constructor takes 1 HolonAST payload (opaque)
  ;; predicate :myapp::is-Foo?
  ;; no accessors (single payload; use from-holon)

;; Form 2 — multi-field (Stone 227.2 NEW)
(:wat::holon::defrecord :myapp::Voltage
  [magnitude <- :wat::core::f64
   unit      <- :wat::core::String])
  ;; constructor takes N typed args (magnitude, unit)
  ;; predicate :myapp::is-Voltage?
  ;; accessors :myapp::Voltage/magnitude, :myapp::Voltage/unit
```

**Field-list syntax:** `[name <- :Type ...]` — matches defservice's typed-binder convention (`user <- :counter::User`). Bare-symbol field names; FQDN types after `<-`.

**Macro dispatch by arity:**
- 1-arg → single-data form (existing behavior)
- 2-arg → multi-field form (new)

**Instance shape for multi-field** (per arc 230 classifier-wrap):

```
Bind(Atom("myapp::Voltage"),
  Bundle(
    Bind(Atom("magnitude"), Atom(<f64-value>)),
    Bind(Atom("unit"),      Atom(<String-value>))))
```

The classifier is the FQDN; the inner Bundle holds named-field Binds. Field-name = bare symbol from declaration; field-value = the typed value lifted to HolonAST via `:wat::holon::to-holon`.

**Accessor synthesis** — for each field:

```
(:wat::core::defn :myapp::Voltage/magnitude
  [v <- :myapp::Voltage] -> :wat::holon::HolonAST
  ;; extract the inner Bundle from v's classifier-wrap,
  ;; find the Bind with classifier-atom "magnitude",
  ;; return its inner Atom contents (raw HolonAST)
  ...)
```

Accessor returns `:wat::holon::HolonAST` (the raw inner Atom contents). Caller uses `:wat::holon::from-holon` if they want it back as a primitive.

(Alternative: accessor returns the typed primitive directly. Decide via four-questions during sonnet flight. The HolonAST-returning version is the conservative honest baseline; primitive-returning is the ergonomic upgrade. Pick the version that compiles cleanly and reads well; document choice in SCORE.)

## Your scope (sonnet)

### Phase 1 — Extend macro head + dispatch

Edit `wat/holon/defrecord.wat`:
- Macro now accepts 1 or 2 args (variadic `& rest` OR explicit 2-arity overload — pick whichever wat's defmacro supports cleanly)
- 1-arg path: existing behavior unchanged
- 2-arg path: extract field-list; synthesize multi-field constructor + accessors

Investigate via grep how existing variadic defmacros pattern this (e.g., `wat/runtime.wat` define-alias). Mirror that shape.

### Phase 2 — Multi-field constructor synthesis

Constructor takes N args (one per field, typed). Expands to:

```
(:wat::core::defn ~fqdn [arg1 <- :Type1, arg2 <- :Type2, ...]
                        -> :wat::holon::HolonAST
  (:wat::holon::Bind
    (:wat::holon::Atom (:wat::holon::to-holon ~classifier-str))
    (:wat::holon::Bundle
      (:wat::holon::Bind (:wat::holon::Atom (:wat::holon::to-holon "field1"))
                         (:wat::holon::Atom (:wat::holon::to-holon arg1)))
      (:wat::holon::Bind (:wat::holon::Atom (:wat::holon::to-holon "field2"))
                         (:wat::holon::Atom (:wat::holon::to-holon arg2)))
      ...)))
```

Use `:wat::holon::to-holon` to lift each typed arg to HolonAST before wrapping in Atom.

### Phase 3 — Accessor synthesis (one per field)

For each `[name <- :Type]` declaration, generate:

```
(:wat::core::defn ~(keyword/of fqdn "/" "field-name") [v <- ~fqdn] -> :wat::holon::HolonAST
  (... walk inner Bundle; find Bind with classifier-atom "field-name"; return inner Atom contents ...))
```

The body needs to:
1. Extract the inner Bundle from v (via `extract_classifier_inner_bundle` or equivalent)
2. Iterate Bundle items; find the one whose outer atom matches the field-name string
3. Return that Bind's inner contents (raw HolonAST)

**If substrate-level Bundle-walking primitives aren't ergonomic** — surface as STOP-5b (need new primitive OR macro should be more conservative). Use `:wat::holon::Bundle/children` + `is?` + classifier extraction to walk.

### Phase 4 — Predicate (unchanged from 227.1b)

```
(:wat::core::defn :myapp::is-Voltage? [v <- :wat::holon::HolonAST] -> :wat::core::bool
  (:wat::holon::is? v "myapp::Voltage"))
```

Same as single-data form. Both shapes share the predicate logic.

### Phase 5 — Tests

Extend `tests/probe_arc227_stone1_defrecord.rs` OR create a sibling file `tests/probe_arc227_stone2_defrecord_multifield.rs` (sonnet picks; sibling is cleaner):

**Test categories** (8+ tests):
1. **Single-data form still works** (backward compat — Stone 227.1b shape)
2. **Multi-field construct + accessor read** — single field
3. **Multi-field construct + accessor read** — multiple fields
4. **Predicate works on multi-field instance**
5. **Cross-namespace independence** for multi-field (appA::Voltage vs appB::Voltage)
6. **Accessor returns raw HolonAST** (or typed primitive — document the choice)
7. **Constructor type-checks each field** — wrong type → check error
8. **Empty field-list `[]`** behavior — error or treat as single-data? (Pick + document.)

### Phase 6 — Verification

Run each ONE AT A TIME, foreground:

```
cargo build --release -p wat
cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2
cargo test --release --test probe_arc227_stone1_defrecord
cargo test --release --test probe_arc227_stone2_defrecord_multifield     # if sibling created
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

**Write `wat-rs/docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.2.md`** mirroring SCORE-STONE-227.1b.md shape; document accessor-return-type choice + empty-list behavior.

## STOP triggers

- **STOP-1 (compile error UNEXPECTED):** STOP and report
- **STOP-2 (test failure beyond new probe):** STOP + diagnose
- **STOP-3 (180 min elapsed):** wall-clock STOP
- **STOP-4 (holon-rs touched accidentally):** STOP and report
- **STOP-5 (substrate-primitive route taken when wat-defmacro works):** STOP — defrecord stays pure macro expansion
- **STOP-5b (substrate lacks ergonomic Bundle-walking primitive):** if accessor body cannot be expressed without proposing new substrate primitives, STOP and surface as finding; orchestrator decides whether to mint helpers OR defer this stone
- **STOP-6 (methods bundled in defrecord):** STOP — per `STONE-227.2-NOTES.md` Pattern 3, methods stay separate defns. Field-list is the only addition.
- **STOP-7 (bash discipline):** cargo hang from pipes
- **STOP-8 (backward compat broken):** STOP if Stone 227.1b's single-arg form stops working. All Stone 227.1b probe tests must continue to pass.

## Out-of-scope

- Methods bundled in defrecord (STOP-6; per `STONE-227.2-NOTES.md`)
- Inheritance via classifier-chain (Stone 227.3)
- `:with-<field>` setters that return new instance with one field replaced (Stone 227.4 if requested)
- `:invariants` predicates on construction (future enhancement)
- defprotocol / extend-type (arc 232)
- from-holon support for multi-field structs returning typed Tuple/HashMap (future stone — accessors are the v1 access path)
- holon-rs / wat-edn changes
- Aliases (HARD CUT)

## Doctrine context

Stone 227.2 ships the ergonomic upgrade defrecord needs to be useful:

```
Stone 227.1 v3 ✓  single-data newtype (opaque payload)
Stone 227.1b ✓    rename defclass → defrecord (honest name)
Stone 227.2       multi-field structs + auto-accessors (THIS STONE)
Stone 227.3?      inheritance via classifier-chain (when needed)
Stone 227.4?      INSCRIPTION (closes arc 227)
```

defrecord becomes Clojure-defrecord-comparable: declare data shape; substrate generates the boilerplate; methods are separate defns; immutable updates construct new instances; type-checking via classifier-similarity (arc 226).

Per `project_defrecord_defservice_doctrine` — defrecord is the immutable-data abstraction; defservice is the mutex-around-mutable-state abstraction. This stone advances defrecord's surface; defservice (arc 209) stays its own arc.
