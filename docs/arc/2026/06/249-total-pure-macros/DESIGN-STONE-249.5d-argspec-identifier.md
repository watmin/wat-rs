# DESIGN — Stone 249.5d: ArgSpec carries the Identifier (the strip-and-re-walk root fix)

> Status: STRIKE-DRAWING.
> Predecessors (shipped): 249.5a (`e871d22a`, mint `src/scope/` + lift `identifier.rs`),
> 249.5b (`aaffdf64`, `env_key` — capture 200→105), 249.5c-fix1 (`ab14e400`, defclause
> cascade), 249.5c-fix2 (`9a49e7c7`, `src/scope/` encapsulation soot).
> This stone closes the JUNCTURE the `src/scope/` R2 re-cast surfaced and the user
> diagnosed: *"why isn't this handled in argspec?"*

## Why

The macro-hygiene fix (249.5b/c) made runtime resolution key on the full
`(name, scope-set)` identity via `scope::resolution::env_key`. But `ArgSpec`
(`src/argspec/parse.rs:14`) carries **bare `String`** parameter names —
`parse_triple` (`:158`) does `WatAST::Symbol(ident, _) => ident.as_str().to_owned()`,
**throwing away the `Identifier`'s scope tags at parse time.**

So every consumer that needs the SCOPED key re-derives it by **re-walking the raw
arg-vector** a second time:

- `crate::scope::scoped_arg_names` (`src/scope/resolution.rs:122`)
- `runtime::scoped_params_from_args_vec` (`src/runtime.rs:4173` → delegates)
- `function::eval::extract_scoped_params` (`src/function/eval.rs:27` → delegates)

This re-walk is a **band-aid**, and it carries a latent bug. `scoped_arg_names`
guards `if items.len() % 3 != 0 { return fallback }` — but a **rest-binder**
argspec (`name <- :T … & rest <- :T`) has `3n + 4` items (`3n` fixed + `&` + the
3-item rest triple), and `3n + 4 ≡ 1 (mod 3)`. The guard fires → the function
returns the **bare** fallback for the fixed params → its `&`-skip branch is
**dead**. A macro-generated `defclause`/`fn` **with a rest param** therefore binds
its fixed params BARE while the scope-tagged body looks them up SCOPED →
latent `UnboundSymbol`. (The current hygiene probe uses no rest param, so the bug
is unexercised — see the RED contract below.)

The bug in the re-walk is a **symptom**. The root is the strip: `ArgSpec` keeps a
poorer type than it was handed, forcing every scoped consumer to reconstruct what
the parser already saw.

## What it delivers (failure-engineering: eliminate the strip-and-re-walk CLASS)

`ArgSpec` **preserves the `Identifier`** (name + scopes). Each consumer derives
exactly the view it needs from the ONE source:
- **bare** via `ident.as_str()` (macro substitution keys, struct/enum field names),
- **scoped** via `env_key(&ident)` (runtime bind keys).

The re-walk vanishes — `scoped_arg_names`, `scoped_params_from_args_vec`,
`extract_scoped_params`, and the fn-path inline rest-compensation are **deleted**.
The `% 3` bug, the dead `&`-branch, the fn/defclause re-walk split, and the
double-walk all become un-expressible at once. **Behavior-preserving:** the scoped
strings produced are byte-identical (same `env_key` over the same identifiers);
the only delta is the rest-binder case, which goes from latent-`UnboundSymbol` to
correct (the RED contract proves it).

## The contract decision (pinned)

**`ArgSpec` carries `Identifier`; bare-vs-scoped is decided AT THE CONSUMER, never
in the parser.**

```rust
pub struct ArgSpec {
    pub fixed_params: Vec<(Identifier, TypeExpr)>,   // was (String, TypeExpr)
    pub rest_param: Option<(Identifier, TypeExpr)>,  // was (String, TypeExpr)
}
```
`parse_triple` returns `ident.clone()` instead of `ident.as_str().to_owned()`.

**Corollary (refined by the crawl):** `parse_fn_signature_prefix`
(`src/function/parse.rs:163`) feeds BOTH the eval tier (scoped) AND the check
tier (bare). It must stay **neutral** — return the `Identifier`s (not pre-keyed
strings) — so the eval callers can `env_key` and the check callers can `as_str`.
The prefix is the fork point; the fork is resolved one level out, at each caller.

## Consumer map (grounded — crawl + direct reads this session)

**SCOPED (derive `env_key(&ident)`):**
| Site | Today | After |
|---|---|---|
| `runtime.rs:4280` `try_parse_fn_shape_def` | `scoped_params_from_args_vec(.., params_bare)` | `parse_fn_signature` returns scoped (via identifiers); use directly |
| `runtime.rs:4380` `try_parse_variadic_def_fn_form` (fixed) | `scoped_params_from_args_vec(.., fixed_params_bare)` | `spec.fixed_params.iter().map(|(id,_)| env_key(id))` |
| `runtime.rs:4385-4396` same (rest, inline walk) | inline `args_vec…position('&')…env_key` | `spec.rest_param.map(|(id,_)| env_key(&id))` |
| `runtime.rs:6753` `parse_defclause_clause` | `scoped_arg_names(&items[0], &bare_names)` | `env_key` over `spec.fixed_params`/`rest_param` |
| `function/eval.rs:67` `eval_fn` | `extract_scoped_params(sig_args[0], params_bare)` | `parse_fn_signature` returns scoped; use directly |

