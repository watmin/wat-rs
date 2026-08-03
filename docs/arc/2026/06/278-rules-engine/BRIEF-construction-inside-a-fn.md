# BRIEF — a fn may construct the fact it returns

Unblocks Stone B's headline (`ac90d262` shipped the mechanism and says plainly that this does not
work yet). Spec: `DESIGN-STONE-then-is-a-vector-of-singular-facts.md` § "Stone B".

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it — if the harness moves a long one to the background, that run is lost to
you. Your turn ends when the numbers are in your hands.

## The defect, in one sentence

**The fence gives two different answers to the same act depending on which side of macro-expansion it
sees it from.**

```clojure
;; ADMITTED — constructor_meta rules it pure ∧ det ∧ total BY DECLARATION (purity.rs:~509):
;; "a field declared on an aggregate exists on every instance by construction"
:then [(:usr::Rate :count ?c :window ?w)]

;; REFUSED — identical construction, but inside a fn body it has already expanded to
;; :wat::core::kwargs-construct, which head_ok default-denies
(:wat::core::defn :usr::make-rate [c <- …  w <- …] -> :usr::Rate
  (:usr::Rate :count c :window w))
:then [(:usr::make-rate ?c ?w)]
```

`:wat::core::aggregate-new` (`purity.rs:1210`) and `:wat::core::kwargs-construct` (`:1226`) sit in the
**`KNOWN_UNREVIEWED`** ratchet — unclassified in `intrinsic_meta`, therefore default-denied.

## ⛔ CLASSIFY. DO NOT WHITELIST.

`KNOWN_UNREVIEWED`'s own doc comment is the instruction:

> *"Never add a line to make a red gate green — that is the laundering this gate exists to prevent;
> CLASSIFY the verb in `intrinsic_meta`, or give its namespace a disposition in `RULES` with the
> reason."*

So: classify the two verbs in `intrinsic_meta`, and **delete their lines from `KNOWN_UNREVIEWED`.**
The ratchet SHRINKS by two. It must never grow here.

## ★ GROUND THE THREE AXES BY READING THE IMPLEMENTATIONS — do not reason from the surface

This is the whole job and the rest is mechanical. **Read `eval_aggregate_new` and
`eval_kwargs_construct` and answer each axis from what the code does**, exactly as S1+S2 (`#52`) did
for the `total` column. An argument from "a constructor is obviously pure" is not grounding; that same
argument was available for the four holon verbs and two of them turned out partial.

| axis | the question to answer from the code |
|---|---|
| **pure** | does it perform IO, mutate, or touch ambient state? |
| **deterministic** | same args ⇒ same value, always? |
| **total** | ★ **THE ONE THAT CAN GO EITHER WAY.** Can it RAISE on any input that reaches it? Arity mismatch, a field name that is not on the type, a type mismatch, a duplicate kwarg, an unknown aggregate. If the checker provably rejects all of those before runtime, it is total. **If ANY reachable path raises, it is NOT total — say so and stop** (STOP-1). |

Note the asymmetry to watch: `constructor_meta` reaches its verdict from the **declaration**, which is
why the surface form is total. The expanded form takes its arguments as a flat list at runtime. Those
are not automatically the same situation — prove the equivalence, do not assume it.

## STOP triggers — rejection criteria

1. **STOP-1 — either verb is NOT total on a reachable path.** Report the path with a `file:line` and
   ship nothing. Do NOT mark it total to make the headline work; that is the laundering above, and a
   partial op inside a `where`/`:then` is the live hazard this whole fence exists for.
2. **STOP-2 — the two verbs differ on an axis.** Classify them separately with separate reasoning;
   do not average them.
3. **STOP-3 — the ratchet's own gate goes red.** It asserts the list only shrinks. If removing two
   lines breaks it, read why before touching the assertion.
4. **STOP-4 — scope.** Do NOT arm law A, do NOT mint rete vocabulary, do NOT widen anything beyond
   these two classifications.

## Prove it both directions

- **GREEN** — the headline: a `defn` that CONSTRUCTS and returns a record, called from a `:then`,
  compiles and derives the fact. Drive it through **both** the oracle (`fire-rules-spec`) and the
  native kernel (`fire-rules`), as `probe_arc278_then_user_forms` does.
- **RED, and it must still refuse** — a `defn` that constructs a record *and* touches an impure op is
  still refused, with the impure head named. Classification must not have opened a hole: the walk into
  the fn body is what keeps this honest, and it must still bite.

## Gates — foreground, report every result line

```
cargo build --release --all-targets            # exit 0, ZERO warnings
cargo clippy --release --all-targets           # likewise
cargo test --release --test rete
cargo test --release --test lint
cargo test --release --lib -p wat              # the ratchet gate lives here
./wat-scripts/perf/grid/check-where-shapes.sh  # 9 pairs, 98 rows agreeing
```

**Do NOT run `cargo nextest run`** — the orchestrator weighs the whole floor centrally.

## Do not

Do not commit, push, stash, or revert anything you did not write. Do not add `#[allow(dead_code)]` or
a `rune:lint`. Do not add a line to `KNOWN_UNREVIEWED` under any circumstance.
