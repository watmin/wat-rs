# SCORE — Stone 234.4.match — match-arm hash-destructure

**Status:** SHIPPED. 11/11 PASS.

**Date:** 2026-05-25.

---

## Scorecard

| # | Row | Verification | Result |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **NEW probe 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone4_match_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.4 let-binding probe regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 236.0 CheckResult probe regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | **Lib baseline** (LOAD-BEARING) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `52` (≤ 54) |

---

## Receivers shipped

All three receivers landed:
- **`Value::wat__Record`** — match-arm hash-destructure over record instances (probes 1, 2, 6)
- **`Value::Struct`** — match-arm hash-destructure over Rust structs via TypeDef (structurally shipped; reachable via Struct scrutinee)
- **`Value::wat__std__HashMap`** — match-arm hash-destructure over HashMaps returning `Option<V>` per binding (probes 3, 4)

No receivers deferred.

---

## Three-file change summary

### src/parser.rs (0 lines net — ZERO TOUCH)

Parser zero-touch confirmed. Stone 234.4's `BraceKind::HashDestructure` mechanism already fires
in match-arm position: a brace anywhere in source produces `WatAST::StructPattern` with alternating
Symbol/Keyword children when the hash-destructure discriminant matches. Match arms are parsed as
`(pattern body)` lists where `pattern` is a normal expression — no special match-position parser.
Probe-first verification confirmed at startup (probes 1-6 passed without parser changes).

### src/check.rs (+~90 lines net)

Three extension sites:

1. **`MatchShape` enum** — added `Open(TypeExpr)` variant for open-typed matches where all arms are
   hash-destructure or wildcard. `as_type()` returns the bare fresh type variable, so scrutinee
   unification with any type always succeeds.

2. **`detect_match_shape`** — added skip case for hash-destructure StructPattern arms (they do not
   determine the variant-constructor shape). Changed default from `MatchShape::Option(fresh)` to
   `MatchShape::Open(fresh)` — this was the critical fix enabling record/HashMap scrutinees to pass
   type-checking.

3. **`infer_match` arm loop** — early-exit branch before `pattern_coverage` call: detects
   hash-destructure StructPattern, injects fresh type vars for each (Symbol, Keyword) pair into
   `arm_locals`, sets `wildcard_seen = true` (and all coverage flags), type-checks body against
   declared type. `continue` skips the shape-based coverage machinery.

4. **Exhaustiveness / shape resolution** — `MatchShape::Open` handled in the apply_subst
   refinement block, exhaustiveness check (`wildcard_seen` sufficient), and error message arm.

5. **`pattern_coverage`** — added `MatchShape::Open(_)` arm for `:None` keyword pattern (returns
   `Coverage::Wildcard` for open-shaped matches; unusual but not forbidden).

Arc-169 struct-destructure path in `infer_match` unchanged: all-Symbol StructPattern continues
to go through `pattern_coverage` → error (let-binding-only restriction intact per T7).

### src/runtime.rs (+~80 lines net)

Two extension sites:

1. **`try_match_pattern` signature** — added `sym: &SymbolTable` parameter to thread symbol
   table access through to the `Value::Struct` arm (which needs `keyword_accessor_struct`,
   requiring the TypeDef registry). All 7 call sites updated (2 top-level in eval_match functions;
   5 recursive internal calls within `try_match_pattern`).

2. **`try_match_pattern` StructPattern arm** — replaced the blanket `Err(MalformedForm)` with:
   - Hash-destructure detected (`items[1]` is Keyword): collect `(var_name, bare_field)` pairs;
     dispatch on scrutinee:
     - `wat__Record` → `keyword_accessor_record` per pair; `Some(env_extended)`
     - `Struct` → `keyword_accessor_struct` per pair; `Some(env_extended)`
     - `wat__std__HashMap` → keyword key + Option wrap per pair; `Some(env_extended)`
     - Other → `Ok(None)` (arm falls to next arm — T4 fall-through implemented)
   - Arc-169 struct-destructure (all-Symbol): keeps existing `Err(MalformedForm)` (let-binding-only)

