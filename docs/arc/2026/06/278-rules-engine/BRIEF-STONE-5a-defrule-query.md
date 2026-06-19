# BRIEF — Stone 5a: `defrule` (rule macro) + `query` (read derived facts)

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A PURE
WAT stone (`wat/rete.wat` only — NO Rust). Build, run the named tests, report verbatim. Another agent weighs.

## The work
Add two surface forms to `wat/rete.wat`: the **`defrule`** macro (readable rule form → a zero-arg `defn`
returning a `Rule`) and the **`query`** fn (read derived facts of a type from a fired session). The reflection
that auto-gathers rules (`collect-rules`) is stone 5b — NOT here; 5a's probe collects the one rule manually.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-5a-defrule-query.md` — the worked `defrule` expansion, the
   macro parse approach, the `query` algorithm, the pinned contract, out-of-scope.
2. `wat/test.wat:303-310` — `deftest`: the canonical defmacro shape (`-> :wat::WatAST`, backtick quasiquote,
   `~` unquote, `~@` unquote-splice, expanding to a `defn` whose RETURN TYPE marks it). `defrule` mirrors this
   (zero-arg `defn` returning `:wat::rete::Rule`).
3. `wat/core.wat` — a representative `defmacro` using `map` / `ast->children` / `keyword/to-string` over its
   args (the macro-eval engine supports `map`, `~@`, nested quasiquote, ast helpers — arc 249). Find one to
   copy the idiom for the per-condition quote-and-assemble.
4. `wat/rete.wat` — the `Rule` record (`:48-55`: name String, lhs PV<WatAST>, rhs PV<WatAST>); how probes
   build a `Rule` with `(:wat::core::quote <cond>)` forms (`tests/probe_arc278_4a_production_fire.rs:35-38`);
   the `Session`/`production-memory` (`:124-131`) + the 4c flatten-production-memory + filter-by-type idiom.
5. `tests/probe_arc278_5a_defrule_query.rs` — the contract (already live, RED). Do not modify it.

## Part 1 — `query` (do this first; it's small + unblocks the probe's query tests)
`(:wat::core::defn :wat::rete::query [session <- :wat::rete::Session  ty <- :wat::core::keyword] -> :wat::core::PersistentVector ...)`:
- normalize `ty` to the `(:wat::core::type fact)` string: `(:wat::core::keyword/to-string ty)`, then strip a
  leading `:` if present (so `:weather::ColdAndWindy` → `"weather::ColdAndWindy"`; confirm `keyword/to-string`'s
  exact output and strip iff needed).
