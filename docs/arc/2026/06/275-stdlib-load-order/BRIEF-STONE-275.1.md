# BRIEF — Stone 275.1: `deporder`, the stdlib load-order analyzer (one intrinsic + pure wat)

## The work (one paragraph)

Build a self-hosted load-order verifier. (1) Add **one thin Rust intrinsic** `:wat::stdlib::sources`
that hands wat the runtime's own baked load order — the `(path, source)` pairs of `STDLIB_FILES`, in
order. (2) Build a pure-wat tool `wat/deporder.wat` (namespace `:wat::deporder::`) that, given an ordered
list of sources, parses every top-level form, builds a map of every defined symbol → (file, def-kind),
classifies every cross-file reference (definer is a `defmacro` → **order-free**; definer is a
`defn`/`defenum`/`defalias`/`def`/`defprotocol`/`defclause` → **eval-dependency**, the referencer must
load after the definer; symbol defined nowhere → **intrinsic**, order-free), and returns the list of
**violations** — a file that eval-depends on a file loaded after it (empty = the order is valid). The
top surface is two lines: `(verify-stdlib) = (verify (stdlib-sources))`.

`wat/fix.wat` is your **worked reference** for AST-walking; `src/io.rs::eval_io_list_dir` is your
worked reference for the intrinsic.

## Read in order (the rooms)

1. **`src/io.rs:1425-1452`** (`eval_io_list_dir`) — the intrinsic-return pattern: build
   `Value::Vec(Arc::new(entries))` of `Value::String(Arc::new(s))`. Your `:wat::stdlib::sources` does
   this one level deeper: a `Value::Vec` of `[path, source]` pairs (each pair itself a
   `Value::Vec` of two `Value::String`s).
2. **`src/runtime.rs:4345-4369`** — the `:wat::io::*` eval dispatch arms; add a `:wat::stdlib::sources`
   arm here (or wherever the keyword-head match lives), calling your new eval fn.
3. **`src/check.rs:14338-14346`** (the `read-file` scheme registration) — copy this shape to register
   `:wat::stdlib::sources`'s type scheme: zero params, returns
   `Vector<Vector<String>>` (`TypeExpr::Parametric { head: "wat::core::Vector", args: [Vector<String>] }`).
4. **`src/stdlib.rs:26` (`stdlib_files()`)** — the source of truth your intrinsic maps: each
   `WatSource { path, source }` → the `[path, source]` pair, in array order.
5. **`wat/fix.wat:1-110`** — the AST walk you mirror: `:wat::core::ast-kind` (→
   `"list"`/`"keyword"`/`"symbol"`/…), `:wat::core::ast-name` (→ the name string), `ast->children`,
   `first`/`rest`/`drop`/`take`/`concat`/`empty?`, `Option/expect -> :wat::WatAST … "msg"`, the
   `structural?` predicate + recursive walk.
6. **`wat/fix.wat:310-318`** (`fix-text`) — **how to get a file's top-level forms**:
   `tree (:wat::core::read-string src)` then `(:wat::core::ast->children tree)`. `read-string` returns
   ONE container node; its children are the top-level forms. Use this exact pattern.
7. **`wat/service.wat:67-110`** — the HashMap idiom: `(:wat::core::HashMap :K :V)`, `HashMap/assoc`,
   `HashMap/get` (→ Option), `HashMap/contains-key?`, and the `foldl`+`range`+`get` walk.
8. **`wat/Record.wat:27-95`** — `(:wat::Record::def :ns::Name [field <- :Type …])` for the typed
   records (use typed records, not loose maps — a wrong shape must be uncompilable).
9. **`wat/test.wat:73-100, 265-330`** — `assert-eq<T>`/`assert-true` + the `deftest` shell for your
   proof.
10. **`src/stdlib.rs:30`** — register `wat/deporder.wat` as a `WatSource` (near `fix.wat`). **No
    reorder of the array in this strike** (that is 275.2).

## Implementation sketch (you fill the bodies)

### The intrinsic (Rust)

```rust
// src/<io.rs or a small new fn>; dispatched from runtime.rs, scheme in check.rs.
// :wat::stdlib::sources  () -> Vector<Vector<String>>   each = [path, source]
pub fn eval_stdlib_sources(args, list_span, ...) -> Result<Value, RuntimeError> {
    arity(":wat::stdlib::sources", args, 0, list_span)?;
    let pairs: Vec<Value> = crate::stdlib::stdlib_files().iter().map(|ws| {
        Value::Vec(Arc::new(vec![
            Value::String(Arc::new(ws.path.to_string())),
            Value::String(Arc::new(ws.source.to_string())),
        ]))
    }).collect();
    Ok(Value::Vec(Arc::new(pairs)))
}
```

### The tool (pure wat)

