# BRIEF (SCOUT) — can a Rust-minted per-op type ALIAS be named by the defservice macro?

> **This is a disconfirming probe, not a build.** One question, answered by a run. It either
> survives or it dies on that one point. Ship the probe and the finding; ship no feature.
>
> **Origin:** the parametric protocol landed (`1ac85d96`) at the cost of a rule — a parametric
> surface's `:messages` must spell the surface's params *even vacuously* (`PutRequest<T>` where no
> field mentions `T`). Builder: *"we made extensive use of aliases to make parametrics vanish…
> maybe the defservice macro can mint them?"* Grounding says the macro can't (it never sees the
> surface), but **Rust at surface registration can** — and this probe decides whether the checker
> can see what Rust mints there.

## The defect this would cure (grounded, not asserted)

The message type NAME is **declared once and re-derived once**:

```clojure
;; DECLARED — the surface's :features names the request type outright
(put [self <- :Box<T>  req <- :Box::PutRequest<T>] -> :Box::PutResponse<T>)

;; RE-DERIVED — wat/service.wat rebuilds it by concatenation, having only the
;; :satisfies keyword (`proto-str`, :244). It never reads :features.
req-ty  (interpolate "{b}::{v}Request{p}" :b proto-base :v variant-pascal :p proto-tp)
```

Two sources, one truth; the derived one cannot see the message's real arity, so it re-attaches the
surface's own params and the user must spell them to match. Same class as arc 278's three generics
bugs: **a reconstructed spelling racing a declared one.**

## The proposed shape (what the probe is testing the feasibility of — do NOT build it)

Rust, at surface registration, mints one alias per op; the macro then writes one uniform spelling
and never guesses arity:

```clojure
(defrecord :Box::PutRequest [item <- :i64])            ;; user writes what is HONEST — bare
(typealias :Box::put/Request  :Box::PutRequest)        ;; Rust mints, from the declared sig
(typealias :PCache::get/Request :PCache::GetRequest<K,V>)   ;; …or parametric, as declared
:Box::put/Request                                      ;; the macro writes ONLY this
```

## Read in order (grounded this session — verify, do not trust the numbers blind)

1. **`src/types.rs:2752`** — `d.extend(synthesize_surface_protocol(surf, env, acronyms, &decl_span)?)`.
   This is the surface-registration site and it already extends a `Vec<TypeDef>`, so pushing a
   `TypeDef::Alias` needs no new plumbing. **`:2744`** mints `build_surface_forms_carrier` in the
   same breath — the precedent that Rust already mints per-surface artifacts here.
2. **`src/types.rs:285` (`AliasDef`) + `:376` (`TypeDef::Alias`)** — the alias declaration shape.
   **`:4257-4282`** is `resolve_alias`'s substitution (note the `alias.type_params.is_empty()`
   branch — parametric aliases take a different path); **`:4373`** is the `CyclicAlias` guard.
3. **`src/freeze.rs:618-619`** — the phase order: step 4 `register_defmacros` → `expand_all`;
   step 5 `register_types`. Check is step 8. So a type minted at step 5 is *in principle* visible
   at step 8 — that is the easy half.
4. **`wat/service.wat:244`** (`proto-str`) and **`:268-276`** (`proto-base` / `proto-tp`) — where
   the macro's only knowledge of the surface begins and ends. **`:891-893`** is one of the eleven
   derivation sites (the client-method `req-ty`).
5. **`wat/service.wat:1749-1767`** — the working precedent for the ordering question: the macro
   writes `(:S::surface-forms)`, a symbol Rust emits at registration, at expand time.

## ★ THE CRUX — the question the probe exists to answer

The easy ordering (step 5 mints, step 8 checks) is not the question. **The question is ordering
*within* step 5.** The service's own synthesized types reference the alias in field positions, and
they are registered in the same pass as the surface that mints it. So:

> **Does a `TypeDef::Alias` minted during `register_types` resolve when a *later declaration in
> that same pass* names it — and does a PARAMETRIC alias substitute its args correctly through
> `resolve_alias`?**

`surface-forms` does not answer this: it is a **runtime `defn` call**, resolved at eval. An alias
is a **type reference**, resolved by the checker through `resolve_alias`. Different phase, different
machinery. That distinction is why this is a probe and not a plan.

## The probe

Land ONE probe under `wat-scripts/scratch-pad/` (loader-gated, so it must be GREEN or deleted) plus
whatever minimal Rust it needs, that answers the crux in isolation:

1. From the surface-registration site, mint a `TypeDef::Alias` named on the `<Surface>::<op>/Request`
   pattern, whose target is the request type the surface's `:features` **actually declares**.
2. Reference that alias name from a wat form that goes through the checker — the load-bearing case
   is a form registered in the *same* `register_types` pass as the surface.
3. Do it twice: once **monomorphic** (`:Box::put/Request` → `:Box::PutRequest`) and once
   **parametric** (`:PCache::get/Request` → `:PCache::GetRequest<K,V>`), because `resolve_alias`
   takes a visibly different branch on `type_params.is_empty()`.

`target/release/wat --check <file>` is the fast per-file arbiter (~0.2s); read its output, not `$?`
through a pipe. `macroexpand` first if any macro output confuses you — read what was EMITTED before
theorising.

## STOP triggers — each is a rejection; report and ship nothing further

1. **If the alias does not resolve for a same-pass declaration** — STOP and report the exact
   diagnostic. That is a complete, valuable answer: it kills the design. Do not reorder
   `register_types`, do not add a second pass, do not special-case the lookup.
2. **If the monomorphic alias resolves but the parametric one does not** — STOP and report both,
   with the `resolve_alias` branch that diverges. A half-working alias is a finding, not a feature.
3. **If answering the crux requires touching more than the surface-registration site + the probe**
   — STOP and report the blast radius before spending it.

## Out of scope — rejected, not deferred

The eleven `wat/service.wat` derivation sites, the message-params lock in `src/types.rs`, the two
existing parametric gates, and any corpus change. **This scout changes no user-facing behavior.**
The vacuous-`<T>` rule stays exactly as it shipped in `1ac85d96` regardless of the answer.

## Gate

- The probe is GREEN (or, if the crux answers NO, the probe is **deleted** and the finding is the
  deliverable — a red probe left in `wat-scripts/` fails the loader gate).
- `cargo build --release` clean.
- `cargo nextest run --release` — the **Summary line, verbatim**. Floor: **4180 passed, 314
  skipped**.
- Run everything in the FOREGROUND. Do not background a command and return.
- **Do NOT commit.** The orchestrator weighs by their own re-run and commits.

## Your report

The crux answered YES or NO, with the diagnostic quoted verbatim either way; which `resolve_alias`
branch each case took; the probe's path; the verbatim Summary line; any STOP.
