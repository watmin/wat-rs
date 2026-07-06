# BRIEF — `#[wat_dispatch]`: support `-> Result<Self, E>` (fallible opaque constructors)

> **Executor: one sonnet SHADOWDANCER** (running in PARALLEL with the mod/rem/quot shadowdancer — you touch
> DISJOINT files: `crates/wat-macros/` + `src/rust_deps/sqlite.rs`; it touches `src/runtime.rs`/`check.rs`/
> `wat/core.wat`. Do NOT touch its files). Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first; anchor git;
> `.claude/worktrees/` illegal). `cargo build` to check; `cargo nextest run --release` (NEVER `cargo test`).
> **Commit NOTHING.**

## The work

Teach `#[wat_dispatch]` to marshal a `-> Result<Self, E>` return automatically (a fallible constructor of an opaque
handle — `open`/`connect`/any resource acquisition), then **retire the `ctor_result` workaround** in
`src/rust_deps/sqlite.rs` (S1's `open`/`open_readonly` become plain `-> Result<Self, RawFault>`). This is the class
S1 surfaced (`ALIVS ARGVIT` — the first shim with a fallible opaque constructor); fix it once, every future resource
shim gets it for free.

## The grounded design (do NOT re-derive — verify + implement)

- The **scheme codegen already works** for `Result<Self,E>`: `rust_type_to_type_expr_tokens` (`codegen.rs:519`)
  handles a nested `Self` at `:520` (→ `emit_self_type_expr`, the opaque path) and `Result` at `:582` (recurses).
  **Do NOT change the scheme codegen.** (The S1 shadowdancer's scheme-break came from substituting the concrete
  struct name — don't do that; keep `Self`.)
- The **ONLY broken path is the runtime dispatch**: `emit_return_marshal` (`codegen.rs:357–395`). Today its
  `ReturnType::Type` arm (`:387`) handles bare `Self` (→ `wrap_self_return`) else the generic
  `<#ty as ToWat>::to_wat(result)` (`:392`). For `Result<Self,E>` the generic path re-quotes `Self` verbatim into
  the generated free fn (`mod __wat_dispatch_*`, not an impl block) where `Self` has no meaning
  (`cannot find type Self in this scope`). **Fix: add a `Result<Self,E>` arm that operates on the result VALUE, never
  naming `Self` as a TYPE** — the same shape `ctor_result` builds by hand (`src/rust_deps/sqlite.rs:224`).

## Implementation sketch (fill it; do not invent the shape)

```rust
// codegen.rs — refactor wrap_self_return so the INNER opaque-value expression is reusable
//   (today it wraps in `Ok(make_rust_opaque(TYPE_PATH, <cell>::new(#inner)))`; split out the
//    `make_rust_opaque(...)` part as `opaque_self_value(#inner) -> TokenStream`, scope-aware).
// then, in emit_return_marshal's `ReturnType::Type(_, ty)` arm, BEFORE the generic fall-through:
//
//   if let Some(err_ty) = result_ok_is_self(ty, self_type) {   // ty == Result<Self, ErrTy>
//       return Ok(quote! {
//           match result {
//               Ok(inner) => Ok(::wat::runtime::Value::Result(::std::sync::Arc::new(
//                   Ok(#opaque_self_value_of_inner)))),        // opaque_self_value(quote!{ inner })
//               Err(e)    => Ok(::wat::runtime::Value::Result(::std::sync::Arc::new(
//                   Err(<#err_ty as ::wat::rust_deps::ToWat>::to_wat(e))))),
//           }
//       });
//   }
//
// result_ok_is_self(ty, self_type): if ty is a Path `Result` with 2 generic args and
//   arg0 is `type_is_self` (or types_equal self_type) -> Some(arg1 Type), else None.
//   (Value::Result is `Arc<Result<Value, Value>>` — confirm from ctor_result / src/value.)
```

Then **retire the workaround** in `src/rust_deps/sqlite.rs`: delete `ctor_result`; change `open`/`open_readonly` to
`-> Result<WatSqliteConnection, RawFault>` / `-> Result<WatSqliteReadConnection, RawFault>` (bare `Self` won't work
because they're in an `impl` where the method returns the OTHER type for `open_readonly` — use the concrete struct
type, which `types_equal(ty, self_type)` catches for the RW one; for the RO constructor, see the note below); update
the module doc (the "one exception" paragraph) to reflect that the macro now handles it.

**Note (open_readonly):** it lives in `impl WatSqliteReadConnection` (or wherever) and returns *that* type — so its
return is `Self` for its own impl OR the concrete `WatSqliteReadConnection`. Make `result_ok_is_self` accept BOTH
`Self` and `types_equal(arg0, self_type)` (the same dual check the bare-Self arm at `:388` already uses), so a
constructor returning `Result<ConcreteSelfType, E>` in its own impl block works too.

## STOP triggers (halt + report)
1. **STOP-SCHEME:** do NOT modify `rust_type_to_type_expr_tokens` — it already handles nested `Self`. The fix is the
   runtime dispatch only.
2. **STOP-SELF-TYPE:** the new arm must NOT name `Self` as a type in the generated free fn (that's the whole bug);
   operate on the `result`/`inner`/`e` VALUES.
3. **STOP-REGRESS:** the bare-`Self` return (`:388`, the lru shim + S1's `&self` verbs) must keep working
   identically. `Result<T, E>` where T is NOT Self must keep hitting the generic `ToWat` path (unchanged).
4. **STOP-SCOPE:** handle `Result<Self, E>` only. Do NOT generalize to `Option<Self>`/`Vec<Self>`/nested — those have
   no consumer; name them out-of-scope if you think of them.

## The gate (EXPECTATIONS)
| what | command | expected |
|---|---|---|
| wat-macros + core compile | `cargo build --release` | clean |
| S1 sqlite gate still green (open/open_readonly now via the macro) | `cargo nextest run --release -E 'test(sqlite_interop)'` | passed |
| any wat-macros unit tests | `cargo nextest run --release -p wat-macros` (if present) | passed |
| whole floor | `cargo nextest run --release` | Summary line VERBATIM; 0 failed modulo the known `no_inlined_wat` reminder |

## Final report: the codegen diff (the new arm + `result_ok_is_self`) · the sqlite.rs simplification (ctor_result deleted, open/open_readonly signatures) · verbatim `sqlite_interop` + whole-floor Summary · STOP triggers hit or "none".
