# DESIGN — STONE A-2-ii-b-0: the three accessor-path verbs get homes and rulings

> Prerequisite for A-2-ii-b (`sort$native` imposes Pure ∧ Deterministic and homes). Measured
> 2026-08-30 in A-2-ii-b's pre-flight.

## Why — the exact, measured blocker

A generated record accessor's body is:

```clojure
(Record/field-at (Option/expect (if (= (type self) "R") (Some self) None) "…") 0)
```

Of the eight verbs in it, **exactly three are `KNOWN_UNREVIEWED` and unregistered** — measured, one
row each in `src/rete/purity.rs`:

```
:wat::core::Record/field-at      :wat::core::Option/expect      :wat::core::type
```

`=`, `Some`, `None`, `if`, `:wat::string::concat` all already classify. Because the three
default-deny on **every** axis, the accessor classifies impure through a binding — which is why
`wat/query/mem.wat:136,163` (`sort-by :wat::query::Row/sk`) cannot satisfy an imposed `Pure`.

★ **Dropping the `Total` demand is what makes this reachable at all.** All three are Pure ∧
Deterministic; two are `Partial`. Had we insisted on `Total`
(`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`), those two `Partial`s would
block the accessor **until the `expect` purge lands** — a campaign, not a stone.

## THE ONE CONTRACT DECISION — pinned

**Each verb is ruled from its implementation, and `Partial` is declared where a raise exists — the
`Partial` rows are the deliverable, not an embarrassment.** They are the first real entries on the
totality endgame's census (`Totality`'s own defenum: *"★ THIS VARIANT IS THE WORK LIST"*), which
today holds exactly one verb.

## The rulings — two pinned from measurement, one the rider must measure

| verb | @Purity | @Determinism | @Total | ground |
|---|---|---|---|---|
| `:wat::core::Option/expect` | Pure | Deterministic | **Partial** | raises on `None`. Ruled 2026-08-30: a raise is not a matchable outcome |
| `:wat::core::Record/field-at` | Pure | Deterministic | **Partial** | **measured at the site**: `if index < 0 \|\| (index as usize) >= fields.len()` returns `Err` |
| `:wat::core::type` | Pure | Deterministic | ⚠ **MEASURE IT** | `eval_type` has **4** `return Err` paths. Arity failures retire on homing (the macro generates the shim). If any remaining path is a DOMAIN failure — some value it cannot name a type for — it is `Partial`. Do not assume `Total` |

⚠ **`@ExpandTime`:** all three are Pure ∧ Deterministic and safe to evaluate during expansion. A
`Partial` verb can still be expand-time-legal — `macros/eval.rs` says so for `i64::/`: *"dividing by
zero during expansion raises a deterministic, located MacroError — a compile-time failure instead of
a runtime one, which is strictly better. Totality and expand-time legality are different axes."*

## What ships

Three `#[wat_intrinsic]` delegates, each thin, each delegating to the **existing** named fn — this is
the two-layer architecture: `src/intrinsic/<ns>.rs` registers and delegates, `src/<mod>/<file>.rs`
implements. All three are already thin arms over named fns (`eval_option_expect`,
`eval_record_field_at`, `eval_type`), so no body moves.

Then, forced by the ratchet: **delete the three `KNOWN_UNREVIEWED` rows.** That list is a
two-directional ratchet — *"a verb in this list that is no longer unreviewed ⇒ RED. Rule on it,
delete its line."* If the rows are not deleted, the floor goes red, and that red is the gate working.

## Out of scope = REJECTED (not deferred)

- **`sort$native`'s imposition and homing** — A-2-ii-b, the next stone.
- **The `expect` purge** — the builder's long-term direction (*"the expect calls… a fatal mistake…
  rip them out as we grind forwards"*). A campaign. This stone makes two of its targets *visible* by
  declaring them `Partial`; it removes none.
- **The other `KNOWN_UNREVIEWED` verbs** — only the three in the accessor path.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **b-0** home + rule the three, then impose separately | YES | YES | YES | YES | ✅ **ADMITTED** |
| home the three **and** impose in one stone | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| declare the three `Total` to avoid `Partial` rows | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| special-case accessors in the classifier | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **one-stone Simple? NO** — three homings plus a behavioural gate; a red could not be attributed.
- **declare-Total Honest? NO** — directly contradicts the ruling and would silently corrupt the
  totality census, which is the `expect` purge's worklist.
- **special-case Obvious? NO** (a reader meets accessors exempted for no stated reason);
  **Honest? NO** — it hides three unruled verbs behind an exemption instead of ruling them.

## Acceptance

| what | command | expected |
|---|---|---|
| the accessor classifies pure through a binding | probe: `(let [k :R/sk] (pure? '(fn [a] (k a))))` | `true` (is `false` today) |
| the three are registered | `lookup_entry` for each | `Some` |
| the ratchet is satisfied | the three `KNOWN_UNREVIEWED` rows | deleted |
| `Partial` census grew honestly | `grep -c "@Total *Partial"` in `src/` | 1 → 3 |
| no widening | probe: effectful fn through a binding | `false` |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
