# DESIGN — Stone 249.5e: the check pass keys locals by env_key (one keying policy)

> Status: STRIKE-DRAWING.
> The NAMED follow-on from Stone 249.5d (`ce6e331a` SCORE § Out of scope): the
> pre-existing check-pass BIND-SCOPED/LOOKUP-BARE mismatch, surfaced by the
> 249.5d consumer crawl and verified at `check.rs:3397`.

## Why

The runtime `Environment` keys parameters by `scope::resolution::env_key` (bare
ident → name; scoped macro-template ident → `name\u{1}<scopes>`) — that is what
made macro hygiene real at runtime (249.5b). The **type checker's** local
environment (`locals: &HashMap<String, TypeExpr>`, threaded through `fn infer`)
was never taught the same policy. It keys inconsistently:

- **Some binds are already env_key'd strings** — `func.params`, `clause.args`,
  `rest_param` (scoped at construction by 249.5b/d): `check.rs:3158`, `:7765`,
  `:12831`.
- **Every other bind keys BARE** via `ident.as_str()` — `let`/`match`-arm/pattern
  binders: `check.rs:5824`, `:6319`, `:6606`, `:10548`, `:10586`, `:10645`,
  `:11430`, and `function/infer.rs:46` (the fn-param path).
- **The single resolution site keys BARE** — `check.rs:3397`:
  `locals.get(ident.as_str())`, `None → fresh.fresh()` (silent-by-intent, 236.1).

So a macro-generated body's scope-tagged param reference computes the BARE name,
misses the SCOPED bind, and falls to the permissive fresh-var arm → **the param's
declared type is lost, and a return-type mismatch the checker would otherwise catch
(`infer_defclause`, `check.rs:7955`) is silently suppressed.** The disconfirming
probe proves it: a hand-written defclause returning its `:i64` param as `:bool` is
REJECTED; the byte-identical macro-generated form is ACCEPTED.

This is NOT a runtime defect (runtime resolution is correct since 249.5b/d) — it is
a **type-checking PRECISION hole on macro-generated code**: the checker isn't
actually checking macro output, it's rubber-stamping it through fresh vars.

## What it delivers (one keying policy across both passes)

**The check-pass `locals` keys by `env_key`, identical to the runtime
`Environment`.** One keying derivation, used by both passes — no second way. The
scope-tagged param resolves to its declared type; the suppressed errors fire.

Behavior on **user code is unchanged**: `env_key` of a bare (unscoped) identifier
is exactly `as_str()`, so every non-macro bind and lookup computes the same key it
does today. The only behavior delta is macro-generated code, which moves from
silently-permissive to correctly-checked.

## The contract decision (pinned)

**`check::infer`'s symbol resolution and every `Identifier`-keyed `locals` bind
route through `crate::scope::env_key`.**

- Lookup — `check.rs:3397`: `locals.get(&crate::scope::env_key(ident))`.
- Each `(B)` bind: `crate::scope::env_key(ident)` instead of `ident.as_str()` /
  `.to_string()`, applied where the `Identifier` is in scope.
- The `(A)` already-env_key'd-string binds stay as-is (the invariant they must
  satisfy: a string in `locals` is always the `env_key` of the identifier that
  should match it — `func.params`/`clause.args` satisfy it at construction).

**This revises Stone 249.5d's `infer.rs:46` choice.** 249.5d kept `infer_fn`
bare-consistent (`as_str`) to match the then-bare lookup; 249.5e flips the lookup
to `env_key`, so `infer.rs:46` flips to `env_key` too. Coherent progression:
249.5d unified the *representation* (ArgSpec carries Identifier); 249.5e unifies
the check *keying* with the runtime.

## Sites (grounded — crawl + direct reads this session)

**Lookup / read sites → `env_key`:**
| Site | Today | After |
|---|---|---|
| `check.rs:3397` (`infer`, the canonical lookup) | `locals.get(ident.as_str())` | `locals.get(&env_key(ident))` |
| `check.rs:11397` (`check_clause`, `matches?` guard) | `locals.contains_key(var)` | `contains_key(&env_key(ident))` — recover `ident` by re-matching `left` |

**`(B)` Identifier-keyed binds → `env_key` (Identifier in scope at each):**
`check.rs:5824` (`infer_match` arm), `:6319` (`pattern_coverage`), `:6606`
(`check_subpattern`), `:10548` (`process_let_binding`, kv binder), `:10586`
(let tuple-destructure — apply at the `:10564` collection point, retaining the
Identifier), `:10645` (let hash-destructure — apply in the `:10639` match arm),
`:11430` (`matches?` ?var — recover Identifier via `left`), `function/infer.rs:46`
(`parse_fn_signature_for_check_diag` → `infer_fn`).

**`(A)` already-env_key'd — LEAVE:** `check.rs:3158`, `:7765`, `:12831`.

## Out of scope = rejected (affirmative cuts)

- **The deadlock-diagnostic walker** (`check.rs:9854-9889`, `check_let_for_scope_deadlock_inferred`).
  It builds its own bindings, keying them BARE (`id.as_str()`) and reading them
  BARE (`extended.get(name)`) — **bare-internally-consistent**, so it has no
  bind/lookup mismatch. It only ever looks up its OWN bare-collected let-binding
  names (Thread/Sender channels), never resolves a param. No hygiene defect; no
  change. (STOP if the executor finds it actually resolves a scoped param.)
- **The struct-field bind** (`check.rs:10734`, `(C)`): a struct field *name*
  looked up by canonical field string, not a scoped identifier. Stays bare.
- **The broad flat-`src/*.rs` → homes migration** (check.rs is still flat) — the
  pending arc-170/109 work; untouched.

## Probe (committed, RED at HEAD)

`tests/probe_check_scoped_param_resolution.rs` — two tests:
- `handwritten_defclause_ret_mismatch_is_caught` (CONTROL) — a hand-written
  defclause returning its `:i64` param as `:bool` is REJECTED. GREEN at HEAD and
  after (proves the observable is valid; isolates macro-scoping as the sole var).
- `macro_generated_defclause_ret_mismatch_is_caught` (THE BUG) — the byte-identical
  MACRO-generated form must ALSO be rejected. RED at HEAD (silently accepted via
  fresh var); GREEN after the fix. `#[ignore]`'d for STRIKE-READY; the strike
  un-ignores it.

Verified this session: control passes, bug test RED, at HEAD.

## Decomposition

One atomic strike. The keying change must land uniformly — a half-converted check
pass (some `env_key`, some bare) is the exact mixed-keying inconsistency this stone
eliminates; an intermediate state is worse than either endpoint. ~10 sites, uniform
mechanical change (`as_str` → `env_key` at a bind; the one lookup), with three
sites needing the Identifier retained one line earlier (cast-mapped).