- flatten `production-memory` values into one `PV<:wat::Record>` (foldl over
  `(:wat::core::PersistentMap/values (:wat::rete::Session/production-memory session))`, inner foldl `conj` — the
  exact idiom in `probe_arc278_4c_retraction.rs`'s `derived_of`).
- `filter` by `(:wat::core::= (:wat::core::type f) <ty-string>)`; return the matching `PV`. Empty PV if none.

## Part 2 — `make-rule` (runtime fn) then `defrule` (the macro)
**This is the part the first attempt LOOPED on. Two fixes are baked in below — heed them.** The macro is kept
TRIVIAL: it quotes the WHOLE `:when` vector + `:then` forms; a plain runtime `make-rule` does the per-element
split. NO per-element quoting, NO nested quasiquote, NO `map`-over-conditions in the macro.

**`make-rule` (runtime fn — build it first):**
`(:wat::core::defn :wat::rete::make-rule [name <- :wat::core::String  when-ast <- :wat::WatAST  then-ast <- :wat::WatAST] -> :wat::rete::Rule …)`.
`when-ast`/`then-ast` are quoted VECTOR nodes. `(:wat::core::ast->children when-ast)` → the per-element
condition WatASTs; convert that to `PersistentVector<wat::WatAST>` (a small `children->pv` = foldl `conj`, since
`ast->children` may return a std `Vector`); same for `then-ast`. Return `(:wat::rete::Rule name lhs-pv rhs-pv)`.

**`defrule` macro — expansion (DESIGN §expansion):**
```
(:wat::core::defn <name> [] -> :wat::rete::Rule
  (:wat::rete::make-rule <name-string>
    (:wat::core::quote <when-vec>)           ;; the whole [conds] node, one quote
    (:wat::core::quote [<insert1> <insert2> …])))   ;; the inserts spliced into a vector literal
```
Steps inside the macro:
1. **`name-string` = `(:wat::core::ast-name name)` then strip a leading `:` iff present.** ⚠ NOT
   `keyword/to-string` — `name` is a WatAST NODE, and `keyword/to-string` is for a keyword VALUE; `ast-name`
   handles Symbol AND Keyword nodes (precedent `service.wat:356`, `deporder.wat:116`). This exact mistake is
   what made the first attempt loop hunting for a name primitive.
2. `when-vec = (:wat::core::get rest 1)` (the `[...]` conditions node); `then-forms = (:wat::core::drop rest 3)`
   (the insert forms). Assume canonical `:when` then `:then` order (STOP if a general parser is required).
3. Emit via quasiquote — quote `when-vec` whole; splice `then-forms` into a vector literal (`Record.wat:114`
   `[~@…]` idiom):
   `` `(:wat::core::defn ~name [] -> :wat::rete::Rule (:wat::rete::make-rule ~name-str (:wat::core::quote ~when-vec) (:wat::core::quote [~@then-forms]))) ``

⚠ **SCOPE GUARD (the other loop cause):** `defrule` needs ONLY the name string from its own `name` argument
node. It does NOT enumerate, list, or reflect over defined functions — that is `collect-rules` = **stone 5b**
(a Rust primitive, NOT this stone). If you find yourself hunting a primitive to extract/list *fn* names for
collection, you have drifted into 5b — STOP. 5a's probe collects the one rule by hand
(`(:wat::core::PersistentVector (:weather::cold-and-windy))`).

## Builder directive: build missing deps, never hack around
Deps SHOULD all exist (`defmacro`, quasiquote/`~`/`~@`, `map`, `ast->children`, `keyword/to-string`,
`PersistentVector`, `PersistentMap/values`, `foldl`, `filter`, `=`, `type`, the `Rule`/`Session` accessors).
**If a macro-eval-engine op you need is genuinely missing / not on the pure-total allow-list → STOP + name it.**

## Engine-source bar (DOGFOOD)
LINT-CLEAN — `cond`/`contains?` over nested `if`; `format`/`interpolate` over nested `concat`. The ONLY
below-bar spot is the EXISTING `render-dag` compound-concat FIXTURE — do NOT touch it.

## STOP triggers
1. A needed macro-eval op is missing / not pure-total-allow-listed → STOP, name it (do NOT hack a workaround
   or move the macro logic to a runtime fn).
2. The keyword-section parse needs more than the canonical `:when`/`:then` order to satisfy the probe → STOP,
   describe (do NOT silently restrict the surface).
3. You reach for `collect-rules` / reflection / a Rust change / `defquery` / `QueryNode` / `Snapshot` → that's
   5b / later; STOP.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_5a_defrule_query -- --include-ignored        # 4/4 GREEN
cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored            # 4/4
cargo test --release -p wat --test probe_arc278_4b_cascade -- --include-ignored               # 4/4
cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored         # 4/4
cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored               # 4/4
cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored              # 3/3
cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored                 # 2/2
cargo test --release --test test_stdlib_load_order | grep result                               # 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                                    # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                     # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                            # Finished; 25 warnings (NO new)
```
Report: the `defrule` macro + `query` fn source verbatim; all outputs verbatim; any STOP hit. No git.
NOTE: `defrule` is a stdlib macro in `wat/rete.wat` — confirm `test_stdlib_load_order` stays 1/0 (rete.wat
loads after its deps; if defrule's expansion references something load-ordered later, that test catches it).

## Blast radius
`wat/rete.wat` ONLY (`defrule` macro + `make-rule` runtime fn + `query` fn + maybe small `children->pv` /
colon-strip helpers) + `tests/probe_arc278_5a_defrule_query.rs` (already live). NO Rust. NO record/signature
change. No git.