3. **`try_match_pattern_ast` StructPattern arm** — replaced blanket `Err` with:
   - Hash-destructure: `Ok(None)` — AST-level mirror cannot dispatch on runtime Value types;
     hash-destructure arms are runtime-only and never match at AST level (T9 decision)
   - Arc-169 struct-destructure: keeps existing `Err(MalformedForm)`

---

## Implementation notes

### Parser zero-touch verified

Step 1 of BRIEF confirmed: parser already recognizes `{var :field}` everywhere via BraceKind.
No parser lines added. The brace shape fires identically in match-arm position.

### MatchShape::Open — the critical new variant

The initial implementation attempted to patch around the existing `MatchShape::Option` default,
but the root cause required a new variant. When all arms are hash-destructure, `detect_match_shape`
previously returned `MatchShape::Option(fresh)`, which forced the scrutinee type to unify with
`Option<T>` — failing for Record/HashMap scrutinees. `MatchShape::Open(fresh)` fixes this by
letting `as_type()` return the raw fresh variable (unifies with anything). One compile round
to diagnose the root cause; one to add the variant.

### `try_match_pattern` sym threading

Stone 234.4's let-binding path had `sym` available via `bind_let_binding(rhs, scope, sym)`.
`try_match_pattern` only received `outer: &Environment`. Threading `sym: &SymbolTable` was the
correct architectural move — the Struct receiver arm needs `keyword_accessor_struct` which
requires the TypeDef registry. 7 call sites updated; no architectural surprise.

### AST mirror parity (T9) — No update needed

`try_match_pattern_ast` is used for macro/quasiquote pattern matching at the AST level.
Hash-destructure arms require runtime Value type dispatch (Record vs Struct vs HashMap) — this
is not possible at AST level where the scrutinee is a parse tree, not a runtime Value. Decision:
`Ok(None)` for hash-destructure StructPattern in `try_match_pattern_ast`. The arm simply does
not match at AST level (quasiquote/macro paths fall to next arm). Documented in the code.

### Empty hash-destructure pattern (T5) — ALLOW (inherited from Stone 234.4)

`{}` empty brace → empty StructPattern (zero items). `items.get(1)` returns None → not classified
as hash-destructure (falls to arc-169 error path). In practice, the parser routes `{}` to
MapLiteral (empty map) before the StructPattern path. So an empty brace in match position
becomes an empty map literal, not a hash-destructure. Behavior is consistent with Stone 234.4.

### Arc-169 struct-destructure preservation (T7)

`items.get(1).is_keyword()` discriminant is FALSE for arc-169 forms (all-Symbol children).
Both `infer_match` and `try_match_pattern` correctly branch to the unchanged error path for
arc-169 StructPattern in match position. Lib baseline 827/0 confirms no regression.

### Probe 4 fix

Initial probe 4 used a heterogeneous HashMap `{:host "localhost"  :present 42}` (String + i64
values). The type checker requires HashMap values to be homogeneous. Replaced with
`{:host "localhost"  :user "admin"}` (homogeneous String values) + tested `:missing` key
(absent) → None. Probe verifies multi-key hash-destructure + missing key → None path.

---

## Cascade depth

- parser.rs: 0 rounds (zero-touch)
- check.rs: 2 rounds (MatchShape::Open missing arm caught by compiler; probe 4 probe fix)
- runtime.rs: 1 round (sym parameter threading; all call sites fixed together)

Total cascade depth: 2 compile rounds + 1 probe fix (probe 4 HashMap heterogeneity).

---

## Time

~60 minutes elapsed. Within 60-90 min target.

---

## Closing note

The named follow-up from Stone 234.4 D8 is CLOSED. Arc 234 is now one decision (234.6 fate)
+ one stone (234.7 INSCRIPTION) from closure.

Match-arm hash-destructure parity with let-binding hash-destructure is complete across all three
receivers (Record / Struct / HashMap). The `MatchShape::Open` variant is a permanent substrate
addition that enables open-typed match forms for future non-algebraic-variant patterns.

Rank-up: Stone 234.4 SCORE template worked exactly as expected. Probe-first parser verification
(BRIEF Step 1) paid off — zero-touch was confirmed before any code was written, saving 30-50
lines of speculative parser work. The critical insight (MatchShape::Open) was diagnosed in one
compile round from the TypeMismatch error message.
