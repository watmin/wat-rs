# BRIEF — the type-param splitter must track bracket depth (unblocks `lru-svc<K,V>`)

> Follow-on to `BRIEF-parametric-defservice.md` (landed `7336464e` — single-param services work and RUN).
> **This is the last thing between us and cache Stone 2.** Builder-ruled 2026-07-25.

## The defect — ONE function, grounded

`split_name_and_type_params` (`src/runtime.rs:3207`) splits a parametric name's body on **every** comma,
with no bracket-depth tracking:

```rust
let head   = kw[..lt_index].to_string();
let inside = &kw[lt_index + 1..kw.len() - 1];
let params: Vec<String> = inside
    .split(',')                     // ← flat. no depth.
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();
```

That was always sufficient, because its historical input was a *declaration* name like `defn :foo<K,V>` —
a body with no nesting. The `defservice` macro is the **first caller to feed it a name whose type-argument
list itself contains parametric types**:

```
Locus/launch<Op, Reply, State<K,V>, Admin<K,V>, Status<K,V>>
```

Flat-split yields **8** fragments instead of 5 — `Op`, `Reply`, `State<K`, `V>`, `Admin<K`, `V>`,
`Status<K`, `V>>` — so every parameter shifts. The observed error, verbatim:

```
:wat::spawn::Locus/launch<…>: parameter #2 expects :V>; got :probe::pair-svc::Admin<K,V>
```

Note `expects :V>` — a torn fragment. Single-param `<T>` works precisely because `State<T>` has no inner
comma to mis-split.

## This is NOT a syntax, lexer, or EDN problem — orchestrator-verified

- `:wat::core::HashMap<wat::core::String,wat::core::i64>` is a complete, legal single symbol; the comma is
  the required separator and whitespace inside `<>` is illegal. That is the current syntax and it is correct.
- **A nested comma in TYPE POSITION already type-checks clean** — proven this session, exit 0:
  ```clojure
  (:wat::core::defn :probe::f
    [m <- :wat::core::Vector<wat::core::HashMap<wat::core::String,wat::core::i64>>]
    -> :wat::core::i64  0)
  ```
  So the substrate's real type parser **is** depth-aware. This is one straggler that predates nested inputs.
- Do NOT "fix" this by mangling commas to underscores, changing the syntax, or touching the lexer.

## The fix

Make the split depth-aware: scan `inside`, track `<`/`>` nesting, split **only on commas at depth 0**.
Everything else about the function stays (the `ends_with('>')` guard, the trim, the empty filter).

A non-nested body has no depth to track, so **every existing caller is bit-for-bit unaffected** — that is
the safety property, and it is what makes this small.

## ⚠ GROUND THIS BEFORE YOU FIX — is it the ONLY flat splitter?

`split_name_and_type_params` is the one I located from the failing path, but I did **not** prove it is the
only one. **Grep the tree for sibling splitters on parametric names** and check each for the same shape —
start with `parse_declared_name` (`src/types.rs:3366`), and search for `.split('<'`, `.split(',')`,
`find('<')`, `ends_with('>')` near type-name handling in `src/`. Fix every one that is depth-blind; report
what you found. If a sibling turns out to be *correctly* flat (its input genuinely cannot nest), say so
rather than changing it.

## The gate

**1. The RED case must go green — and RUN, not merely `--check`.** This exact probe currently produces 8
type-check errors:

```clojure
(:wat::core::defsurface :probe::Pair<K,V> :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Pair::PutRequest [item <- :wat::core::i64])
   (:wat::core::defenum :probe::Pair::PutResponse :wat::enum::Pure
     :Ok              [echo <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(put [self <- :probe::Pair<K,V>  req <- :probe::Pair::PutRequest]
     -> :probe::Pair::PutResponse :max-request-bytes 1024)])

(:wat::service::defservice :probe::pair-svc<K,V>
  :satisfies :probe::Pair<K,V>
  :durable   [k <- :wat::core::Option<K>  v <- :wat::core::Option<V>]
  :ephemeral []
  :impls
  [(put [s req] (:wat::service::Outcome::Reply s (:probe::Pair::PutResponse::Ok 1)))])
```

Land it as a `deftest` sibling of `wat-tests/service-parametric.wat`, following that file's shape: stand the
service up, `connect'` to `Handle/addr`, round-trip one call, and make **both** `K` and `V` load-bearing —
pin them to *different* concrete types at the `/start` call site (e.g. `K=String`, `V=i64`) and have the
handler READ both durable fields. A gate where K and V are the same type would not prove the split.

**2. The existing single-param gate stays green** (`wat-tests/service-parametric.wat`).

**3. All nine concrete defservices unchanged.**

## Blast radius

`src/runtime.rs` (the splitter) + any sibling splitter you ground as having the same defect + the new gate.
Nothing else. **STOP + report if it exceeds this** — in particular, if fixing the split surfaces a *further*
multi-param failure downstream (the wire, `Locus/launch`'s own arity, the child-lineage forms), STOP and
report rather than chasing it; that is a separate ruling.

## Known-adjacent, explicitly OUT of scope

Process tier for parametric services is unverified — `child-main-form` emits
`(:wat::program::self-peer :<svc>::Status<T> :<svc>::Admin<T>)` inside `:user::main []` where `T` is free,
shipped as `forms` data and only re-checked at a forked child's startup. Do not chase it here.

## Gate

- The `<K,V>` probe green and round-tripping as a `deftest`.
- `cargo build --release` clean.
- `cargo nextest run --release` — report the **Summary line VERBATIM**. Current floor: **4170 passed,
  314 skipped**.
- Everything FOREGROUND; never background a command and return. **Do NOT commit** — the orchestrator weighs
  by their own re-run and commits.

## Your report

The diff shape, what your splitter grep found (every candidate, and your verdict on each), the probe's
before/after, the verbatim Summary line, any STOP, and anything you could not verify.
