# BRIEF — `constructor_meta`'s columns, audited (the surface form's turn)

Closes the inconsistency `b98cf189` named and did not touch. **Must land before `total?` is armed
(#57's third conjunct)** — arming over it makes the fence contradict itself.

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it — if the harness moves a long one to the background, that run is lost to
you. Your turn ends when the numbers are in your hands.

## The inconsistency

`b98cf189` classified the **expanded** constructor forms (`aggregate-new` / `kwargs-construct`) as
`pure ∧ deterministic ∧ total`, grounded, after closing two checker gaps to earn `total`.

`constructor_meta` (`src/rete/purity.rs:612-637`) still rules the **surface** form
`total: false` at both its return sites, and its own comment admits why:

> *"stays unmeasured rather than inferred … Left `false` on discipline, not because a counter-example
> was found."*

Default-deny, never audited. So the moment `total?` arms:

```clojure
:then [(:usr::Rate :count ?c :window ?w)]   ;; SURFACE   → refused on Total
:then [(:usr::make-rate ?c ?w)]             ;; EXPANDED  → admitted
```

Backwards, and traceable to an unaudited default rather than a finding. **This is #52's job, one file
later:** get the columns honest in both directions, by reading the implementation.

## The two sites — audit them SEPARATELY

`constructor_meta` returns from two places and they are different constructs:

| site | what it rules | today |
|---|---|---|
| `purity.rs:~622` | **aggregate** constructors (record / struct / holon-record) | `pure: a.nature.is_pure(), deterministic: true, total: false` |
| `purity.rs:~632` | **enum-variant** constructors | `pure: e.purity.is_pure(), deterministic: true, total: false` |

Do not assume they answer alike. If they differ on an axis, classify them separately with separate
reasoning — averaging is how a real asymmetry disappears.

## ★ AUDIT BOTH COLUMNS, NOT JUST `total`

`#52`'s discipline was *both directions* — remove falses-marked-true AND trues-marked-false. Apply it
here to **`pure` as well as `total`**.

**On `total`** — `b98cf189` established that construction is total once (a) `infer_aggregate_new_check`
validates arity and (b) the holon capacity budget is checked at freeze. **Both landed.** So the
question for the surface form is whether it reaches those same guarantees, or whether it has a path
the expanded form does not. Read it; do not infer it from the expanded verdict.

**On `pure`** — `a.nature.is_pure()` makes purity depend on the *nature of the thing being built*. But
the ACT of construction is assignment: it takes values that already exist and binds them into a shape.
A `Nature::Struct` may HOLD a resource; constructing one does not acquire it, and acquisition is
denied at its own head by the same `classify_expr` walk. That is the reasoning `b98cf189` used for the
expanded forms.

**⚠ Do NOT take that from me — it is exactly what you must ground.** Note the asymmetry the last
strike surfaced: `validate_aggregate_containment` rejects an impure field on a
`Nature::Record`/`HolonRecord` at startup, but a `Nature::Struct` may legitimately declare one. So the
question is precise: **inside a fence-walked expression, is there ANY route by which a resource-valued
binding reaches a struct constructor's argument?** If there is, `nature.is_pure()` is load-bearing and
must stay. If there is not, it is over-conservative and the act is pure.

## STOP triggers

1. **STOP-1 — either site is NOT total on a reachable path.** Report it with a `file:line` and leave
   that site `false` **with the grounded reason replacing the default-deny comment.** A measured
   `false` is a success here; the deliverable is an honest column, not a `true`.
2. **STOP-2 — you find a route by which a resource reaches a constructor argument inside a fence.**
   Then `pure` stays nature-dependent. Report the route — it also bears on `b98cf189`'s reasoning for
   the expanded forms, which would then be too permissive.
3. **STOP-3 — the two sites differ.** Classify separately; do not average.
4. **STOP-4 — scope.** Do NOT arm `total?`. Do NOT touch the expanded forms' classification. Do NOT
   mint rete vocabulary.

## Prove it

Whatever you conclude, a gate must show it. If a column flips to `true`, a probe must exercise the
newly-admitted form. If a column stays `false`, a probe must show the path that keeps it false —
otherwise the next reader cannot tell a measured verdict from the default-deny you replaced.

**Whatever the verdict, replace the "left false on discipline" comment.** That sentence is the debt;
it must not survive an audit either way.

## Gates — foreground, report each result line

```
cargo build --release --all-targets            # exit 0, ZERO warnings
cargo clippy --release --all-targets           # likewise
cargo test --release --test rete
cargo test --release --test lint
cargo test --release --lib -p wat              # the ratchet gate lives here
./wat-scripts/perf/grid/check-where-shapes.sh  # 9 pairs, 98 rows agreeing
```

**Do NOT run `cargo nextest run`** — the orchestrator weighs the floor centrally.

## Do not

Do not commit, push, stash, or revert anything you did not write. Do not add `#[allow(dead_code)]` or a
`rune:lint`. Do not flip a column to `true` to make a form work — the audit decides, not the desired
outcome.
