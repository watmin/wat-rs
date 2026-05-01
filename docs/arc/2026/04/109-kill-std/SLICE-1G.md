# Arc 109 Slice 1g — list retires; tuple → Tuple (verb-equals-type)

**Status: shipped 2026-05-01.** Walker (commit `1dea484`) +
three-tier sweep across commits `83abf44` → `bad3c5e` →
`e59a077`. (Tier 4 was a no-op — `examples/` had zero hits.)
22 files swept; **74 tuple sites + 12 list sites = 86 rename
sites**. Zero MANUAL flags, **zero substrate-gap fixes**. cargo
test --release --workspace 1476/0.

Higher than the brief's ~65 estimate — `wat/std/stream.wat`
alone had 21 tuple sites (tier 1 totalled 47 just there +
others).

**Originally drafted as a compaction-amnesia anchor mid-slice;
preserved here as the durable record. Smallest substrate-gap
work so far in arc 109's slices because the Pattern 2 mechanism
was already shipped + rehearsed in slice 1f. Adding two more
poison arms + two more hint helpers + two new dispatch arms is
purely additive — no new walker logic, no new CheckError variant,
no canonicalization extension.**

## What this slice does

Two coupled retirements per INVENTORY § D, both Pattern 2 (verb
retirement via synthetic TypeMismatch poison):

1. **`:wat::core::list` retires.** It was always a duplicate of
   `:wat::core::vec` (now `:wat::core::Vector` post-slice-1f);
   both produced `Vec<T>`. The redundancy goes; consumers
   migrate to `:wat::core::Vector`.

2. **`:wat::core::tuple` → `:wat::core::Tuple`.** Verb-equals-type
   per slice-1f's playbook (`vec` → `Vector` shipped first; this
   is the analogous tuple sibling). `(:wat::core::Tuple x y z)`
   reads as "construct a Tuple of these elements." The type
   spelling `:(T,U,V)` is parsed specially (tuple-literal type),
   so the verb rename is independent of the type spelling — no
   parametric-head walker entry needed.

## What this slice does NOT do

- **`:wat::core::range` move to `:wat::list::range`** — § H
  territory. Namespace move (path changes; name stays). Different
  shape from these verb retirements; needs the
  `:wat::list::*` namespace to exist as a substrate concept.
  Different slice.
- **Variant constructor FQDN** (`Some` / `:None` / `Ok` / `Err` →
  `:wat::core::*`) — § C territory. Different mechanism (not a
  Pattern-2 verb retirement; the constructors live at the AST-
  parse layer for now).

## The protocol

Pattern 2 (verb retirement) for both, identical mechanism to
slice 1f's `vec` poison. No new walker; no new variant; just two
new synthetic-TypeMismatch arms + two migration-hint helpers.

This is the **simplest slice of arc 109's remaining substrate
work** — all the discovery and template-building happened in
slices 1c/1d/1e/1f. Slice 1g is just two more applications of
the proven Pattern 2 mechanism.

## Scope

- `:wat::core::list` references in user code: **6** sites.
- `:wat::core::tuple` references in user code: **59** sites.

Total ~65 rename sites. Smallest sweep so far. Substrate stdlib
boots clean once dispatchers are wired; user-code sweep is a
quick four-tier pass.

## What to ship

### Substrate (Rust)

1. **Add `:wat::core::Tuple` dispatch arm** in `src/check.rs` and
   `src/runtime.rs` next to `:wat::core::tuple`. Same
   `infer_tuple_constructor` / `eval_tuple_ctor` body.

2. **Poison `:wat::core::list`** in `src/check.rs::infer_list`
   dispatcher. Push synthetic
   `CheckError::TypeMismatch { callee: ":wat::core::list",
   expected: ":wat::core::Vector", got: ":wat::core::list", ... }`
   then continue dispatching (consumers sweep call-by-call, no
   cliff). Same shape as slice 1f's `:wat::core::vec` poison.

3. **Poison `:wat::core::tuple`** with a sibling synthetic
   TypeMismatch redirecting to `:wat::core::Tuple`.

4. **Add migration hint helpers** in `src/check.rs::collect_hints`:
   - `arc_109_list_verb_migration_hint` — fires when callee is
     `:wat::core::list`; redirect to `:wat::core::Vector`. Note:
     `list` retires entirely (it was a duplicate); `Vector`
     replaces both.
   - `arc_109_tuple_verb_migration_hint` — fires when callee is
     `:wat::core::tuple`; redirect to `:wat::core::Tuple`.

### Verification

Probe coverage:
- `(:wat::core::list :T 1 2 3)` → fires (Pattern 2 list poison)
- `(:wat::core::Vector :T 1 2 3)` → silent (canonical, post-1f)
- `(:wat::core::tuple x y z)` → fires (Pattern 2 tuple poison)
- `(:wat::core::Tuple x y z)` → silent (new canonical)
- `:my::pkg::list` and `:my::pkg::tuple` (user paths) → silent
  (callee head doesn't match)

## Sweep order

Same four tiers as slices 1c/1d/1e/1f. Substrate stdlib first
(probably ~few sites in stdlib).

1. **Substrate stdlib** — `wat/`, `crates/*/wat/`.
2. **Lib + early integration tests** — embedded wat strings.
3. **`wat-tests/`** + **`crates/*/wat-tests/`**.
4. **`examples/`** + **`crates/*/examples/`**.

Verification gate after each tier: cargo test --release
--workspace shows zero TypeMismatch on `:wat::core::list` AND
zero on `:wat::core::tuple` before next tier.

## What does NOT change

- **Internal Rust string literals** referencing `:wat::core::tuple`
  / `:wat::core::list` in dispatcher arms — these are the
  canonical-form internal representations. The substrate's
  special-case dispatch reads against `tuple` head; that stays.
- **The walker logic** — already shipped slice 1c/1d/1e/1f.
  Slice 1g doesn't extend the walker at all (it's pure Pattern 2
  verb work; no TypeExpr-shape detection involved).
- **Variant constructors** — slice § C territory.
- **`:wat::core::range`** — § H territory (namespace move).

## Closure (slice 1g step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § D — strike `list` row (mark "retired —
   use Vector"); mark `tuple` ✓ shipped slice 1g.
2. Update `J-PIPELINE.md` — slice 1g done.
3. Update `SLICE-1G.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row.

## Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 2 verb retirement
  mechanism.
- `docs/arc/2026/04/109-kill-std/SLICE-1F.md` — the precedent
  slice (`vec` poison + `Vector` dispatch); 1g is two more
  applications of the same shape.
- `docs/arc/2026/04/114-spawn-as-thread/INSCRIPTION.md` — original
  Pattern 2 application (spawn poison; arc 114).
- `src/check.rs::collect_hints` — where the two new hint helpers
  land.