**BARE (derive `ident.as_str()`):**
| Site | Use |
|---|---|
| `macros/parse.rs:172-173` `parse_defmacro_form` | `MacroDef.params`/`rest_param` (expansion-time substitution keys) |
| `types/defstruct.rs:377` `parse_struct_fields` | struct field names (type-registry metadata) |
| `types.rs:1772` `parse_defenum` | tagged-variant field names (type-registry metadata) |

**CHECK (stay BARE — neutral prefix → `as_str` at the caller):**
| Site | Use |
|---|---|
| `function/parse.rs:163` `parse_fn_signature_prefix` | return `Vec<Identifier>` (neutral) |
| `function/parse.rs:218` `parse_fn_signature_for_check` | names unused (arity + types only) |
| `function/infer.rs:42/115` `…_for_check_diag` → `infer_fn` | binds bare, looks up bare — keep bare-consistent |

**UNAFFECTED (read `Function.params`, not `ArgSpec`):** `closure_extract.rs:203`
(seeds free-symbol walk from `func.params` — already the runtime-level key string).

## Out of scope = rejected (affirmative cuts)

- **The check-pass BIND-SCOPED / LOOKUP-BARE mismatch.** The crawl surfaced (and I
  verified at `check.rs:3397`: `locals.get(ident.as_str())`, `None`→`fresh.fresh()`)
  that `check_function_body` (`check.rs:3149`) and `infer_defclause`
  (`check.rs:7763`) bind scoped keys (from `func.params` / `clause.args`) but look
  up body symbols BARE → a macro-generated body's params miss the bind and fall
  through to a fresh type var (imprecise, silently permissive; no hard error).
  **This is PRE-EXISTING — introduced by 249.5b's scoped strings flowing into the
  check locals, NOT by this stone — and this stone PRESERVES it exactly** (same
  scoped strings, derived from identifiers instead of a re-walk). It is a distinct
  failure class (check-pass resolution-key consistency) deserving its own probe (a
  macro-generated defclause body that type-checks IMPRECISELY today → precisely
  after). **Tracked as the NAMED follow-on Stone 249.5e** (DESIGN to draft at this
  stone's close); not folded in, because mixing it widens the blast radius into
  `check.rs`'s hot symbol-lookup path and it has an independent contract.
- **The broad flat-`src/*.rs` → homes migration** (runtime.rs, check.rs) — the
  pending arc-170/109-era work; untouched here.
- **Closure-extract round-trip of a scoped param** (`closure_extract.rs:2351` wraps
  `func.params[i]` in `Identifier::bare`) — latent only if closure extraction runs
  on a macro-generated function; no caller exercises it; re-open on a real need.

## Rooms (exact, for the brief)

- `src/argspec/parse.rs` — struct (`:14-20`), `parse_triple` (`:153-174` → `:158`),
  `parse_argspec_triples` (`:69-118`, build `ArgSpec` from identifiers).
- `src/function/parse.rs` — `parse_fn_signature_prefix` (`:115-166`, return
  `Vec<Identifier>`), `parse_fn_signature` (`:186`, eval — `env_key`),
  `parse_fn_signature_for_check` (`:218`, `as_str`/unused).
- `src/function/infer.rs` — `parse_fn_signature_for_check_diag` (`:42`),
  `infer_fn` bind (`:115`) — keep bare.
- `src/function/eval.rs` — DELETE `extract_scoped_params` (`:23-29`); caller (`:67`)
  uses `parse_fn_signature`'s now-scoped result.
- `src/runtime.rs` — DELETE `scoped_params_from_args_vec` (`:4173-4177`);
  `try_parse_fn_shape_def` (`:4276-4280`), `try_parse_variadic_def_fn_form`
  (`:4377-4408`, DELETE inline rest-compensation `:4385-4396`),
  `parse_defclause_clause` (`:6730-6766`).
- `src/scope/resolution.rs` — DELETE `scoped_arg_names` (`:92-152`) + its
  `pub use` (`src/scope/mod.rs:30`).
- `src/macros/parse.rs` (`:172-173`), `src/types/defstruct.rs` (`:377`),
  `src/types.rs` (`:1772`) — `.as_str()` over the identifiers.

## Probe (the disconfirming contract — RED at HEAD)

`tests/probe_argspec_rest_param_hygiene.rs` — a macro-generated `defclause` AND a
macro-generated `fn`, **each with a `& rest <- :T` rest param**, evaluated so the
body references both a fixed param and the rest param. RED at HEAD (the `% 3` guard
binds the fixed params bare → `UnboundSymbol`); GREEN after the root fix. Written +
verified RED before the brief, committed as design substrate the executor mirrors.

## Decomposition

One atomic strike (the representation change forces every consumer in the same
breath; splitting leaves the tree non-compiling between halves). Sonnet executes
against the committed probe; orchestrator scores against an independent re-run.
