# BRIEF — Arc 227 Stone 227.2 v2 — Mandate field-list on defrecord (RETIRE single-arg form)

**Stone scope:** Unify `:wat::holon::defrecord` under ONE canonical shape — **always takes a field-list `[fields]`** as second argument. Field-list can be empty `[]` (tagged unit) OR contain N `[name <- :Type]` entries (N-field struct). Auto-generates N-arg constructor + N accessors `:ns::Type/<field>` + predicate. Retires Stone 227.1b's single-arg form (HARD CUT). Migrates 18 existing probes.

**Type:** Sonnet Mode A.
**Time budget:** 90-180 min target; 240 min STOP.
**Depends on:** Stone 227.1b SHIPPED (commit `aa2b9f1`); defrecord macro exists at `wat/holon/defrecord.wat`.
**Calibration:** Bigger scope than 227.1b (~5 min for rename) but smaller than 227.1 v3 original mint (~18 min) — macro is more uniform; probe migration is mechanical sed-able work.

## v2 supersedes v1 (orchestrator stop+reframe)

The v1 BRIEF (committed at `2162d82`) proposed TWO defrecord forms:
- 1-arg `(defrecord :ns::Foo)` — single-data (Stone 227.1b shipped)
- 2-arg `(defrecord :ns::Foo [fields])` — new multi-field

User pushback 2026-05-22 night: *"i don't know if i like having options ..... i think forcing the empty vec is best?...."*

Four-questions atomic check on optional-args:
- **Obvious?** NO — two shapes per verb; readers must learn arity-dispatch
- **Simple?** NO — two paths; macro branches internally
- **Honest?** NO — implies defrecord has two kinds when really we're accommodating v1
- **Good UX?** Worse per `feedback_wat_llm_first_design` — LLM-first design rejects synonym features; one canonical path

YES YES YES YES (one shape) wins decisively. v2 mandates the field-list. **HARD CUT — single-arg form retired.**

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`**
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- DO NOT commit. DO NOT touch holon-rs. DO NOT touch wat-edn.
- **HARD CUT** discipline — single-arg form retired; no aliases.

## BASH DISCIPLINE

- ONE cargo command at a time, foreground; no piping; no concurrent runs
- 5 known signal-handler test hangs (task #413) — skip per Verification

## Doctrine context

Per `project_defrecord_defservice_doctrine` (inscribed `72a7ad5`): defrecord wraps immutable data. Per `feedback_wat_llm_first_design`: one canonical path per task; reject synonym features; the path of least resistance IS the path we want.

**defrecord IS a fields-struct.** Zero fields is honestly a tagged unit (`Bind(Atom("classname"), Bundle())`). One field is a wrapped-payload. N fields is a struct. ALL share the same shape.

## The canonical form (locked)

```
(:wat::holon::defrecord :myapp::Voltage
  [magnitude <- :wat::core::f64
   unit      <- :wat::core::String])
