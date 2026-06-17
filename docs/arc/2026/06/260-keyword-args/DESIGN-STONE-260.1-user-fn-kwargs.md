# DESIGN-STONE 260.1 — user-fn keyword arguments = a minted typed record

> Re-pinned 2026-06-17. **SUPERSEDES the earlier draft of this file** (which pinned a "usage-detected
> all-or-nothing reorder" disambiguation — wrong; clojure-guiding means OPT-IN, and opt-in dissolves the
> ambiguity). Confirmed via four-questions + the REALIZATIONS entry. Grounded against HEAD `f21c1700`.

## The confirmed contract — kwargs ARE a typed record

A fn opts into keyword arguments by declaring a **kwargs section** in its signature; that section is a
**typed record** the definer mints (the rs-1 move). Keywords are first-class *values* in wat, so kwargs
MUST be opt-in (a non-declaring fn is positional-only; `:foo` to it is a keyword value) — opt-in is what
makes it unambiguous.

```clojure
(:wat::core::defn :user::connect
  [host <- :wat::core::String                 ; positional
   & {port <- :wat::core::i64                 ; the kwargs section → minted record :user::connect::Kwargs
      tls  <- :wat::core::bool}]
  -> :wat::core::nil  (… port … tls …))
```

- **`& {field <- :T …}`** (NEW) = the kwargs record, **minted** by `defn` as `:<name>::Kwargs`. (Today
  `& name <- :T` = variadic rest; the two are mutually exclusive — one `&` slot, exactly as clojure's
  `& {:keys}` *is* the rest position.)
- **`& opts <- :SomeRecord`** = name an EXISTING record as the kwargs (the sharing case). Mint-vs-name is
  identical to rs-1's `:state [fields]` vs `:state :Record`.
- **Four-questions verdict (typed record, not untyped map):** the hard constraint is wat's no-magic /
  typed-record law ([[feedback_no_magic_that_lets_llm_fake_correctness]]) — an untyped `Map<K,V>` is a
  structureless bag (the magic affordance); a typed record carries per-key types, wrong shape uncompilable.
  Clojure-guiding for the shape, wat-typed for the substance.

### Three call forms — all build/pass that one record
```clojure
(:user::connect "h" :port 443 :tls true)               ; inline :k v (sugar)
(:user::connect "h" {:port 443 :tls true})             ; map literal (arc 257)
(:user::connect "h" cfg)                               ; pass the record value (Ruby ** collapses to this)
(:user::connect "h" (:user::connect::Kwargs 443 true)) ; explicit construct (no sugar — just a record arg)
```
Validation = the opts-map discipline (unknown/missing/duplicate key → named compile error). It transports
across loci for free (it's a record → arc-272 rails).

## Decomposition (the realization clarified the seam)

- **260.1a — declare side + explicit-record call (THE FOUNDATION, fully decided — build FIRST).**
  `defn` detects `& {fields}` in the param vector, mints `(:wat::Record::def :<name>::Kwargs [fields])`
  (or uses the named record for `& opts <- :Record`), and reshapes the `fn` so its last param is that
  record. The **explicit-record call** then works with ZERO new call machinery — it's record construction
  + a normal call. Proves: mint, share-by-name, transport-for-free. All-macro (defn is a macro), no Rust.
- **260.1b — the inline `:k v` (and `{map}`) call sugar (the open fork — NEXT, not this stone).**
  A normal macro fires on its OWN head; `(connect …)` has a fn head, so no macro intercepts it (the
  phase-order wall, rs-1). Two paths to dial in: (a) `defn` emits a **companion macro** `connect` that
  scoops trailing `:k v` → `(:<name>::Kwargs …)` and calls the impl — all-wat, but a macro-isn't-a-value
  (no higher-order under the sugary name); (b) **check/eval** recognizes+builds the record at the resolved
  call site — Rust, keeps fn-as-value. Four-questions this fork before drawing 260.1b.

## 260.1a build (where it lands)
`wat/core.wat` `defn` (the macro, ~188 — `(def name (fn ~@rest))`): add a branch — if the param vector's
tail is `& {…}`, (1) extract the field vector, (2) mint `(:wat::Record::def :<name>::Kwargs <fields>)`
(mirror defservice's Record::def emission), (3) reshape the param vector replacing `& {…}` with a normal
final param `kwargs <- :<name>::Kwargs`, (4) emit `(:wat::core::do <record-def> (def name (fn <reshaped>)))`.
Backward-compatible: no `& {…}` → defn unchanged. (The `& opts <- :Record` named form: no mint, just the
final record param.)

## STOP triggers
1. STOP if the `defn` macro can't parse the param vector + detect `& {…}` in the macro-eval fence (it
   should — defservice does heavier AST work; confirm `ast->children`/`with-children`/the `&`-tail read).
2. STOP if minting `:<name>::Kwargs` collides with an existing generated/user name → report.
3. STOP if reshaping the sig breaks an EXISTING `defn` (no `& {…}`) — the full suite is the guard; zero
   existing fn may change meaning.
4. STOP if `& {…}` and `& rest` can't be kept mutually exclusive cleanly → report.

## Gate
- A probe (`tests/probe_arc260_decl_kwargs_minted_record.rs`, to write + commit RED): `defn` with
  `& {fields}` → `:<name>::Kwargs` record exists + an explicit-record call round-trips; a `& opts <- :Rec`
  named form works; an existing `& rest` fn still works.
- The arc's headline probe `tests/probe_arc260_keyword_args.rs` (inline `:k v`) stays RED/#[ignore] until
  260.1b ships the sugar.
- lib 929/36, nursery 893/4 (zero new — no existing defn changes meaning).
