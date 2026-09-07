# REVIEW — the measurement HOLDS and is mutation-proved. The differential is HALF-BUILT.

## What I re-ran myself, and what held

| claim | my independent result |
|---|---|
| native ANCHOR `l23::Ok2` ⇒ 1 | **HOLD** — reproduced |
| native MEASURE `l23::Tally` ⇒ 1 | **HOLD** — reproduced |
| oracle ANCHOR `{"l23::Ok2" 1}`, MEASURE `{}` | **HOLD** — I drove the scratch `.wat` myself on a verified-HEAD binary |
| keys agree, no leading colon | **HOLD** — both spell `l23::Tally` / `l23::Ok2` |
| first floor red captured, not re-run blind | **HOLD** — `.floor/2026-09-07T07-20-39Z` carries `5473 run, 1 failed`, arm named `rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves`, cause a citation spelling. ⭐ Correct discipline, and caught by a gate this very session's crawl had read. |
| final floor | **HOLD** — `5473 tests run: 5473 passed (2 slow), 22 skipped` |

**Mutation-proved by me, both arms, live source, restored after each** — one mutation cannot prove
a two-arm gate:

| mutation in `src/rete/kernel/stratify.rs` | arm that reddened |
|---|---|
| `+ i64::from(derived)` → `+ 0` (:224) | **MEASURE** — `native MEASURE = {}` |
| `+ 1` → `+ 0` on the negated fold (:211) | **ANCHOR** — *"empty means the native instrument is inert"* |

The finding itself is credited: **the strata differ, the facts do not.** DESIGN reading 1. The
prediction held and STOP-1 did not fire.

## ⛔ THE REFUTATION — the oracle half of this differential is not driven by anything

`stratify_numbers.rs:118-119`:

```rust
ORACLE ANCHOR keys (raw):  [\"l23::Ok2\" 1]\n\
ORACLE MEASURE keys (raw): []\n\
```

Those are **string literals inside the format string**, printed in the same shape and alignment as
the two lines above them, which are real measurements. Three facts, each checked:

1. **Zero assertions touch the oracle** — `grep -c` over the assertion bodies returns 0. Every
   assertion in the file reads `native_anchor` / `native_bag`.
2. **The scratch `.wat` is never RUN by a gate.** `every_wat_scripts_file_loads` collects and
   *loads* — parse plus type-check. `wat-rs/CLAUDE.md` is explicit that this is not execution.
3. So **nothing on the floor re-derives the oracle's numbers.** If the oracle's sweep changes
   tomorrow, this test stays green and prints two lines asserting an oracle state that no longer
   exists.

**This is the strike's own defect shape, one level down.** The strike exists because two headers
claim lockstep and nothing re-derives the claim. The test now claims the oracle's numbers and
nothing re-derives *that*. `[[half-built-is-worse-than-unbuilt]]` — it compiles, its output looks
like a four-line two-engine comparison, and half of it is a constant.

## The work

Drive the oracle **in the same test**, in the world it already has. `world()` is a `FrozenWorld`
and `rules()` already evaluates wat forms in it via `eval_in_frozen`; `:wat::rete::stratify` is an
ordinary wat verb, so the same call shape reaches it:

```rust
// same frozen world, same rule-set expressions already built above
let oracle_anchor = /* eval_in_frozen of
    (:wat::rete::stratify (:wat::core::PersistentVector (:l23::ok) (:l23::neg))) */;
let oracle_bag    = /* ... (:l23::ok) (:l23::tally) ... */;
```

Then make both arms cross-engine, so each has a real second side:

- **AGREEMENT arm** — the engines must agree on the negation: `oracle_anchor` and `native_anchor`
  both raise `l23::Ok2` to 1. This is the term they share, and it is what proves the two
  instruments are comparable at all.
- **DIVERGENCE arm** — `native_bag["l23::Tally"] == 1` while `oracle_bag` has no `l23::Tally`.
  Both sides measured, in one process, on one binary.

Delete the hardcoded `ORACLE …` lines rather than leaving them beside derived ones — a printed
constant formatted like a measurement is how the next reader is misled, and this file's whole
subject is a claim that nothing re-derives.

## STOP triggers

1. **If `:wat::rete::stratify` cannot be reached through `eval_in_frozen` from this test** — STOP
   and report the exact error. Do **not** restore hardcoded oracle values as a fallback; that is
   the state being refuted. If it genuinely cannot be driven in-process, the honest artifact is a
   `rune:excusare(no-falsifier)` on the oracle half naming why, and I will weigh that.
2. **If the two engines turn out to AGREE on the bag set when the oracle is actually driven** —
   STOP and report. That would mean the scratch `.wat` and the in-test call disagree, which is a
   finding about the instrument, not about the engines, and it outranks everything else here.
3. Still no `+1` added or removed on either side; still do not touch
   `probe_arc278_derived_exists_acc`.

## Not in scope

The header fix. `stratify.rs:205`'s *"Mirrors `stratify-sweep`"* and the oracle's *"lockstep with
native `rule_consumes`"* are both false and both stay untouched until this gate is whole — the gate
is what will keep the corrected wording honest, so it lands first.
