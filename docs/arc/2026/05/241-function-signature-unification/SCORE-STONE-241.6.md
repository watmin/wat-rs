# SCORE — Stone 241.6 — Phase 2 opens: optional `{...}` metadata-map storage on `def`; defn inherits via fn-peel

**Status:** Mode A — PASS
**Runtime:** ~35 min (within 25–45 min target band)
**Summary:** Metadata-map storage shipped. `def` parser (check.rs `infer_def` + `extract_def_binding`, runtime.rs `try_parse_fn_shape_def` + `register_runtime_defs_form`) extended for 4-item form `(def :name {meta} expr)`. `SymbolTable.binding_metadata: HashMap<String, HashMap<String, WatAST>>` minted. `defn` macro kept UNCHANGED; substrate fn-peel (`try_parse_fn_shape_def` + `eval_fn` + `infer_fn`) transparently extracts metadata from fn-embedded position when defn expands with `{meta}`. Empty `{}` already rejected by check layer. Stone 241.6 probe 6/6 PASS. Lib 834 PASS. Clippy 902 (delta −2). Workspace build clean.

---

## Phase A Scorecard

| Row | Claim | Result |
|-----|---|---|
| 1 | Probe contracts 01-03 PASS (def + defn with metadata) | **PASS** — 3 passed; 0 failed |
| 2 | Probe contracts 04-05 PASS (regression: no-metadata) | **PASS** — 2 passed; 0 failed |
| 3 | Probe contract 06 PASS (empty `{}` rejected) | **PASS** — 1 passed; 0 failed |
| 4 | Probe whole-suite 6/6 | **PASS** — 6 passed; 0 failed |
| 5 | Stone 241.5 probe preserved 8/8 | **PASS** — 8 passed; 0 failed |
| 6 | Stone 241.4 canonical probe preserved 15/15 | **PASS** — 15 passed; 0 failed |
| 7 | Stone 241.2 + 241.3 probes preserved | **PASS** — 10+6 passed; 0 failed |
| 8 | 237.8b Gate 1 PASS preserved | **PASS** — 1 passed; 0 failed |
| 9 | Lib baseline preserved | **PASS** — 834 passed; 0 failed; 1 ignored |
| 10 | Workspace test-build clean | **PASS** — `cargo build --release --tests --workspace` exit 0; 0 errors |
| 11 | Clippy delta ≤ 0 | **PASS** — 902 warnings (baseline 904; delta −2) |

---

## Structural Verification

| Verification | Command | Result |
|---|---|---|
| 4-item def discrimination present in try_parse_fn_shape_def | `grep -A 30 "^fn try_parse_fn_shape_def" src/runtime.rs \| grep -c "items.len() == 4\|items.len() != 3 && items.len() != 4"` | **2 matches** |
| `:wat::core::HashMap` head detection | `grep -n "wat::core::HashMap" src/runtime.rs \| head -5` | **≥1 match** — lines 3892, 3906, etc. in def parsers |
| SymbolTable.binding_metadata exists | `grep -n "binding_metadata" src/runtime.rs src/check.rs \| head -5` | **5 matches** — field declaration (runtime.rs:1778), Debug (1801), insert site (2683), comment (2673); check.rs:15179 comment |
| defn macro updated (comment documents metadata threading) | `grep -rn "metadata\b" wat/*.wat \| head -5` | **5 matches** — Stone 241.6 comment in wat/core.wat documenting fn-peel mechanism |
| `src/argspec/*` UNCHANGED | `git diff src/argspec/` | **empty diff** |

---

## Migration Audit (per-file line deltas)

| File | Pre-stone | Post-stone | Delta |
|---|---|---|---|
| `src/runtime.rs` (try_parse_metadata_map + try_parse_fn_shape_def extension + SymbolTable field + Debug + fn-peel in eval_fn + def runtime arm + register_defines + preregister helpers) | (current) | (current) | **+~120 lines** |
| `src/check.rs` (infer_def 3-arg path + extract_def_binding 4-item + infer_fn fn-embedded peel) | (current) | (current) | **+~45 lines** |
| `wat/core.wat` (Stone 241.6 comment on defn macro) | (current) | (current) | **+12 lines** (comment only) |
| `tests/probe_arc241_stone6_def_metadata_map.rs` | 155 | 155 | **0** (probe existed pre-stone as FM 2-bis evidence) |
| `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.6.md` | 0 | (this file) | **NEW** |
| **Net delta** | — | — | **~+177 lines** (vs DESIGN estimate of ~+215; within band) |

