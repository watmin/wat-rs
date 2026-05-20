# Arc 214 — Parser pivot Stone P1 — `:wat::core::HashMap` constructor: Vector-symmetric refactor

## Mission

Refactor the existing `:wat::core::HashMap` variadic constructor from its `:(K,V)` tuple-type-keyword shape to a `:K :V` two-separate-keywords shape that mirrors `:wat::core::Vector :T x0 x1 ...`. Verb-equals-type pattern uniform across collection constructors (per arc 109 slice 1f).

**Pre-spawn substrate-truth (verified by orchestrator dig 2026-05-20):**

- The constructor EXISTS. `eval_hashmap_ctor` at `src/runtime.rs:8848`; `infer_hashmap_constructor` at `src/check.rs:10564`; type-scheme registration at `src/check.rs:15557-15565`.
- Current call shape: `(:wat::core::HashMap :(K,V) k0 v0 k1 v1 ...)` — first arg is a tuple-type-keyword wrapping both types.
- Error message verbatim (runtime.rs:8848 body): *"first argument must be a tuple type keyword :(K,V)"*.
- ZERO downstream callers across wat/, wat-tests/, crates/*/wat/. Refactor is pure surface clean-up; no migration cascade.
- The TypeScheme registration ALREADY has `type_params: vec!["K".into(), "V".into()]` + `params: vec![k_var(), v_var()]` — the signature anticipates two type-args; only the runtime + check evaluators need refactoring.

**Why this refactor (one paragraph):**

The substrate established **verb-equals-type** as the canonical collection constructor pattern (arc 109 slice 1f). `Vector` takes ONE type-arg (`:T`); HashMap conceptually takes TWO (`:K`, `:V`). The natural mirror is two separate keywords, parallel with Vector. The existing `:(K,V)` tuple-keyword form packs both type-args into one wat token via the tuple-type-keyword convention — wat-idiomatic but DIFFERENT shape from Vector's. Per `feedback_options_are_tangle` + `project_wat_llm_first_design` (one canonical path per task at the design-grammar layer): readers from arc 109 + Vector experience expect the symmetric form; the tuple-keyword wrapping is wat-correct but adds a layer the symmetry doesn't need.

Zero downstream callers means this refactor ships as pure surface clean-up. Same shape as Slice 2 forward-correction (bounded factory had zero callers; clean retirement → mini-TCP foundation).

## Substrate context (substrate-truth verified pre-spawn)

- **`src/runtime.rs:8848`** — `fn eval_hashmap_ctor(args, list_span, env, sym)`. Current body parses `args[0]` as a single keyword (the tuple-type-keyword `:(K,V)`), then `args[1..]` as alternating key/value pairs (even count). Reformat to parse `args[0]` as K type-keyword AND `args[1]` as V type-keyword, then `args[2..]` as alternating pairs.
- **`src/check.rs:10564`** — `fn infer_hashmap_constructor`. Type-checker that pairs with the runtime constructor. Refactor to expect `:K :V` as first two args.
- **`src/check.rs:15557-15565`** — `env.register(":wat::core::HashMap", TypeScheme { type_params: vec!["K".into(), "V".into()], params: vec![k_var(), v_var()], ret: hashmap_of(k_var(), v_var()), ... })`. The TypeScheme is ALREADY shaped for two type-args; no change needed here. Update the doc-comment to drop the `:(K,V)` tuple-keyword note (currently lines 15550-15556 say "accepts a leading `:(K,V)` tuple-keyword followed by alternating key/value pairs").
- **`src/runtime.rs:4492`** — dispatch arm `":wat::core::HashMap" => eval_hashmap_ctor(...)`. No change; the verb name dispatches identically.
- **Vector reference (the mirror pattern)** — `eval_list_ctor` at `src/runtime.rs:4437` handles `:wat::core::Vector :T x0 x1 ...` with a SINGLE type-keyword. Model HashMap's new shape's arity + type-arg parsing after Vector's, then extend to TWO type-keywords + pairs.
- **Test fixture locations** — search `tests/*.rs` and `wat-tests/**/*.wat` for any references to HashMap's old `:(K,V)` form. Pre-spawn grep returned zero in production wat; verify zero in tests too. If tests exist exercising the old form, they update to the new form OR retire if their premise was specifically tuple-keyword.

## Concrete deliverables

### 1. Refactor `eval_hashmap_ctor` (runtime.rs:8848)

Parse `args[0]` as `:K` type-keyword AND `args[1]` as `:V` type-keyword; `args[2..]` as alternating pairs (even count). Update arity check (was 1+; becomes 2+). Update error messages to reference the new shape:

- *"first two arguments must be type keywords (K, V); got <something>"* — when args[0] or args[1] aren't keywords
- *"arity after :K :V type args must be even (alternating key/value pairs); got <n>"* — when remaining count is odd

### 2. Refactor `infer_hashmap_constructor` (check.rs:10564)

Update type-checker to expect `:K :V` as the first two positional args. Validate K and V are valid type-keywords (per the existing type-keyword recognition machinery). Validate each key in pairs has type K; each value has type V.

### 3. Update doc-comments in check.rs:15550-15556

Replace the existing block:

```rust
// :wat::core::HashMap — variadic at runtime (accepts a leading
// `:(K,V)` tuple-keyword followed by alternating key/value
// pairs; infer_hashmap_constructor at check.rs:7904). The
// fingerprint registers a 2-arg `:K, :V` sentinel since
// TypeScheme has no variadic shape today AND no
// tuple-type-keyword shape. Real shape checking lives in the
// handler. Per arc 144 slice 3 limitation.
```

With:

```rust
// :wat::core::HashMap — variadic constructor at runtime (accepts
// `:K :V k0 v0 k1 v1 ...`, 2+ args; infer_hashmap_constructor at
// check.rs:<NEW_LINE>). Verb-equals-type per arc 109 slice 1f;
// mirrors :wat::core::Vector :T x0 x1 ... with two type-args
// for HashMap's K + V. The 2-arg fingerprint matches the call
// shape directly. Real arity + pair-parity + per-element checking
// lives in the handler.
```

(Sonnet updates the line number reference to wherever `infer_hashmap_constructor` ends up after the refactor.)

### 4. Probe tests

New probe file `tests/probe_hashmap_ctor_vector_symmetric.rs` (or extend an existing probe if one exists for HashMap). Tests:

1. **Empty literal** — `(:wat::core::HashMap :wat::core::Keyword :wat::core::i64)` constructs empty HashMap<Keyword, i64>.
2. **Single pair** — `(:wat::core::HashMap :wat::core::Keyword :wat::core::i64 :foo 42)` constructs HashMap with one entry; verify length 1; verify get returns 42.
3. **Multi pair** — three or four pairs; verify length matches; verify get for each key.
4. **String-keyed** — `(:wat::core::HashMap :wat::core::String :wat::core::i64 "a" 1 "b" 2)` confirms K can be any type, not just Keyword.
5. **HolonAST-keyed** — `(:wat::core::HashMap :wat::holon::HolonAST :wat::holon::HolonAST (:wat::holon::Atom 42) (:wat::holon::Atom "answer"))` confirms K can be HolonAST.
6. **Wrong-type rejection** — `(:wat::core::HashMap :wat::core::Keyword :wat::core::i64 :foo "not-an-i64")` should fail at type-check with a clear diagnostic.
7. **Odd count rejection** — `(:wat::core::HashMap :wat::core::Keyword :wat::core::i64 :foo)` should fail with the "even count" message.
8. **Missing K type-arg** — `(:wat::core::HashMap)` should fail with the "first two arguments must be type keywords" message.
9. **Missing V type-arg** — `(:wat::core::HashMap :wat::core::Keyword)` should fail (only one type-arg given).

### 5. Update WAT-CHEATSHEET.md

Add a HashMap constructor row alongside Vector in the cheatsheet's constructor section (if such a section exists; otherwise create it). Document:

```
(:wat::core::Vector :T x0 x1 ...)              ;; Vector<T>
(:wat::core::HashMap :K :V k0 v0 k1 v1 ...)    ;; HashMap<K,V>
(:wat::core::HashSet :T x0 x1 ...)             ;; HashSet<T>  (mirror)
(:wat::core::Tuple :T0 :T1 :T2 x0 x1 x2 ...)   ;; Tuple<T0,T1,T2>  (heterogeneous)
```

Brief: "verb-equals-type constructors; first N args are type-keywords (1 for Vector/HashSet; 2 for HashMap; per-element for Tuple), remaining args are values."

### 6. arc 058 changelog row

Add to `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md` a row noting:

```
| 2026-05-20 | arc 214 P1 | refactor | :wat::core::HashMap constructor: :(K,V) tuple-keyword shape retired; :K :V two-separate-keywords shape (Vector-symmetric per arc 109 slice 1f) |
```

(Format per the file's existing pattern.)

### 7. SCORE doc

`docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-PARSER-PIVOT-P1-HASHMAP-CTOR-VECTOR-SYMMETRIC.md` — score against the scorecard in EXPECTATIONS; include verification command output.

## Verification commands (run yourself; include output in SCORE)

```bash
# Build clean
cargo build --release