```

Macro head: `(defrecord <fqdn> <field-list>)` — always 2-arg.

**Three field-count cases, ONE shape:**

| Field-list | Constructor signature | Accessors | Instance shape |
|---|---|---|---|
| `[]` | `(:ns::T)` zero-arg | none | `Bind(Atom("ns::T"), Bundle())` |
| `[v <- :T1]` | `(:ns::T v)` one-arg | `:ns::T/v` | `Bind(Atom("ns::T"), Bundle(Bind(Atom("v"), Atom(<v-val>))))` |
| `[a <- :T1, b <- :T2]` | `(:ns::T a b)` two-arg | `:ns::T/a`, `:ns::T/b` | `Bind(Atom("ns::T"), Bundle(Bind(Atom("a"), Atom(<a>)), Bind(Atom("b"), Atom(<b>))))` |

Uniform composition (per `feedback_simple_is_uniform_composition`): N identical Bind-of-Atom-Atom pairs in the inner Bundle.

## Pre-flight verified

### Existing defrecord macro (`wat/holon/defrecord.wat`)

Reviewed in full at HEAD `aa2b9f1`. Current shape (single-arg; retiring):

```
(:wat::core::defmacro
  (:wat::holon::defrecord
    (fqdn :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     (:wat::core::defn ~fqdn [v <- :wat::holon::HolonAST] -> :wat::holon::HolonAST
       (:wat::holon::Bind ...))
     (:wat::core::defn ~<predicate-fqdn> [v <- :wat::holon::HolonAST] -> :wat::core::bool
       (:wat::holon::is? v ~classifier-str))))
```

v2 macro becomes 2-arg head, body branches by field-count.

### Existing 18 probes (`tests/probe_arc227_stone1_defrecord.rs`)

All 18 use 1-arg form: `(:wat::holon::defrecord :test::Voltage)` then `(:test::Voltage 5.0)` etc. Each test must migrate to v2 form.

**Migration mapping:**

| Test 227.1b shape | Test 227.2 v2 shape | Rationale |
|---|---|---|
| `(defrecord :test::Voltage)` then `(:test::Voltage 5.0)` | `(defrecord :test::Voltage [value <- :wat::core::f64])` then `(:test::Voltage 5.0)` | Constructor signature now reflects the field; accessor `:test::Voltage/value` becomes available |
| Pure-tag tests (if any — sonnet inspects) | `(defrecord :test::Tag [])` then `(:test::Tag)` | Zero-field tag |

Most existing probes use the opaque-payload pattern → migrate to single-field `[value <- :Type]` form. Sonnet picks the appropriate field name per probe.

### Substrate primitives available

- `:wat::core::defmacro` with fixed-arity head (matches v2's 2-arg shape)
- `:wat::core::quasiquote` + `:wat::core::unquote` + `:wat::core::splice` (`~@`)
- `:wat::core::keyword/to-string` + `:wat::core::keyword/from-string` + `:wat::core::keyword/of`
- `:wat::core::string::split` + `string::join` + `string::concat`
- `:wat::core::Vector/length` / `last` / `take` / `map`
- `:wat::core::Option/expect`
- arc 228 `:wat::holon::Bind` / `:wat::holon::Bundle` / `:wat::holon::Atom`
- arc 228 `extract_classifier_inner_bundle` (substrate helper; used by accessor body)
- arc 226 `:wat::holon::is?`

## Your scope (sonnet)

### Phase 1 — Rewrite macro head + body

Edit `wat/holon/defrecord.wat`:
- Macro head becomes 2-arg: `(:wat::holon::defrecord (fqdn :AST...) (fields :AST...))` — always takes field-list
- Body branches on field-count:
  - **Empty field-list** → zero-arg constructor; no accessors; predicate
  - **N-element field-list** → N-arg constructor; N accessors; predicate

Update header doc-comment to reflect v2 mandate. Note in macro body that single-arg form is RETIRED per Stone 227.2 v2.

### Phase 2 — Constructor synthesis (uniform N-arg)

For field-list of length N:

```
(:wat::core::defn ~fqdn [arg1 <- :Type1, arg2 <- :Type2, ..., argN <- :TypeN]
                        -> :wat::holon::HolonAST
  (:wat::holon::Bind
    (:wat::holon::Atom (:wat::holon::to-holon ~classifier-str))
    (:wat::holon::Bundle
      ~@(map (fn [field-name field-val]
               `(:wat::holon::Bind (:wat::holon::Atom (:wat::holon::to-holon ~field-name-str))
                                   (:wat::holon::Atom (:wat::holon::to-holon ~field-val))))
             field-names field-vals))))
```

For N=0: Bundle has zero children; constructor is zero-arg.

### Phase 3 — Accessor synthesis (one per field; skipped for N=0)

For each field `[name <- :Type]`:

```
(:wat::core::defn ~(:wat::core::keyword/of fqdn "/" name-str)
  [v <- ~fqdn] -> :wat::holon::HolonAST
  ;; extract inner Bundle from v; find Bind matching field-name; return inner contents
  ...)
```

Use `extract_classifier_inner_bundle` + `:wat::holon::Bundle/children` iteration. Find the Bind whose outer Atom matches `name-str`. Return its inner Atom contents (raw HolonAST).

**Accessor return type:** `:wat::holon::HolonAST` (raw inner Atom contents). Caller uses `:wat::holon::from-holon` to recover the typed primitive. Honest baseline — typed-primitive return is future ergonomics.

### Phase 4 — Predicate (unchanged shape from 227.1b)

```
(:wat::core::defn ~<predicate-fqdn> [v <- :wat::holon::HolonAST] -> :wat::core::bool
  (:wat::holon::is? v ~classifier-str))
```

Always generated regardless of field-count.

### Phase 5 — Migrate existing 18 probes

Edit `tests/probe_arc227_stone1_defrecord.rs`:
- Every `(:wat::holon::defrecord :test::Foo)` (1-arg) becomes `(:wat::holon::defrecord :test::Foo [value <- :Type])` where `:Type` matches the data the test passes
- Every `(:test::Foo somedata)` call retains its shape (constructor still takes one arg)
- Predicate tests unchanged (predicate behavior unchanged)

If any probe asserts on the OPAQUE-payload behavior specifically (e.g., asserts the constructor takes HolonAST not typed primitive), that probe needs semantic update.

Rename file to reflect v2 ownership: `git mv tests/probe_arc227_stone1_defrecord.rs tests/probe_arc227_stone2_defrecord.rs` — the v2 stone supersedes v1's tests. (Sonnet's choice if a separate stone1 probe should retire entirely.)

### Phase 6 — Add new tests for v2-specific behavior

Extend the migrated probe (OR new sibling file):
- Multi-field construct + accessor read
- Empty field-list `[]` zero-arg constructor works
- N-field constructor type-checks each arg
- Accessor returns raw HolonAST (document choice)
- Cross-namespace independence with multi-field

Total target ~25+ tests (18 migrated + ~7 v2-specific).

### Phase 7 — Update src/stdlib.rs comment

Line 74 in `src/stdlib.rs` mentions Stone 227.1b. Update to note Stone 227.2 v2 supersedes:

```rust
// Arc 227 Stone 227.2 v2 — :wat::holon::defrecord macro (multi-field shape;
// supersedes 227.1b single-arg form). Mints user-defined classifier-wrapped
// typed entities with named fields.
```

### Phase 8 — Append rename note to SCORE-STONE-227.1b.md

Per `feedback_inscription_immutable`: DO NOT rewrite 227.1b's SCORE body. APPEND a section:

```markdown
## Addendum 2026-05-22 night — Stone 227.2 v2 supersedes (HARD CUT)

Stone 227.2 v2 retires the single-arg form this stone shipped. Per
`feedback_wat_llm_first_design` four-questions check: optional args is a
synonym feature. defrecord now mandates the field-list (possibly empty `[]`).

- Macro signature: `(defrecord :fqdn [fields])` — always 2-arg
- Single-arg `(defrecord :fqdn)` form RETIRED (HARD CUT; no alias)
- Probes migrated to explicit field-list form
- Commit: [TBD by orchestrator]

This SCORE doc's body above remains unchanged as historical record per
`feedback_inscription_immutable`.
```

### Phase 9 — Verification

Run each ONE AT A TIME, foreground:

```
cargo build --release -p wat
cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2
cargo test --release --test probe_arc227_stone2_defrecord       # or whatever the migrated file is named
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

**Post-stone grep verification:** `grep -rn "defrecord :[^\s]* *)$" --include="*.wat" --include="*.rs" .` should return ZERO matches (no bare 1-arg defrecord calls).

**Write `wat-rs/docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.2.md`** mirroring SCORE-STONE-227.1b.md shape.

## STOP triggers

- **STOP-1 (compile error UNEXPECTED):** STOP and report
- **STOP-2 (test failure beyond migrated probes):** STOP + diagnose; broken-by-this-stone framing per Stone 221.3 Delta 1a
- **STOP-3 (240 min elapsed):** wall-clock STOP
- **STOP-4 (holon-rs touched accidentally):** STOP and report
- **STOP-5 (substrate-primitive route):** STOP — defrecord stays pure macro
- **STOP-5b (substrate lacks ergonomic Bundle-walking):** if accessor body cannot be expressed in pure wat, STOP and surface — orchestrator decides whether to mint helpers OR defer
- **STOP-6 (methods bundled):** STOP — per `STONE-227.2-NOTES.md` Pattern 3, defrecord NEVER bundles methods
- **STOP-7 (bash discipline):** cargo hang from pipes
- **STOP-8 (1-arg form retained as alias):** STOP — HARD CUT; no `(defrecord :fqdn)` alias for `(defrecord :fqdn [])` or similar; users WRITE the field-list
- **STOP-9 (historical artifact rewritten):** STOP — BRIEF/EXPECTATIONS of Stone 227.1b stay intact; SCORE-227.1b body unchanged (append-only via Phase 8 addendum)

## Out-of-scope

- Methods bundled in defrecord (STOP-6; methods stay separate defns per notes Pattern 3)
- Inheritance via classifier-chain (Stone 227.3)
- `:with-<field>` immutable setters (future)
- `:invariants` (future)
- defprotocol / extend-type (arc 232)
- from-holon support for multi-field structs returning typed Tuple (future)
- holon-rs / wat-edn changes
- Aliases (HARD CUT)

## Doctrine context

Stone 227.2 v2 unifies defrecord under one canonical shape:

```
Stone 227.1 v3 ✓  defclass macro (historical name; superseded by 227.1b)
Stone 227.1b ✓    rename to defrecord (semantics unchanged from v3 — single-arg form)
Stone 227.2 v2    field-list mandate + multi-field + accessors (THIS — retires single-arg)
Stone 227.3?      inheritance via classifier-chain (when needed)
Stone 227.4?      INSCRIPTION (closes arc 227)
```

Per `feedback_wat_llm_first_design`: one canonical path; reject synonym features; engineered pedagogy for AI co-authors. The mandate IS the design.

User direction: *"i am engineering this for models like yourself"* — the bias toward forcing the empty vec IS LLM-first discipline firing. Stone 227.2 v2 honors it.