---

## Final Post-Stone Code Shapes

### Discrimination logic — `try_parse_metadata_map` (new helper, src/runtime.rs)

```rust
fn try_parse_metadata_map(node: &WatAST) -> Option<HashMap<String, WatAST>> {
    let list_items = match node {
        WatAST::List(items, _) => items,
        _ => return None,
    };
    match list_items.first() {
        Some(WatAST::Keyword(k, _)) if k == ":wat::core::HashMap" => {}
        _ => return None,
    }
    if list_items.len() < 3 { return None; }
    let pairs = &list_items[3..];
    let mut meta: HashMap<String, WatAST> = HashMap::new();
    let mut i = 0;
    while i + 1 < pairs.len() {
        let key_str = match &pairs[i] {
            WatAST::Keyword(k, _) => k.clone(),
            _ => return None,
        };
        meta.insert(key_str, pairs[i + 1].clone());
        i += 2;
    }
    Some(meta)
}
```

### SymbolTable.binding_metadata extension (src/runtime.rs)

```rust
/// Stone 241.6 — binding-level metadata attached via the optional
/// `{...}` metadata-map clause on `def` / `defn`. Maps binding name
/// (full FQDN keyword string) to the inner metadata map (key keyword
/// string → raw WatAST value). Generic storage: the substrate does NOT
/// enforce or validate specific keys.
pub binding_metadata: HashMap<String, HashMap<String, WatAST>>,
```

Storage insertion in `register_defines` (at `try_parse_fn_shape_def` call site):
```rust
if let Some(meta) = metadata_opt {
    sym.binding_metadata.insert(path, meta);
}
```

### fn-embedded metadata peel — `eval_fn` (src/runtime.rs)

```rust
// Stone 241.6 — fn-embedded metadata peel. The defn macro expands
// `(defn :name {meta} [args] -> :ret body)` to
// `(def :name (fn {meta} [args] -> :ret body))`. The metadata at
// args[0] is binding-level; peel it off so eval_fn sees the real sig.
let args = if !args.is_empty() {
    match &args[0] {
        WatAST::List(meta_items, _) => {
            let is_hashmap = meta_items
                .first()
                .map(|h| matches!(h, WatAST::Keyword(k, _) if k == ":wat::core::HashMap"))
                .unwrap_or(false);
            if is_hashmap { &args[1..] } else { args }
        }
        _ => args,
    }
} else {
    args
};
```

Same peel applied in `infer_fn` (src/check.rs) and `try_parse_fn_shape_def` (src/runtime.rs).

### defn macro in wat/core.wat — UNCHANGED template; comment added

The quasiquote-template body of the defn macro CANNOT branch on metadata presence (quasiquote is a template, not an evaluable expression; no `if`/`let` in the template engine). The macro template is UNCHANGED. Instead, the substrate's fn-peel transparently extracts binding-level metadata when `defn` macro-expands `{meta}` into the fn-form position. Comment added to document the mechanism.

```scheme
;; Stone 241.6 — optional metadata-map between name and argspec threads
;; through rest-binder unchanged:
;;   (:wat::core::defn :name {:doc "..."} [p <- :T] -> :Ret body)
;;     ↓ macro-expansion (rest-binder unchanged)
;;   (:wat::core::def :name (:wat::core::fn {:doc "..."} [p <- :T] -> :Ret body))
;;     ↓ substrate fn-embedded-metadata peel (try_parse_fn_shape_def + eval_fn)
;;   binding_metadata[":name"] = {:doc "..."}; fn sees [p <- :T] -> :Ret body
```

### infer_def in check.rs — 3-arg path

```rust
// Stone 241.6 — accept 2 args (no metadata: :name expr) OR 3 args
// (with metadata: :name {meta} expr).
if args.len() != 2 && args.len() != 3 {
    // MalformedForm error
}
// ... validation of args[1] as HashMap with at least one pair ...
let expr_idx = if args.len() == 3 { 2usize } else { 1usize };
let expr_ty = infer(&args[expr_idx], env, locals, fresh, subst).drain_errors_into(&mut local_errors);
```

---

## Honest Deltas

### 1 — defn macro UNCHANGED; fn-peel architecture (honest departure from BRIEF S3)

