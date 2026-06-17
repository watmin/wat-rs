# Arc 274 — `fresh-symbol`: capture-proof binders for computed (program-body) macros

> Opened 2026-06-17. Surfaced by an **L3 nit** in arc-260.1a: defn-kwargs's hidden binder `__kwargs__`
> is a fixed `symbol-node` name, capturable if a user names a param `__kwargs__`. Grounding revealed the
> real gap: wat's Racket **sets-of-scopes hygiene** (Flatt 2016) is applied on the **quasiquote-template
> path** (`walk_template` stamps `macro_scope` on template symbols) but **NOT on the program-body /
> macro-eval path** (`expand.rs:332` — `macro_scope` unused; hygiene there is only Gate E's default-deny
> *refusal of literal binders*). So a **computed** binder (`symbol-node "name"`) emits a plain, unscoped
> symbol → capturable. Grounded against HEAD `711d8914`.

## The flaw (what we annihilate)
A program-body macro that invents a binder must build it via `symbol-node` (Gate E refuses literal
binders). But `symbol-node "__kwargs__"` produces a bare, unscoped `__kwargs__` — it can capture / be
captured by a user's `__kwargs__`. There is **no `gensym`/`fresh-symbol` primitive** (grep-confirmed) and
no scope-stamping on the computed path. The capture class is *alive* for every computing macro
(defservice, defn-kwargs, any future one), patched today only by reserved-looking names — a convention,
the bottom rung.

## The fix (four-questions verdict — see below): `fresh-symbol`, backed by sets-of-scopes

`(:wat::core::fresh-symbol <base-name-string>)` → a symbol value whose `Identifier` carries a **fresh
unique `ScopeId`** (`add_scope(fresh_scope())`, `src/scope/`). Its display base is `<base-name>` (readable),
but its scope is unique, so at bind+lookup (the scoped-key resolution the defclause fix already uses) it
**cannot collide with any user symbol** — capture is structurally impossible, by construction. A macro uses
the SAME returned value for both the binder and its references, so they share the unique scope (match each
other, never the user). This is exactly Racket's `generate-temporaries`: readable, capture-proof temps for
the computed/iterated binders that template hygiene doesn't auto-cover.

**It dissolves the "fresh-symbol vs sets-of-scopes" fork: `fresh-symbol` IS sets-of-scopes, exposed as a
primitive** — the same `fresh_scope()`/`add_scope` machinery the quasiquote path uses, made callable from a
computing macro. Not a gensym counter; the real hygiene engine.

### Four-questions (hard constraint first)
- **Hard constraint — annihilate the capture class STRUCTURALLY** (extirpare top rung; the builder's
  "annihilate this problem"). A fresh-*scoped* symbol cannot capture, by construction. ✅ fresh-symbol PASSES.
- **(rejected) auto sets-of-scopes over the whole program-body output:** Obvious YES, Honest YES, but
  **Simple = NO** — Flatt hygiene tracks scopes on identifiers as they *flow*; a computing macro *rebuilds*
  names from strings (`symbol-node fname-str`), losing provenance, and user-meaningful reconstructed names
  (`port`/`tls`) MUST stay visible. Auto-distinguishing invented vs reconstructed in computed output is
  research-hard. Disqualified on Simple.
- **`fresh-symbol`:** Obvious YES (familiar; Racket precedent) · Simple YES (reuses `fresh_scope`/`add_scope`/
  scoped-key resolver — a bounded primitive) · Honest YES (real sets-of-scopes, not discipline) · Good UX
  YES (capture-proof readable temps; user-facing reconstructed names correctly stay plain `symbol-node`).

## Contract (pin)
- **`(:wat::core::fresh-symbol <base :wat::core::String>)` → a symbol value** (WatAST Symbol /
  `symbol-node`-shaped) whose `Identifier` has base = `<base>` and a fresh unique `ScopeId`.
- **macro-eval callable** (on the `is_pure_total` allow-list — a macro needs it; the "does a macro need it"
  boundary) AND evaluable (returns the symbol value).
- Capture-proof: the unique scope makes its scoped key distinct from any user binder of the same base name,
  at bind AND lookup.
- Name/arity is an intueri candidate (`fresh-symbol` / `gen-temp` / `gensym'`); base-arg pinned (readable).

## Decomposition
- **274.1 — the `fresh-symbol` primitive** (this stone): the intrinsic + scope-stamp + the capture probe.
- **274.2 — defn-kwargs adopts it** (arc-260.1a follow-on): `(symbol-node "__kwargs__")` → `(fresh-symbol "kwargs")`;
  the L3 nit annihilated structurally. (defservice's generated binders can adopt it too, as a sweep, if wanted.)

## Gate / probe
`tests/probe_arc274_fresh_symbol_no_capture.rs` (write + commit RED): mirror `probe_macro_hygiene_capture.rs`
but on the program-body path — a computing macro binds `(fresh-symbol "t")` to 100 and adds the user's
unquoted arg; the caller passes its own `t`=5. HYGIENIC → 105; CAPTURED → 200. RED at HEAD (`fresh-symbol`
unknown). GREEN when 274.1 ships. Plus: lib 929/36, nursery 893/4 (zero new).
