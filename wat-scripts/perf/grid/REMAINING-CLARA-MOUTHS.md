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

## This list is empty. Next endeavor is not a mouth.

2026-08-17: all five items above are locked. Do not invent a sixth mouth.
The next work is **annihilate interpretation** — compile every rete expr.
Breadcrumb: `docs/arc/2026/06/278-rules-engine/CURRENT-STATE-annihilate-interpretation.md`.
