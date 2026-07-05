# BRIEF — substrate stone: body-only `extend-type` inherits nil in a baked stdlib source

> **Executor: one sonnet SHADOWDANCER.** Orchestrator isolated the root; weighs the kill against its own re-run. Work
> ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first; `.claude/worktrees/` is illegal). `cargo nextest run` (NEVER
> `cargo test`), `cargo wat <file>` to dogfood. **Commit NOTHING.**

## The confirmed bug (isolated by the orchestrator — do not re-derive, verify)

A **body-only `extend-type`** (e.g. `(ensure-schema [self table indexes] body)` — bare params, no declared return)
inherits its arg + return types from the surface method. This inheritance **WORKS in user source** but produces **`nil`
for every arg and return when the `extend-type` is itself in a BAKED stdlib source, inheriting from a surface in another
baked stdlib source.** Result: `signature declares :wat::core::nil` (a `ReturnTypeMismatch` on every impl method), plus
`self`/args reading as `nil`.

It is a never-exercised baked-context path — no stdlib file `extend-type`'d a stdlib surface until `wat/query/mem.wat`.
It is NOT the generic mechanism (5 user-context probes prove `extend-type` + `Result<T,E>` + record/vector args + a
`Peer'` field all inherit fine in user source).

## The RED probe (create it, watch it fail, then your fix turns it green)

Create `wat/query/probe-extend.wat` and bake it in `src/stdlib.rs` **right after** the `wat/query.wat` entry:

```wat
(:wat::core::defstruct :wat::query::ProbeExtend [n <- :wat::core::i64])
(:wat::core::extend-type :wat::query::ProbeExtend :wat::query::Store
  (ensure-schema [self table indexes] (:wat::core::Ok nil))
  (put [self rows] (:wat::core::Ok nil))
  (scan [self q]
    (:wat::core::Ok (:wat::query::Page (:wat::core::Vector :wat::query::Row) :wat::core::None)))
  (scan-index [self q]
    (:wat::core::Ok (:wat::query::IndexPage (:wat::core::Vector :wat::query::IndexRow) :wat::core::None))))
```

`cargo nextest run --release -E 'test(query_contract)'` → **4 `ReturnTypeMismatch … signature declares nil`** errors at
`wat/query/probe-extend.wat`. That is the exact bug in minimal form. Your fix makes this go green (and `self`/args typed).
(This probe FILE + stdlib entry are TEMPORARY scaffolding — leave them un-committed; the orchestrator decides the
permanent gate. Do NOT delete `wat/query/mem.wat`.)

## Where to look (the rooms — orchestrator's grounded pointers)

1. **`src/check.rs:8912-8938`** (`collect_splice_defs_ctx`, the `:wat::core::extend-type` `is_top` arm) — registers each
   impl as a `TypeScheme` under `<type_name>/<method>` from **`clause.args.fixed_params` + `clause.return_type`**. For a
   body-only impl those are nil. NOTE it already holds the surface: `env.types().get(&ed.protocol_name)` →
   `TypeDef::Surface(s)` with `s.members` (`SurfaceMember::Method { name, ret, args, .. }`).
2. **`src/runtime.rs:6157` `parse_extend_type_form`** — a ONE-ARG PURE parser (no `env`/surface); it cannot inherit, so a
   body-only clause's `return_type`/param types default to nil/Infer here.
3. **The body-only inheritance that DOES work in user source** — find the pass/site that fills a body-only impl's missing
   types from the surface's `SurfaceMember::Method`. It is surface-aware; determine why it does not apply (or runs before
   the surface is available) during the BAKED stdlib freeze. Grep `SurfaceMember::Method`, `extend`, and the freeze
   pipeline (`src/freeze/env.rs`, the register/check pass order for stdlib).
4. **`src/check.rs:5910-5944`** is the CALL-SITE arm (surface-method dispatch) — it WORKS (the S0 test dispatches through
   it). It is NOT the bug; use it only as the reference for how to read `SurfaceMember::Method { ret, args }`.

## The fix (hypothesis to prove or replace)

Make a body-only baked `extend-type` inherit its impl signatures from the surface's `SurfaceMember::Method`, the same as
user source does. The likely fix is at `check.rs:8912` — when a clause's types are missing (body-only), inherit `ret` +
param types from `s.members` (already in scope) instead of using the nil clause values — OR a freeze-order correction so
the existing user-path inheritance runs for baked stdlib too. Prove which by reading the user-path inheritance first;
mirror it for baked. Keep it minimal; do not rewrite the extend-type machinery.

## STOP triggers (rejection criteria)

- **STOP-USER-REGRESSION:** user-source `extend-type` (incl. body-only, incl. the S0 test's `ReadStore` extend-type) must
  keep working identically. If your change alters user-context behavior, STOP.
- **STOP-CASCADE:** if the fix wants to change a function signature threaded through many call sites (like the last
  attempt at the expand_all gap), STOP and report — prefer a localized inherit-from-surface at the registration site.
- **STOP-CHECK-WEAKEN:** do not make the checker accept a genuinely wrong impl (e.g. by defaulting returns to Infer/Any).
  The impl must inherit the REAL surface return type and be checked against it.

## The gate (EXPECTATIONS)

| what | command | expected |
|---|---|---|
| the RED probe goes GREEN (baked extend-type inherits real sigs) | `cargo nextest run --release -E 'test(query_contract)'` | passed |
| baked `mem.wat` now type-checks | temporarily bake `wat/query/mem.wat` too → the same test | passed (then un-bake it; report) |
| user-context extend-type intact | `cargo wat` a body-only `extend-type` of a user surface returning `Result<…>` | works |
| whole floor | `cargo nextest run --release` | `0 failed` (modulo the known `no_inlined_wat_in_tests` reminder) |

## Blast radius

The register/check inheritance site (`src/check.rs` around `:8912`, or the freeze-order in `src/freeze/env.rs`), + the
temp probe (`wat/query/probe-extend.wat` + one `src/stdlib.rs` entry). No change to `parse_extend_type_form`'s arity, no
new params threaded through call sites.
