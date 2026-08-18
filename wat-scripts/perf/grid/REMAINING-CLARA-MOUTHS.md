# Remaining Clara mouths

Locked here so a compaction cannot drop them. A mouth leaves this list
only when it has a `where-*` twin, Clara agrees, and `check-spec-native.sh`
is green on that stem.

Not on this list: identity-bag Hits, dup-insert / query-input-type
(value-session purity), process rete IPC (Claude), compiled-vs-interpreted.

## 1. Unbound grouping in a leading `:from` — DONE (`where-accum-group`)

Clara `test-count-none-joined` (accum first) and a lone

```text
[?c <- (acc/count) :from [Temp (= ?loc loc)]]
```

Temps at MCI and ORD are two groups, not one global count. A later Wind
at `?loc` sees `{?c 0, ?loc MCI}` when MCI has wind and no temps.
Compile defers accumulators; accumulate-pass groups leftover `:from` binds.

## 2. `:not` of `:and` with a bound fact — DONE (`where-not-and-bound`)

Clara `test-complex-negation`:

```text
[:not [:and [?t <- Temp] [Cold (= temperature (:temperature ?t))]]]
```

Inner `:and` is a join on the Temp's temperature, not “both patterns exist.”
`binding-extensions` already backtracks; the twin locks the mismatch row
(Temp 10 + Cold 20 still fires). Wat joins on the shared field `?c`.

## 3. Nested `:not` inside that `:and` — DONE (`where-not-and-not`)

Same Clara test (and issue 304):

```text
[:not [:and [Temp ?loc] [:not [Cold]]]]
[Wind ?l] [:not [:and [Temp ?l ?c] [:not [Cold ?c]]]]
```

Inner `:not` is a join-filter on the Temp's temperature. Temp-without-matching-Cold
makes the `:and` true, so the outer `:not` drops. Matching Cold flips it.
`binding-extensions` already recurses through `:not`; the twin locks it.

## 4. Parametric query mouth — DONE (`where-query-params`)

Clara `[defquery q [:?loc] …]`. One `query` mouth:

```text
(:wat::rete::query session (:wqp::temps-at) :?loc "MCI")
```

`defquery` + `query`. MCI with wind and no temps → `{?n 0, ?loc MCI}`.

## 5. Fact-bind `(?t <- :ns::Type …)` — DONE (`where-fact-bind`)

Clara `[?t <- Temp]`. Form B: `<-` binds; the type keyword has `::`.
A field-only `(:Type …)` does not put the record on a binding.
`:where` / `:then` may use `(:Type/field ?t)` once asked for.

## Query compat grid — `check-query-compat.sh`

Clara | wat-oracle | wat-native on the query families
(`where-query-compat`, `where-query-params`, `where-fact-bind`).
`where-query-compat` prints binding maps (sorted scalars), not just `n=`.

## 6. Inline leftover on a HashJoin — DONE (`where-join-left`)

Clara `[Wind (= ?loc loc) (= ?w kph) (> ?w ?c)]` after a Temp that
bound `?c`. Beta rematch of the right cond under the left token
(`alpha-match-under` / `join_extend`). Same as exists/not.

`check-where-shapes.sh where-join-left` 9/9 == Clara.
`check-spec-native.sh where-join-left` 9/9 spec == native.

## 7. Leftover on accumulate `:from` — DONE (`where-accum-from-left`)

Clara `[?n <- (acc/count) :from [Wind (= ?loc loc) (> kph ?c)]]`
after a Temp that bound `?c`. Beta filter on the `:from` bag.
Empty `:from` still fires with count 0. Field form (no extra `?w`
bind) so the count is not grouped.

`check-where-shapes.sh where-accum-from-left` 7/7 == Clara.
`check-spec-native.sh where-accum-from-left` 7/7 spec == native.

## This list is empty.

2026-08-17: items 1–7 locked.
Breadcrumb: `docs/arc/2026/06/278-rules-engine/CURRENT-STATE-annihilate-interpretation.md`.
