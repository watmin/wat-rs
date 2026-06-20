# NOTE — purity is a DEFINITION-TIME property; it must be queryable metadata (the arc-283 sweep demanded it)

> Surfaced 2026-06-17 by the arc-277 sweep (the self-hosted linter rewriting its own corpus). Builder:
> *"we need to make a note that we declare a function pure or not at definition time — there's an arc to
> make all intrinsics have queryable metadata — this just demanded that."* That arc is THIS one
> (255 — builtin-registry / queryable intrinsic metadata).

## What demanded it

The sweep's concat→format auto-fix rewrote `(string::concat …)` calls into `(:wat::core::format …)`.
`format` is a **macro** (`wat/core.wat:545`). Many of the rewritten concats live inside **defmacro
bodies** (building keyword names at EXPAND time — `(concat fqdn-str "::Op")` etc.). The macro-eval
purity gate (arc 249 stone 249.2b-i, default-deny F5) refused them at load:

> `keyword head :wat::core::format refused at macro expand time — not on the pure-combinator allow-list`

The whole stdlib failed to load (deftest 0/263). Reverted; nothing shipped.

## The principle the failure names

**Whether a callable may appear in an expand-time (macro-eval) position is a property declared at its
DEFINITION** — `format` is a macro (never expand-time-legal as a value); `string::concat` is pure-total
(expand-time-legal); a runtime `defn` is neither. Today that property is scattered and implicit: a
hand-maintained Rust `is_pure_total` allow-list (`src/macros/eval.rs`) for intrinsics, plus the
macro/defn/defclause distinction for user forms. **There is no single queryable fact a TOOL can ask:
"is `:wat::core::X` usable at expand time? at runtime? is it a macro?"**

A codemod that introduces a call (the concat→format fix; any future fix) MUST be able to query the
target POSITION's purity context AND the introduced callable's purity class, to know the rewrite is
legal. The lint/RETE engine (arc 277/278) is the first hard consumer; it cannot stay correct on a
guessed/implicit purity model.

## What this arc should carry (the ask)

- **Definition-time purity/expand-time-legality as a first-class, queryable metadatum** on every
  intrinsic + user callable in the builtin registry — `(metadata-of :wat::core::X)` answers
  `{:kind macro|fn|intrinsic, :pure-total bool, :expand-time-legal bool, …}`.
- The Rust `is_pure_total` allow-list becomes a *projection* of this registry, not a parallel
  hand-list (extirpare: one source of truth; the drift between the allow-list and reality is the
  failure class).
- Consumers (the RETE engine, codemod fixes) QUERY it rather than re-deriving — see arc 278's
  output-contract + the concat→format fix's macro-context gate (arc 277), both of which need this.

(Companion requirement, captured in arc 277 + 278: the detection rule also needs to know whether the
form under inspection is ITSELF in an expand-time position — "am I inside a defmacro body?" — which is
the context half; this note is the callable-class half. Both are needed to gate a macro-introducing fix.)

---

## UPDATE 2026-06-20 (arc 278 stone 6a) — purity is TWO orthogonal properties: pure AND deterministic

The rete capability tier (stone 6a — the `where`/`:test`/accumulator fence) became the **live consumer**
this note predicted, and it surfaced that the metadatum is not one bit but **two orthogonal ones**:

- **`pure`** — effect-free (no IO/mutation/spawn). Seed: the *negation* of `is_effectful_op`
  (`runtime.rs`) — `:wat::kernel::`/`:wat::io::`/`:wat::eval-`/`:wat::load`/`:wat::config::`.
- **`deterministic`** — referentially transparent, same inputs → same output (no randomness/clock/entropy).

They are genuinely independent. **`:wat::core::Uuid/v4` is the proof: it does no IO and mutates nothing
→ PURE — yet it is random → NON-deterministic.** (`Uuid/v5` = SHA1(ns,name) is pure ∧ deterministic; a
hypothetical `clock/now` would be pure ∧ non-deterministic; `io::read` is impure ∧ non-deterministic.) A
rete *condition* must be a deterministic, effect-free function of the facts → it requires **both**; the
exposed check is `(and (pure? f) (deterministic? f))`. Collapsing them into one "pure" bit (a first 6a
draft did, by jamming `Uuid/v4` into a "non-deterministic" set *inside* the purity check) is the muddle
this update corrects.

So `(metadata-of :wat::core::X)` must carry **`{:kind, :pure, :deterministic, :expand-time-legal, …}`** —
`:deterministic` is the sibling property the original note didn't name.

**Status / what 278 ships (NOT 255):** 255 (re-lift ~454 builtins into a registry) is **NOT ready** —
builder's call, 2026-06-20, too big to detour into mid-278. To ship rete, stone 6a carries a small
**hand-managed metadata map** in `src/rete/purity.rs` (`{pure, deterministic}` per op, default-deny,
transitive over user-fn bodies) exposing `:wat::rete::pure?` + `:wat::rete::deterministic?`. **This hand
list IS the "parallel hand-list" this note warns against — accepted as the explicit v1 projection.** When
255 lands, the map becomes a *projection* of the registry (delete the hand list; the rete predicates query
`metadata-of`), exactly as prescribed above. The hand list points here in-code for discoverability.
