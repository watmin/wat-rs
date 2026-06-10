# DESIGN — Stone 251.4: `:-` the annotation arrow (`<-`/`->` → `:-`)

**Status: 251.4a STRIKE-READY (drawn 2026-06-10, on a grounded crawl of the binder/return
arrow sites). Probe RED at HEAD before build.**

Predecessor: 251.3a (parametrics-as-forms). Home: `src/argspec/` + `src/function/`.

## The move (core.typed parity)

core.typed annotates with `:-` in BOTH param and return position. wat today uses `<-` (param
binder) and `->` (return). 251.4 retires BOTH arrows in favour of the single `:-` annotation
keyword (the pivot's parity refinement: `->` retires ENTIRELY; a NEW `:->` is the
function-TYPE arrow used only inside a type expression). Keywords return to their role as the
annotation marker — clojure-honest (`:-` is a keyword, a syntactic marker inside the form).

- Binder: `[x <- :i64]` → `[x :- :i64]`.
- Return: `(defn f [...] -> :i64 body)` → `(defn f [...] :- :i64 body)`.
- (later) Fn type inside a type expr: `[f :- [i64 :-> i64]]`.
- (later) `(ann-form expr type)` — expression-position ascription.

## The disk (grounded)

- Binder arrow: `src/argspec/parse.rs:166` — `if !triple[1].is_bare_symbol("<-")`. Slot 1 of a
  `[name <- :Type]` triple is the bare Symbol `<-`.
- Return arrow: `src/function/parse.rs:160` — `if !sig[1].is_bare_symbol("->")`. Sig slot 1 is
  the bare Symbol `->`.
- `:-` / `:->` / `ann-form` do NOT exist yet (grep clean).
- `:-` is a KEYWORD token (`WatAST::Keyword(":-")`); the arrows are bare Symbols — structurally
  distinct, no lexer change. Binder `:-` (inside `[...]`, triple slot 1) and return `:-` (sig
  slot 1, after the `[...]`) are different structural positions → no ambiguity.

## Sub-stones

- **251.4a (this strike)** — `:-` DUAL-READ in binder + return. At the two arrow sites, accept
  the `:-` keyword in addition to the `<-`/`->` bare symbols. Minimal enabler; mirrors
  251.2a/251.3a (add capability, keep the old surface). Probe RED→GREEN.
- **251.4b** — `(ann-form expr type)` expression ascription (a new form: check + runtime, or a
  macro over the engine). Separate strike — it's a new evaluation form, not an arrow swap.
- **251.4c** — `:->` function-TYPE arrow inside type expressions (folds into the type-form
  grammar from 251.3). Separate strike.

## The probe (RED at HEAD) — 251.4a

`tests/probe_arc251_stone4_annotation_arrow.rs`:
- **C01 (RED→GREEN):** a defn with `:-` in BOTH binder and return position type-checks,
  load-bearing (i64 arithmetic on the param). RED at HEAD (the binder rejects `:-`; only `<-`
  is accepted). GREEN post-251.4a.
- **C02 (dual-read):** the `<-`/`->` spelling still type-checks (PRESERVATION; arrows
  HARD-CUT only at 251.5).

## Out of scope (named)

- Corpus `<-`/`->` → `:-` sweep (337 + 313 sites) → the **unified 251.5 sweep** (churn-once).
- `ann-form` → **251.4b**; `:->` fn-type arrow → **251.4c**.
- Hard-cut of the `<-`/`->` arrows → **251.5** (one-canonical-path).