# Run new probe
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat

# Workspace baseline preserved
cargo test --release --workspace --no-fail-fast

# Confirm zero references to old :(K,V) shape anywhere
grep -rn "HashMap :(.*)" --include="*.rs" --include="*.wat"
# (should return zero matches after refactor)

# Confirm error messages updated
grep -rn "tuple type keyword" --include="*.rs"
# (should return zero matches after refactor; the old error string retires)
```

## STOP triggers

- **If grep reveals production wat using the old `(:wat::core::HashMap :(K,V) ...)` shape** → STOP and report; pre-spawn grep returned zero; investigation needed before refactor.
- **If existing tests in `tests/` exercise the old `:(K,V)` form** → STOP and report; the tests need either migration to the new shape OR retirement (if their premise was specifically the tuple-keyword form).
- **If `infer_hashmap_constructor` has shared machinery with other constructors that you'd be touching** → STOP and report; refactor scope expanded beyond pure HashMap.
- **If the `hashmap_key` helper at runtime.rs:8884 has assumptions about the tuple-keyword shape** → STOP and report; broader scope.
- **If workspace baseline regresses after the refactor** → STOP and report what broke; pre-existing failures (1-2 known) should NOT change; new failures need diagnosis.

## Discipline anchors

- `feedback_attack_foundation_cracks` — substrate-grammar asymmetry surfaced; fix immediately
- `feedback_options_are_tangle` — `:(K,V)` packing vs `:K :V` separate IS an option-tangle; collapse to one canonical form
- `feedback_simple_is_uniform_composition` — verb-equals-type across all collection constructors IS simple
- `feedback_substrate_owns_not_callers_match` — substrate owns the constructor shape; callers don't need to learn two patterns
- `feedback_inscription_immutable` — original `:(K,V)` shape inscribed in commit history; refactor is forward-correction
- `feedback_four_questions_inline` — verdict ran inline in conversation; sonnet executes the verdict
- `project_wat_llm_first_design` — LLMs from Clojure/Vector experience expect this symmetry
- arc 109 slice 1f — verb-equals-type discipline established
- Per the kernel impeccability protocol (INTERSTITIAL § "2026-05-19 — Kernel impeccability via ward pass"): 9-ward parallel pass after sonnet ships.

## Out of scope (do NOT touch)

- **`{...}` map literal in expression position** — Stone P2 (task #404). This stone is the verb-form refactor ONLY.
- **Vector / HashSet / Tuple constructors** — all already in Vector-symmetric form; no changes.
- **`hashmap_key` helper (runtime.rs:8884)** — pure key-coercion utility; unchanged.
- **wat-side surface** (`wat/core.wat`'s HashMap/length, /get, etc.) — query/mutation verbs; unchanged.
- **DESIGN.md for arc 214** — sonnet doesn't touch the realization-narrative portions of DESIGN.md. If DESIGN's signature reference for HashMap needs updating, sonnet can update the SIGNATURE block only; orchestrator handles any prose that discusses the convergence-with-self pattern post-ship.
- **INTERSTITIAL entry for the parser-pivot direction** — orchestrator-direct post-ship per `feedback_sonnet_no_realization_voice`. Sonnet does NOT write INTERSTITIAL entries.

## Time budget

- Substrate refactor (eval + check + register doc): 10-15 min
- Probe tests (9 tests): 10-15 min
- WAT-CHEATSHEET update: 3-5 min
- arc 058 row: 2-3 min
- SCORE doc: 5-10 min
- **Total: 30-50 min Mode A**

If sonnet runs > 75 min: STOP via wakeup; report progress; orchestrator decides continue vs Mode B-time-violation.

## What this stone enables

After P1 closes + ward-passes:

- `:wat::core::HashMap` is callable from wat code in the verb-equals-type symmetric form
- Stone P2 (the `{...}` literal in expression position) expands directly to P1's verb-form — no wat-side macro layer needed
- ProgramEnv design from Slice 4 prep lands on this foundation: `{:client-key k :remote-url u}` → `(:wat::core::HashMap :wat::core::Keyword :wat::holon::HolonAST :client-key (:wat::holon::Atom k) :remote-url (:wat::holon::Atom u))`
- Vector-symmetric mental model holds across the substrate's collection constructors
- The dig discipline payoff is real — four rounds of substrate-already-sufficient finds led here; the convergence-with-self pattern (Convergence #12) operating at the conversation layer
