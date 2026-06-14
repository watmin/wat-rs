# BRIEF — Stone C.1: `defservice` skeleton + the op enum (pure-wat defmacro)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`
as your FIRST action; operate ONLY here; use `git -C /home/watmin/work/holon/wat-rs` for git; any
path containing `.claude/worktrees/` is harness state — ignore it). Design (read fully):
`docs/arc/2026/05/209-defservice/DESIGN-STONE-C.1-defservice-skeleton-op-enum.md`. The RED probe
is on disk + verified RED at HEAD (`tests/probe_arc209_c1_defservice_op_enum.rs` —
`UnresolvedReference :my::counter::Op::Increment`). Do NOT commit — the Inquisitor weighs.

## The work in one paragraph

Mint `:wat::service::defservice` as a PURE-WAT defmacro in a NEW file `wat/service.wat`, register
it in `src/stdlib.rs`, and make the RED probe GREEN. C.1 emits ONLY the op enum: `(defservice
<fqdn> :state <T> :ops [(:Op [s <- :State …client-args] -> ret body) …])` expands to `` `(:wat::core::do
(:wat::core::defenum <fqdn>::Op …variants))`` where each variant is the op-head keyword plus, if
any client args remain after dropping the leading `s <- :State` triple, a `[field…]` vector. The
macro walks `:ops` WatAST-native (`ast->children`/`first`/`drop`/`empty?`/`with-children`/`conj`/
`foldl`), builds the enum name via the `keyword/of` pattern, and emits via NESTED quasiquote. The
full algorithm — copy it — is in the DESIGN's "The algorithm" section.

## Read in order (the rooms)

1. **`tests/probe_arc209_c1_defmacro_ast_walk.rs`** (GREEN at HEAD) — THE worked reference. It
   proves a defmacro can `ast->children` + `drop` + `with-children` its arg, and demonstrates the
   ONE contract that matters: the macro body's top-level is a REGULAR form (program-body path),
   params are node-values, NOT a top-level `` `~(…)`` quasiquote. Mirror this exactly.
2. **`wat/core.wat:254-292`** (`cond` defmacro) — the canonical node-walking defmacro shape:
   top-level `(if …)`, walks its arg with `first`/`rest`/`List?`/`Option/expect`, emits via NESTED
   `` `(…)`` with `~`/`~@`, aborts with `macro-error`. Your defservice has the same shape.
3. **`wat/core.wat:300-312`** (`keyword/of` defmacro) — how to BUILD a keyword from string parts:
   `keyword/to-string` (drops the colon) → `string::concat` → `keyword/from-string` (re-adds it).
   The enum-name build is identical: `(keyword/from-string (string::concat (keyword/to-string
   fqdn) "::Op"))`.
4. **`wat/fix.wat:47-52`** (`strip-if`) — the drop-children-and-rebuild move at runtime:
   `(with-children node (drop (ast->children node) N))`. C.1 does the same to the arg-vec (drop 3
   = the `s <- :State` triple). (fix.wat is a `defn`, not a defmacro — but the with-children call
   shape is identical.)
5. **`wat/spawn.wat:101-106`** (`defenum :wat::kernel::ServiceEvent`) — the exact defenum syntax
   you generate: a fieldless variant is a bare keyword (`:Shutdown`); a payload variant is keyword
   + field-vector (`:Message [idx <- :wat::core::i64 msg <- :O]`).
6. **`src/stdlib.rs:30-263`** — the `STDLIB_FILES` array. Add ONE `WatSource { path:
   "wat/service.wat", source: include_str!("../wat/service.wat") }` entry (place after the
   `wat/fix.wat` entry, ~line 256). Order is not load-bearing (comment at 231-236).

## Implementation sketch (fill from the DESIGN's algorithm verbatim)

`wat/service.wat` contains exactly one defmacro. Its shape (program-body path):

```clojure
(:wat::core::defmacro :wat::service::defservice
  [fqdn <- :wat::holon::HolonAST  _state-kw <- :wat::holon::HolonAST  state-ty <- :wat::holon::HolonAST
   _ops-kw <- :wat::holon::HolonAST  ops <- :wat::holon::HolonAST]
  -> :wat::holon::HolonAST
  (:wat::core::let
    [enum-name (:wat::core::keyword/from-string
                 (:wat::core::string::concat (:wat::core::keyword/to-string fqdn) "::Op"))
     clauses   (:wat::core::ast->children ops)
     variants  (:wat::core::foldl <op->tokens-fn> (:wat::core::Vector :wat::WatAST) clauses)]
    `(:wat::core::do
       (:wat::core::defenum ~enum-name ~@variants))))
```

`<op->tokens-fn>` is the `fn` in the DESIGN: per clause, `ast->children` → `opkw` = `(first ch)`,
`argvec` = `(first (drop ch 1))`, `fieldch` = `(drop (ast->children argvec) 3)`; then
`(if (empty? fieldch) (conj acc opkw) (conj (conj acc opkw) (with-children argvec fieldch)))`.
Copy it from the DESIGN. Use `macro-error` for a malformed clause (no head / no arg-vec) instead
of letting `Option/expect` panic if you prefer a cleaner diagnostic — both are acceptable.

## Blast radius

`wat/service.wat` (NEW) + one line in `src/stdlib.rs`. NO other Rust. NO change to the fence
(already shipped, `4718c897`). NO change to the probe.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** a head you need still reports `RefusedInMacro` — report which head; it's a fence
   gap (do NOT route through `from-wat`/holon reflection).
2. **STOP-2:** `ast->children` receives a *value* not a *node* (error "expected :wat::WatAST, got
   …") — you used a top-level quasiquote; restructure to the program-body path (top-level `let`,
   nested quasiquote), per probe_arc209_c1_defmacro_ast_walk. Report if unexpressible.
3. **STOP-3:** the generated `defenum` won't construct/match (the probe's `:my::counter::Op::Increment`
   doesn't resolve, or `n` doesn't extract) — report the exact emitted form (add a temporary
   `write-forms` debug if useful) so the variant shape can be compared to spawn.wat:101-106.

## The gate (report each exact `test result:` line; do NOT commit)

```
cargo test --release -p wat --test probe_arc209_c1_defservice_op_enum                # 1 passed
cargo test --release -p wat --test probe_arc209_c1_defmacro_ast_walk                 # 2 passed (fence prereq holds)
cargo test --release -p wat --lib -- --test-threads=1                                # 915 passed / 36 failed (PRE-EXISTING; ZERO new)
cargo test --release -p wat --test nursery -- --test-threads=1                       # 895 passed / 4 failed (zero new)
cargo test --release --workspace --no-run                                            # full surface compiles
```

NOTE: the lib unit suite has 36 PRE-EXISTING failures (`check::tests`/`runtime::tests`) — NOT
yours; confirm the count stays 36. Run `cargo test` PLAINLY (no setsid/timeout). The harness may
show stale rust-analyzer diagnostics that contradict a clean `cargo build` — trust your own build.

## Prior comparable (copy the shape)

`wat/core.wat` `cond` (the node-walking defmacro template) + `tests/probe_arc209_c1_defmacro_ast_walk.rs`
(the proven program-body composition). For the strike cycle shape, `BRIEF-STONE-C0b.3b-e.md`.
