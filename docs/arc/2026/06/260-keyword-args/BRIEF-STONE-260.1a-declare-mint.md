# BRIEF — Stone 260.1a: `defn` mints a kwargs record from `& [argspec]` (declare side)

> Single-hop sonnet Shadowdancer. Do NOT spawn sub-agents. Do NOT use git worktrees. Work ONLY in
> `/home/watmin/work/holon/wat-rs`. FIRST run `pwd`; if not there, `cd`. `git -C <path>` for git. Commit
> NOTHING — the orchestrator weighs + re-runs the gate. Grounded against HEAD `14108668`. Full design:
> `DESIGN-STONE-260.1-user-fn-kwargs.md` + `REALIZATIONS.md` (READ both). **This is a change to `defn`, a
> CORE macro every fn uses — backward-compat is sacred; the full suite is the guard.**

## The work (one paragraph)
A fn's keyword arguments are a **typed record minted by `defn`**. When a `defn` param vector ends with a
**kwargs section `& [argspec]`** (a `&` followed by a `[…]` Vector of binder-triples — the SAME argspec
syntax as the main params, nested ONE level), `defn`: (1) mints `(:wat::Record::def :<name>::Kwargs <fields>)`
from that argspec; (2) reshapes the `fn` so its last param is a single hidden record param of that type;
(3) **destructures** the fields into the body scope (clojure `& {:keys}` — the body references `port`/`tls`
by name); (4) emits `(do <record-def> (def name (fn <reshaped> -> <ret> <wrapped-body>)))`. PURE WAT — a
`defn`-macro change only; `fn` receives a normal positional signature and needs NO change. No call-site
sugar in this stone (that's 260.1b); the gate uses the explicit-record call form.

## Build — `wat/core.wat`, the `defn` macro (~line 188, currently `(def ~name (:wat::core::fn ~@rest))`)
Add a kwargs branch. `rest` is the fn's `(params-vec -> ret body…)`; `params-vec` = `(first rest)`.
1. **Detect** a trailing kwargs section in `params-vec`: the last two elements are a `&` marker symbol
   followed by a **Vector** node (`[…]`). (Contrast: `&` followed by a **symbol** = variadic rest →
   PASS THROUGH unchanged, `fn` handles it. Only `&` + a Vector is kwargs.)
2. **Parse the inner argspec** (reuse the same binder-triple reading the main params use): a flat list of
   `name <- :type` triples. **DISALLOW a nested `&` inside it** (no kwargs-in-kwargs / rest-in-kwargs) →
   `macro-error` ("kwargs section is flat: no nested & — one level"). One level only.
3. **Mint** `(:wat::Record::def :<name>::Kwargs <fields>)` where `<name>::Kwargs` = `keyword/from-string
   (concat <name-str> "::Kwargs")` and `<fields>` is the inner argspec verbatim (binder-triples →
   Record::def field vector; mirror defservice's `request-records` Record::def emission, `wat/service.wat`).
4. **Reshape** `params-vec`: drop the `& […]` tail; append a final positional param
   `<kw-sym> <- :<name>::Kwargs` where `<kw-sym>` is a HYGIENIC hidden binder (a `symbol-node`, e.g.
   "__kwargs__" — must NOT collide with user names; mirror defservice's `symbol-node` hygiene for
   generated binders, service.wat `discard-sym`/`r-sym`).
5. **Destructure** into the body: wrap the original body as
   `(:wat::core::let [field1 (:<name>::Kwargs/field1 <kw-sym>)  field2 (…)  …] <orig-body>)`
   — one accessor per kwargs field, binders are the field NAMES (so the body sees them by name).
   Field binders use `symbol-node` too (they're generated let-binders).
6. **Emit** `(:wat::core::do <record-def> (:wat::core::def ~name (:wat::core::fn <reshaped-params> -> <ret> <wrapped-body>)))`.
   Backward-compat: NO `& […]` kwargs section → defn behaves EXACTLY as today (`(def name (fn ~@rest))`).

## Out of scope (affirmative — later stones)
- The `& opts <- :SomeRecord` **named-record** form (no mint) — 260.1a-named or later; THIS stone is the
  inline `& [argspec]` mint only.
- The inline `:k v` / `{map}` **call sugar** — 260.1b (companion-macro vs check/eval fork).
- Defaults / `:or` / optional kwargs — later; all declared kwargs are required (records are total).

## Rooms (read in order)
1. `DESIGN-STONE-260.1-user-fn-kwargs.md` + `REALIZATIONS.md` (the contract + why).
2. `wat/core.wat:188` (the `defn` macro to extend) + the `&`-rest handling in the arith macros / `cond`
   for the `& <symbol>` pass-through shape.
3. `wat/service.wat` — defservice as the MINT+hygiene pattern: `request-records` (`:wat::Record::def`
   emission ~115-165), the `symbol-node` hygiene for generated binders (~467), the body `let`-wrap (~300-340).
4. `wat/Record.wat:8-110` (`:wat::Record::def` — what you emit) + the generated accessor `:<Rec>/<field>`.
5. `tests/probe_arc260_decl_kwargs_minted_record.rs` (the GATE).

## STOP triggers (halt + report — rejection criteria)
1. STOP if the macro-eval fence can't do the params-vec walk / `&`-then-Vector detect / the inner argspec
   read (report what it rejected — defservice does heavier AST work, so it should be reachable).
2. STOP if minting `:<name>::Kwargs` or the hidden `<kw-sym>` collides with a user/generated name.
3. STOP if ANY existing `defn` (no kwargs section) changes meaning — the full baseline is the guard; a
   broken core `defn` reds the whole suite (that's the safety net, not a workaround to ship around).
4. STOP if the destructure-let breaks body hygiene (a field name shadowing something the body needs).

## Gate (orchestrator re-runs — BROAD, because defn is everywhere)
- `cargo build --release -p wat` → clean.
- `cargo test --release -p wat --test probe_arc260_decl_kwargs_minted_record -- --include-ignored --test-threads=1` → GREEN (1), `#[ignore]` removed.
- `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"` → **929/36, ZERO new** (defn is used by ~everything — this is the backward-compat guard).
- `cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"` → ~893/4 baseline.
- `cargo test --release -p wat --test test -- 2>&1 | grep "test result"` → the wat deftest corpus, no new failures.
- The headline `tests/probe_arc260_keyword_args.rs` (inline `:k v`) STAYS `#[ignore]` RED (that's 260.1b).

Report: exact `wat/core.wat` diff; how you detect `& [argspec]` vs `& <symbol>`; the mint + reshape +
destructure-let + hygiene approach; the pasted gate results from YOUR OWN runs; any STOP hit. Do not commit.
