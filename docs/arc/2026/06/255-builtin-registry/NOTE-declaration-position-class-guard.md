# NOTE — declaration-form-at-eval guard: a registry position-class property (DEFERRED to 255-wrap)

**Status: LEGIT DEFERRAL (2026-06-24), surfaced during arc 291 kwargs-`start`.** This is the *correct*
kind of deferral — the proper cure depends on the registry that 255 is building, and the work that
surfaced it (kwargs-`start` / macros³) is *actively unblocking that registry being built*. We do NOT
hand-roll the cure now; we record exactly how to reproduce it and exactly what to do once the registry
exists, so the next self cannot forget. Pairs **`NOTE-purity-is-definition-time-queryable-metadata.md`**
(same shape: a form property the registry should own and answer at definition time).

## The failure domain (what we are annihilating, once we can)

A **declaration form** (a registration-only special form: `def`, `recordtype`, `defmacro`, `defclause`,
`defprotocol`, `extend-type`, …) that reaches the runtime **evaluator** must be refused with a **precise,
structural** diagnostic naming the form. Today some are guarded and some are not, so an unguarded one
**leaks to eval and bottoms out as a cryptic, misleading error** — wrong span (points at stdlib macro
internals), wrong symbol (an incidental field name), no hint of the real cause. The builder's words:
*"make the diagnostic such that we never encounter this again — annihilate this failure domain forever."*

## How to reproduce (DO NOT LOSE THIS)

**Reproducer on disk:** `tests/probe_kwargs_emitted_by_macro.rs` — a wrapper macro emits a kwargs `defn`
(the defservice shape). The emitted `Record::def` → `recordtype` leaks to eval. At HEAD it fails:

```
(:t::via-kv) raised: RuntimeError {
  span: core.wat:388:27-68,                 ← MISLEADING: the Record::def TEMPLATE in the stdlib
  kind: UnboundSymbol("a")                   ← MISLEADING: "a" is an incidental field-name symbol
}
```

The *true* event is: a definitional form was **evaluated** instead of **registered**. The generic
`UnboundSymbol` is three layers downstream of the real invariant violation (a declaration reached eval).

Run: `cargo test --release -p wat --test probe_kwargs_emitted_by_macro`

## The two problems are DISTINCT (decomplected — do not conflate again)

| | what it is | home | needed for |
|---|---|---|---|
| **A — the registration bug** | `recordtype` is not *registered* when a macro nests it deep (`register_runtime_defs_form` recurses into `do` but has **no `recordtype` arm**, runtime.rs:1669) → it leaks to eval at all | **arc 291** (freeze-time registration) | kwargs-`start` to *work* |
| **B — the diagnostic (THIS NOTE)** | a declaration form reaching eval → loud, registry-backed refusal | **arc 255-wrap** (this note) | never being *misled* again |

Fixing **A** makes the form never reach eval, so this specific cryptic error stops firing regardless. **B**
is the permanent net for *any future* leak (any cause). The reproducer exercises both; once A lands, write a
dedicated B-probe that forces a declaration form to eval directly (e.g. a raw `(:wat::core::recordtype …)` in
expression position) so B's RED test does not depend on A's bug.

## Why a hand-rolled list is the WRONG cure (the antipattern to refuse)

A `const DECLARATION_FORMS: &[&str]` in `runtime.rs` is the **hand-maintained-index-that-drifts**
antipattern (extirpare): every new declaration form needs someone to remember to add it. The existing
per-keyword guards are already that sin, minimally:
- `src/runtime.rs:3942` — `:wat::core::def` → `DeclarationInExpressionPosition`
- `src/runtime.rs:3950` — `:wat::core::defclause` → `DeclarationInExpressionPosition`

`recordtype` (and the rest) were simply never added. Adding them by hand just grows the drift-prone list.

## What we MUST do once the registry is built

The registry already has the bones (`src/intrinsic/mod.rs:47`): `Kind { Macro, Fn, Intrinsic, SpecialForm }`,
fed by `#[wat_special_form("<fqdn>")]` via `inventory` (`mod.rs:271-288`). So the substrate already knows
`def`/`recordtype`/`defmacro` are `SpecialForm`s. **But `SpecialForm` is necessary, not sufficient:**
`if` / `let` / `do` / `match` / `fn` / `quote` are *also* SpecialForms and are **legal at eval**. The missing
property is finer — **position-class**:

- **Declaration** — registration-only; must NEVER reach eval (`def`, `recordtype`, `defmacro`, `defclause`,
  `defprotocol`, `extend-type`).
- **Expression** — legal at eval (`if`, `let`, `do`, `match`, `fn`, `quote`, `quasiquote`).

**The steps (once the registry can carry/answer this):**
1. **Add a `position-class` (Declaration | Expression) property** to the SpecialForm registry entries. Each
   `#[wat_special_form]` declares it ONCE, at definition — definition-time queryable metadata (the
   `NOTE-purity-...` pattern). The registry becomes the single source of truth; no hand-list.
2. **Query it at the eval seam** — `dispatch_keyword_head` (`src/runtime.rs:3763`), before the giant match:
   if `head` is a SpecialForm with position-class `Declaration` → return `DeclarationInExpressionPosition`
   (consider a richer variant naming the form + a hint: *"a definitional form reached the evaluator — it was
   not registered; if emitted by a macro, ensure it is lifted to a registerable position"*). The error variant
   exists: `RuntimeErrorKind::DeclarationInExpressionPosition(String)` (`src/value/signal.rs:149`; Display
   `:423`; EDN render `src/runtime_error_edn.rs:81`).
3. **Retire the hand-rolled per-keyword arms** (`runtime.rs:3942` def, `:3950` defclause) — replaced by the
   data-driven query. Amend with recognition (note the retirement; don't silently delete).
4. **Gate:** a B-probe asserting the precise diagnostic fires (form named, Declaration class, honest span) —
   RED until the registry property exists, GREEN after. The whole *class* is annihilated by construction: any
   declaration form, present or future, that reaches eval is refused, because the registry knows what it is.

## Pairs
`NOTE-purity-is-definition-time-queryable-metadata.md` (the sibling property) · `src/intrinsic/mod.rs`
(the registry + `Kind` + `#[wat_special_form]`) · `tests/probe_kwargs_emitted_by_macro.rs` (the reproducer) ·
arc 291 kwargs-`start` (the work that surfaced it + is unblocking the registry) · the project ethos
*"the substrate must force our hands relentlessly toward the idealized state"* (a leaked definition must be a
loud, precise, registry-grounded refusal — never a cryptic `UnboundSymbol`).