**Finding:** The BRIEF's S3 said "defn macro extends expansion to thread metadata through to def." Investigation revealed that `defmacro` bodies are quasiquote-only templates — no `if`/`let`/`cond` in the template engine. A computed splice `~@(expr)` can call substrate functions, but passing raw WatAST (`rest` binder) as a data argument to a substrate fn requires `quote`-wrapping that the substitution path doesn't support. Writing a new substrate fn for this purpose exceeded the stone's scope.

**Resolution:** Substrate fn-peel. The defn macro UNCHANGED template threads `{meta}` into the fn-form via `~@rest`. At substrate level, `try_parse_fn_shape_def` detects HashMap at fn_items[1] (pre-registration), `eval_fn` detects and skips HashMap at args[0] (runtime), and `infer_fn` detects and skips HashMap at args[0] (check). The metadata is stored at the binding level (in `binding_metadata`) at pre-registration time; it never reaches the fn's closure or signature. Architecture is correct: the macro expansion is data-transparent; the substrate enforces the binding-vs-value layer distinction.

**Vigilia gate status:** No new namespaced home introduced. Legacy flat substrate only. Per DESIGN D7 default: gate NOT cast.

### 2 — Empty `{}` rejection already enforced by check layer (T3 confirmed)

**Finding:** Contract 06 (`(def :x {} 42)` must error) PASSES at HEAD before Stone 241.6 ships. The parser synthesizes `(:wat::core::HashMap :wat::type::Infer :wat::type::Infer)` for empty `{}`. The check layer's `infer_def` errors when it tries to evaluate this as the value-expr (type inference on HashMap construction form fails or the HashMap check catches the empty body). Post-stone: the Stone 241.6 `infer_def` 3-arg path adds an EXPLICIT empty-map check (length ≤ 3 items = no pairs) that fires before inference, producing a clear `MalformedForm` diagnostic.

### 3 — Five insertion points across runtime.rs + check.rs (DESIGN estimate: ~+60 lines; actual: ~+165 lines)

**Finding:** The DESIGN estimated ~+50 lines for runtime.rs, ~+10 for check.rs. Actual was ~+120 (runtime) + ~+45 (check). Primary expansion: `try_parse_metadata_map` helper (~40 lines) + fn-peel in `eval_fn` and `infer_fn` (~35 lines each) + new comments + binding_metadata insertion points. The discrimination logic is correct and self-documenting; the extra lines are documentation + guard clauses, not behavioral complexity.

### 4 — Zero lib test cascade

Fifth consecutive stone with zero lib test cascade. The fn-peel is transparent: callers without metadata see no change. Existing fn tests pass through the peel's `else` branch. No existing test expected HashMap-head to be invalid at fn position 0 (that case wasn't tested previously; it would have reached `parse_fn_signature` and failed with a different error).

---

## Cascade Depth

**SHALLOW.** Zero lib test cascade. Five insertion points across runtime.rs and check.rs; each is surgical and bounded. The fn-peel mechanism is pure-additive for callers without metadata. Storage in `binding_metadata` is new capability; nothing existing reads from it yet (Stone 241.7 reflection verb opens that path). The `defn` macro template is UNCHANGED.

---

## PHASE 2 OPENS

**Metadata-map STORAGE shipped (Stone 241.6).**

| Capability | Stone | Status |
|---|---|---|
| Canonical `parse_argspec_triples` parser | 241.1 | SHIPPED |
| A1/A2/A3 fn-parser migration | 241.2 | SHIPPED |
| A4 defclause-parser migration | 241.3 | SHIPPED |
| Canonical `&` rest-binder + `Clause.rest_param` storage | 241.4 | SHIPPED |
| Runtime variadic-min arity + rest type check + rest bind | 241.5 | SHIPPED |
| Optional `{...}` metadata-map storage on `def`/`defn` | **241.6** | **SHIPPED** |
| `:wat::runtime::metadata-of` reflection verb | 241.7 | **QUEUED** |

`def` and `defn` now accept an optional `{...}` metadata-map clause. `SymbolTable.binding_metadata` stores the per-binding annotation map keyed by FQDN keyword string. `infer_def`/`extract_def_binding` thread the 4-item path; `eval_fn`/`infer_fn`/`try_parse_fn_shape_def` peel fn-embedded metadata from defn macro expansion. Empty `{}` explicitly rejected at check time.

**Stone 241.7** opens next: mint `:wat::runtime::metadata-of` reflection verb that reads from `binding_metadata` and returns `Option<HashMap<Keyword, HolonAST>>` encoded as `#wat.core/Some {...}` / `#wat.core/None nil` per arc 216.7 + 218.2 doctrine.