```clojure
(:wat::Record::def :wat::deporder::SourceFile [path <- :wat::core::String  source <- :wat::core::String])
(:wat::Record::def :wat::deporder::SymDef     [file <- :wat::core::String  kind <- :wat::core::String])
(:wat::Record::def :wat::deporder::Violation
  [referencer <- :wat::core::String  referencer-pos <- :wat::core::i64
   definer    <- :wat::core::String  definer-pos    <- :wat::core::i64
   symbol     <- :wat::core::String])

;; PURE CORE — operates on the provided ordered sources (no I/O, trivially testable):
(:wat::core::defn :wat::deporder::verify
  [files <- :wat::core::Vector<wat::deporder::SourceFile>]
  -> :wat::core::Vector<wat::deporder::Violation>  …)

(:wat::core::defn :wat::deporder::build-symbol-map
  [files <- :wat::core::Vector<wat::deporder::SourceFile>]
  -> :wat::core::HashMap<wat::core::String,wat::deporder::SymDef>  …)

;; THE SURFACE — wrap the intrinsic's [path source] pairs into SourceFiles, then verify:
(:wat::core::defn :wat::deporder::stdlib-sources [] -> :wat::core::Vector<wat::deporder::SourceFile>
  ;; map each [path source] pair from (:wat::stdlib::sources) → (SourceFile path source)
  …)
(:wat::core::defn :wat::deporder::verify-stdlib [] -> :wat::core::Vector<wat::deporder::Violation>
  (:wat::deporder::verify (:wat::deporder::stdlib-sources)))
```

### Algorithm

- **A definition form** = a top-level list whose head ast-name ∈
  `{:wat::core::defn, defmacro, defenum, defalias, def, defprotocol, defclause}`. The **defined symbol**
  = child[1]'s ast-name; the **kind** = the head's short tail (only "is it `defmacro`?" matters).
- **Pass 1 (`build-symbol-map`):** `foldl` over files; for each, `read-string`→`ast->children`→
  top-level forms; for each definition form, `assoc` defined-symbol → `SymDef{file=path, kind}`.
- **Pass 2 (references):** for each file, recursively walk every form (mirror `fix.wat` `structural?`+
  recurse), collecting nodes where `ast-kind == "keyword"` AND ast-name starts with `":wat::"` AND
  contains `"::"` past the prefix (mirror `fix.wat:56-59`). **Exclude child[1] of a definition form**
  (the symbol being defined is not a reference).
- **Eval-deps:** for each referenced symbol resolving (via the map) to a SymDef in a **different** file:
  `defmacro` → ignore; else → eval-dep edge referencer-file→definer-file. Unresolved → intrinsic →
  ignore.
- **`verify`:** position = index in the input vector. For each eval-dep edge, if
  `definer-pos > referencer-pos` → emit a `Violation`. Return all.

## Your proof (complectens — the tool carries its own proof)

Match how `fix.wat` is tested (find it: `grep -rln "fix-source\|fix-text" wat-tests/ tests/`). At
minimum, `deftest`s on **literal SourceFile fixtures** (no I/O — that's why the core is pure):

1. **defmacro ref is order-free.** `[{path "a" source "(:t::caller (:t::m))"} {path "b" source
   "(:wat::core::defmacro :t::m [] 1)"}]` → `verify` returns **empty** (a-before-b is fine; `m` is a
   defmacro). The load-bearing case.
2. **eval-dep wrong order is a violation.** `[{path "a" source "(:t::caller (:t::f))"} {path "b"
   source "(:wat::core::defn :t::f [] 1)"}]` → **one Violation**. Reverse `[b, a]` → **empty**.
3. **intrinsic ref ignored.** A file referencing `:wat::io::read-file` (defined in no fixture) → no
   violation.
4. **the surface runs.** `(:wat::deporder::verify-stdlib)` evaluates without error (it will be wired as
   the enforcement test in 275.2; here just prove it runs and returns a Vector).

## Blast radius

- NEW: `wat/deporder.wat`; the deftest file (match convention).
- EDIT Rust: the `:wat::stdlib::sources` intrinsic — one eval fn + one dispatch arm + one scheme
  registration. EDIT `src/stdlib.rs` — one `WatSource` entry for `wat/deporder.wat`. **No array reorder.**
- Nothing else.

## STOP triggers (halt + report; do NOT improvise)

1. **STOP-1** — if `read-string`+`ast->children` does not yield top-level forms as expected, STOP and
   report what it actually returns.
2. **STOP-2** — if a top-level form's head is none of the recognized def-heads and is not a plain
   skippable expression (e.g. `extend-type`, `use!`, bare `do`), STOP and report the head ast-name.
   `extend-type` defines methods on a type — surface it, do not silently drop it (a dropped definer
   hides a real eval-dep). Report the full list of unhandled head ast-names seen across the real stdlib.
3. **STOP-3** — if a typed-record field or HashMap type-arg, or the intrinsic's `Vector<Vector<String>>`
   scheme, won't compile, STOP and report the exact checker/compiler error; do not fall back to a loose
   map.

## After it's green

Run `(:wat::deporder::verify-stdlib)` on the **real** baked stdlib and **report the result** — empty
(order already valid) or the violations. Also report the dependency edges found. That output drives
stone 275.2 (the enforcement test + the meaningful reorder). **Do NOT reorder `STDLIB_FILES`.**

## Discipline

- **Do NOT spawn sub-agents.** Single executor.
- Ground every claim against the disk; a broken `deporder.wat` fails stdlib freeze → every test fails
  (instant feedback).
- Return: what you built (Rust intrinsic + wat tool), your own test results, the
  `(:wat::deporder::verify-stdlib)` output on the real stdlib, the line counts, any STOP hit.
