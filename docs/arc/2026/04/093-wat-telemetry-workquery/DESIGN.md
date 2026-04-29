# Arc 093 — `:wat::telemetry::*` reader-side / interrogation — DESIGN

**Status:** READY — opened 2026-04-29; all 11 open questions
settled by Q&A 2026-04-29. Ready to ship slice 1.

The decisions that landed:
- **§1** Form-matcher: sibling arc, Clara-style `matches?` macro.
- **§2** `?var` syntax (no underscore convention).
- **§3** No pagination — prepared-statement step-through; bounded(1) channel for backpressure.
- **§4** Read-handle Drop closes connection.
- **§5** Single .db per script — pry/gdb model; cross-db rejected.
- **§6** Seven indexes provided by substrate; "set once, never touch."
- **§7** No `run-with` — use `:wat::std::stream::filter`.
- **§8** No metric↔log uuid join sugar — pivot is the analysis program's job.
- **§9** Always ASC by `time_ns` / `start_time_ns`.
- **§10** Indexes ride in slice 1 alongside auto-spawn schema derivation.
- **§11** Clara-flavored constraint vec for queries (Option C).

**Predecessor:** [arc 091 — `:wat::telemetry::*` writer side](../091-wat-measure/INSCRIPTION.md).
Arc 091 shipped `Event::Metric` + `Event::Log`, sqlite auto-spawn schema
derivation, the `wat-telemetry-sqlite` writer crate, and slice 8's
`struct->form` + runtime quasiquote that close the data round-trip.

**Named in arc 091's INSCRIPTION:**

> Arc 093 — `:wat::telemetry::WorkQuery`. Reader side. Time-indexed
> queries; prolog-y unify; combinators; bidirectional join. Builds on
> the writer this arc shipped.

**Possible sibling:** form-matching primitive may split into its own
substrate arc (TBD — see open questions §1).

**Concurrent / downstream:** arc 094 — circuit.wat (deferred — the
lab-side N×M topology smoke test that will be the first production
consumer of arc 091's writer beyond pulse/smoke). Arc 094 will use
whatever 093 ships; 094's design isn't blocking but its eventual needs
may inform 093's slice 4-5 examples.

---

## The use case

Post-hoc interrogation of run dbs by us, debugging.
**Wat-as-scripting-language.** Each run.db is a frozen
environment; one script opens one db; questions get answered;
script exits. The user's framing:

> we'll do some run and then we'll interrogate the system - we'll
> build up script templates and treat wat like a ruby of sorts -
> we'll build queries and run them. sql to reduce the candidate
> space and edn into a filter func to do pruning

> the sqlite is our ruby pry... our gdb into the system

The pry/gdb metaphor is exact: deep dive into one frozen artifact,
poke around, ask questions, learn what happened. Each script is
one debug session. Not a continuous monitoring tool. Not a
dashboard. The interrogation analog of attaching a debugger to a
core dump.

The pipeline is **`SQL stream → filter → consumer`** — Ruby's
`Enumerator` shape applied to telemetry data. SQL narrows via cheap-
to-index predicates (time range, uuid, namespace, metric_name) via a
prepared statement that streams row-by-row to the producer thread;
wat-side predicate prunes via expensive structural matching (the
`data` column's EDN content). See §3 for streaming details.

**Not** in scope:
- Real-time streaming consumers (this is post-hoc).
- Cross-machine query federation.
- SQLite EDN UDF (deferred per arc 091; would let WHERE clauses
  pattern-match EDN in SQL directly — substantial substrate add, out of
  scope until scale demands).
- Auto-magic caller detection.

## Schema is locked; richness lives inside the WatAST columns

A clarification from the user mid-design:

> the schema is extremely unlikely to change at this point - all of the
> fields are WatAST now - we can make the columnar data as rich as we
> want

The arc 091 schema is FROZEN at the column level:
- `metric` (start_time_ns, end_time_ns, namespace, uuid, tags,
  metric_name, metric_value, unit)
- `log` (time_ns, namespace, caller, level, uuid, tags, data)

What VARIES is the content of `tags` (HashMap<HolonAST,HolonAST> as
NoTag EDN) and `data` (Tagged WatAST as EDN). The consumer's struct
defining the `data` payload — `PaperResolved`, `TickSnapshot`,
`MarketState`, anything — can be arbitrarily rich. The substrate
preserves it round-trip via arc 091 slice 8's `struct->form` /
`eval-ast!` path.

This is load-bearing for the form-matcher. The matcher operates against
**rich AST/struct values stored in fixed columns**. SQL push-down
narrows candidate rows by indexed column predicates; the form matcher
prunes by walking the rich payload against constraints. The two layers
compose cleanly because they operate on different axes — *SQL filters
the row; the matcher filters inside the row.*

**Implication:** the form matcher's primary job is constraint
satisfaction over WatAST/HolonAST values reified from EDN. It is NOT a
schema migration tool, NOT a data validator at write time, NOT an ETL
layer. It's the read-side question-asker.

## The Ruby specification (verbatim)

The user's reference shape:

```ruby
all_data = Enumerator.new do |yielder|
  next_token = nil

  loop do
    resp = db.query(:some_query, next_token)
    resp.items.each { |i| yielder << i }
    next_token = resp.next_token
    break if next_token.nil?
  end
end

filtered_data = Enumerator.new do |yielder|
  all_data.each do |item|
    next if skip_item?(item)
    yielder << item
  end
end

filtered_data.each do |item|
  # do whatever with item here
end
```

Three stages, each lazy, bounded memory regardless of total result size:

1. **Streaming source** — DB-side iterator yielding rows; consumer
   pulls one item at a time.
2. **Filter** — pure predicate; lazy; pulls from upstream as the
   consumer demands.
3. **Consumer terminal** — `each-do`/`for-each`; drives the whole
   pipeline.

## The wat shape

The substrate already has the Enumerator equivalent:
`:wat::std::stream::Stream<T> = :(Receiver<T>, ProgramHandle<()>)` from
arcs 004 / 006 / 022. The combinators (`filter`, `map`, `for-each`,
`with-state`, `take`, `chunks`, `flat-map`, `inspect`) already ship. We
ride on top.

| Ruby | wat |
|---|---|
| `db.query(:q, token)` | prepared statement + `sqlite3_step` row-by-row in producer thread |
| `yielder << i` | `send-blocking` through bounded(1) channel |
| `all_data.each { yielder ... unless skip_item? }` | `:wat::std::stream::filter` |
| `filtered_data.each { ... }` | `:wat::std::stream::for-each` |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ wat script (e.g. examples/interrogate-pulse.wat)            │
│                                                             │
│   (let* ((db (sqlite/open "runs/pulse-X.db"))               │
│          (q (log-query                                      │
│               (vec :LogConstraint                           │
│                 (since     ts0)                             │
│                 (until     ts1)                             │
│                 (namespace :trading))))                     │
│          (events (sqlite/stream-logs db q))                 │
│          (filtered (filter events                           │
│                      (lambda (e) (matches? <template>       │
│                                            (data e))))))    │
│     (for-each filtered                                      │
│       (lambda (e) (println (render e)))))                   │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ wat-telemetry-sqlite reader (THIS ARC)                      │
│                                                             │
│   sqlite/open  →  ReadHandle (read-only sqlite connection)  │
│   sqlite/stream-logs handle query                           │
│     │                                                       │
│     ├─ spawn producer thread                                │
│     ├─ prepare SELECT ... WHERE <push-down> ORDER BY time_ns│
│     ├─ thread loop:                                         │
│     │    sqlite3_step → row                                 │
│     │      reify to Event::Log                              │
│     │      send-blocking (bounded(1) → Receiver<Event>)     │
│     │    until SQLITE_DONE                                  │
│     └─ return Stream<Event::Log>                            │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ sqlite (arc 091 schema, with new indexes from this arc)     │
│                                                             │
│   metric (start_time_ns, end_time_ns, namespace, uuid,      │
│           tags TEXT NoTag, metric_name, metric_value, unit) │
│   log    (time_ns, namespace, caller, level, uuid,          │
│           tags TEXT NoTag, data TEXT Tagged)                │
│                                                             │
│   indexes (NEW): time_ns, uuid, namespace, metric_name      │
└─────────────────────────────────────────────────────────────┘
```

## What's new in this arc

1. **Read-handle primitive.** `(sqlite/open path) ->
   :wat::telemetry::sqlite::ReadHandle`. Read-only sqlite connection
   (separate from the writer's Service-shaped pool). Drop closes the
   connection.

2. **Stream sources via prepared-statement step-through.**
   - `(sqlite/stream-metrics handle query) -> :wat::std::stream::Stream<:wat::telemetry::Event::Metric>`
   - `(sqlite/stream-logs handle query) -> :wat::std::stream::Stream<:wat::telemetry::Event::Log>`

   Internally: spawn a producer thread that prepares the SELECT
   statement and calls `sqlite3_step` (via rusqlite's iterator)
   row-by-row, reifies each row to its Event variant, sends through
   bounded(1) channel. Memory bounded regardless of total result
   count. WAL mode means long-running readers don't block writers.
   See §3 for the streaming-vs-pagination decision.

3. **Query types + builders.**
   `:wat::telemetry::MetricQuery` and `:wat::telemetry::LogQuery`
   structs with optional fields (time-range, uuid, namespace, level,
   metric-name). Functional builders (`(query/since q ts)`,
   `(query/until q ts)`, `(query/namespace q ns)`, etc.) accumulate
   WHERE clauses. The query value is data; `stream-*` consumes the
   data and assembles SQL inside the producer.

4. **Materialization helpers.** Two-level access to the `data` column:
   - `(event-data-ast e) -> :Option<:wat::holon::HolonAST>` — the raw
     Tagged AST (cheap; pattern-match in wat against shape).
   - `(event-data-value e) -> :Option<:Value>` — full lift via
     `:wat::eval-ast!` to a `Value::Struct` (round-trips through arc
     091 slice 8's `struct->form` / `eval-ast!` path).

   The user picks per-call: shape-match on AST for cheap structural
   filters; full eval to get a struct's fields.

5. **Sqlite indexes.** Auto-spawn (arc 085) gains indexes on `time_ns`
   (load-bearing), `uuid`, `namespace`, `metric_name`. Without these,
   "SQL pre-filters" is aspirational at scale.

## Sibling arc — Clara-style form-matching primitive

The user's preference is firm:

> i have a such strong bias for clara style - this is well understood?
> this is an amazing UX

**Clara Rules** (Clojure rule engine, Ryan Brush et al., 2014–) is the
reference implementation. RETE-based forward-chaining production rule
system. Field-keyed pattern syntax with `?var` bindings and constraint
interleaving — well understood, decade-plus in production, beloved
by the Clojure data community. Same shape we've been circling.

The matcher operates on `Value::Struct` (lifted from the `data`
column via `event-data-value` → `eval-ast!`). Pattern asserts type
and specific fields; silent about un-mentioned fields. Adding a
field to a struct doesn't break queries — you only break what you
mention.

### The shape

```
(:wat::form::matches? SUBJECT
  (TYPE-NAME
    (= ?var :field)        ; binding — extract :field into ?var
    (= ?var "literal")     ; constraint — equality check
    (> ?var 5.0)           ; constraint — predicate over bound var
    ...))
```

Inside the pattern, the macro recognizes special heads as part of
the matcher's grammar (no `:wat::core::` prefix needed):

- `=` `<` `>` `<=` `>=` `not=` — comparisons.
- `and` `or` `not` — logical combinators over constraints.
- `where` — escape hatch; arbitrary wat expression evaluated with
  bindings in scope.

Each clause classifies as either binding or constraint:

- **Binding**: `(= ?var :keyword)` where `?var` is fresh and the
  RHS is a `:keyword` matching a struct field name.
- **Constraint**: anything else.

Bindings land first (extracted from subject via field accessors);
constraints evaluate with bindings in lexical scope; result is
`:bool` (true iff type matches AND all constraints pass).

### Lineage

- **Clara Rules** (2014–) — Clojure rule engine. RETE-based.
  Field-keyed pattern syntax we adopt directly. The reference UX.
- **Datomic Datalog** — Hickey's read-side query language for
  immutable databases. Same field-keyed-with-constraints shape.
- **core.logic** — Clojure miniKanren port; relational programming
  as a library.
- **miniKanren** — Friedman / Byrd / Kiselyov, 2005.
- **Prolog** — Colmerauer / Kowalski, 1972.

We inherit Clara's pattern shape directly. The other ancestors inform
if/when the matcher extends toward full goal-language territory
(joins, recursion, backtracking).

### What ships in the sibling arc

A new arc (call it arc 097, or next free) ships:

- **`:wat::form::matches?`** defmacro — Clara-style pattern matching
  over `Value::Struct`. Walks the pattern at expansion; classifies
  clauses as bindings or constraints; emits `(and (struct-of?
  subject :Type) (let* (bindings...) (and constraints...)))`.
  Reusable beyond telemetry.
- **`:wat::core::struct-of?`** predicate — type-check on `Value`.
  (If not already shipped — likely trivial slice add.)
- **`?var` symbols as logic variables.** Convention: any symbol
  starting with `?` is a placeholder. The macro detects them.
  No reader changes needed (wat already accepts `?` in symbol
  bodies; this is a convention layered on top).

Arc 093 depends on it for slice 4's example scripts.

### What does NOT ship in the sibling arc

- Cross-fact joins (Clara's multi-pattern `[A ...] [B ...]`).
- Backtracking (multiple solutions per match).
- Full goal language (`fresh`, `or`-of-clauses, recursion).
- Forward-chaining rule firing (Clara's `defrule` + `=>` action).

These are bigger work. Arc 093's use case (one-fact-at-a-time filter
in a stream) doesn't need them. They arrive if a real consumer
demands them — likely never as one arc; more as a series of small
primitives that compose.

## Worked examples — what the UX feels like

Two queries we'd actually want to run, in the committed Clara-style
shape. The form-matcher operates on `Value::Struct` (lifted from
the Tagged WatAST `data` column via `event-data-value` →
`eval-ast!`).

```
(:wat::form::matches? SUBJECT
  (TYPE-NAME
    (= ?var :field)        ; binding
    (= ?var "literal")     ; constraint
    (> ?var 5.0)           ; constraint
    ...))
```

### Query 1 (warmup) — all Grace outcomes in this run

> "What papers Graced in this run, regardless of which thinker?"

```scheme
(:wat::core::let*
  (((db :wat::telemetry::sqlite::ReadHandle)
    (:wat::telemetry::sqlite/open "runs/proof-003-1714514400.db"))
   ((graces :wat::std::stream::Stream<wat::telemetry::Event::Log>)
    (:wat::std::stream::filter
      (:wat::telemetry::sqlite/stream-logs db
        (:wat::telemetry::log-query
          (:wat::core::vec :wat::telemetry::LogConstraint)))
      (:wat::core::lambda
        ((e :wat::telemetry::Event::Log) -> :bool)
        (:wat::form::matches?
          (:wat::telemetry::Event::Log/data-value e)
          (:trading::PaperResolved
            (= ?outcome :outcome)
            (= ?outcome "Grace")))))))
  (:wat::std::stream::for-each graces
    (:wat::core::lambda
      ((e :wat::telemetry::Event::Log) -> :())
      (:wat::io::IOWriter/println stdout
        (:wat::telemetry::Event::Log/render-debug e)))))
```

The matcher block is four lines. Type asserted by
`:trading::PaperResolved`; `?outcome` bound from the `:outcome`
field; constraint says it equals `"Grace"`. Silent about every other
field on the struct.

### Query 2 — the user's case: big-Grace winners, then their metrics

> "Find log rows from caller `:trading::test::proofs::003` in the
> last hour where `outcome = Grace AND grace-residue > 5.0`. For each,
> fetch the metrics that share its uuid."

```scheme
(:wat::core::let*
  (((db :wat::telemetry::sqlite::ReadHandle)
    (:wat::telemetry::sqlite/open "runs/proof-003-1714514400.db"))

   ;; Stage 1 — SQL push-down: caller + time window.
   ;; `hours-ago` ships in the sibling time arc — see Predecessors below.
   ((q :wat::telemetry::LogQuery)
    (:wat::telemetry::log-query
      (:wat::core::vec :wat::telemetry::LogConstraint
        (:wat::telemetry::since (:wat::time::hours-ago 1))
        (:wat::telemetry::caller :trading::test::proofs::003))))
   ((candidates :wat::std::stream::Stream<wat::telemetry::Event::Log>)
    (:wat::telemetry::sqlite/stream-logs db q))

   ;; Stage 2 — form-match + constraint prune (Clara-style).
   ((winners :wat::std::stream::Stream<wat::telemetry::Event::Log>)
    (:wat::std::stream::filter candidates
      (:wat::core::lambda
        ((e :wat::telemetry::Event::Log) -> :bool)
        (:wat::form::matches?
          (:wat::telemetry::Event::Log/data-value e)
          (:trading::PaperResolved
            (= ?outcome :outcome)
            (= ?grace-residue :grace-residue)
            (= ?outcome "Grace")
            (> ?grace-residue 5.0))))))

   ;; Stage 3 — collect winning uuids.
   ((winner-uuids :Vec<Bytes>)
    (:wat::std::stream::collect
      (:wat::std::stream::map winners
        (:wat::core::lambda
          ((e :wat::telemetry::Event::Log) -> :Bytes)
          (:wat::telemetry::Event::Log/uuid e))))))

  ;; Stage 4 — pivot: metrics for each winner uuid.
  (:wat::core::foldl winner-uuids ()
    (:wat::core::lambda
      ((_ :()) (u :Bytes) -> :())
      (:wat::std::stream::for-each
        (:wat::telemetry::sqlite/stream-metrics db
          (:wat::telemetry::metric-query
            (:wat::core::vec :wat::telemetry::MetricConstraint
              (:wat::telemetry::uuid u))))
        (:wat::core::lambda
          ((m :wat::telemetry::Event::Metric) -> :())
          (:wat::io::IOWriter/println stdout
            (:wat::core::string::concat
              "uuid=" (:wat::telemetry::Bytes/to-hex u)
              " metric=" (:wat::telemetry::Event::Metric/name m)
              " value=" (:wat::core::f64::to-string
                          (:wat::telemetry::Event::Metric/value m))))))))))
```

The matcher block:

```scheme
(:wat::form::matches? (:wat::telemetry::Event::Log/data-value e)
  (:trading::PaperResolved
    (= ?outcome :outcome)
    (= ?grace-residue :grace-residue)
    (= ?outcome "Grace")
    (> ?grace-residue 5.0)))
```

Five lines. Each clause says exactly what it asserts. No discards,
no positional ceremony.

### What these examples imply

1. **The matcher operates on `Value::Struct`, not raw HolonAST.**
   `event-data-value` lifts the Tagged WatAST through `eval-ast!`
   to a struct value; the matcher walks named fields. Field-keyed,
   not position-keyed.

2. **Type assertion is the form's first guard.**
   `(:trading::PaperResolved ...)` checks the subject IS that type
   before attempting field access. Schema drift on type rename or
   removal breaks queries loudly. Silent about un-mentioned fields
   means adding fields doesn't break queries — the right honesty
   for debugging-by-us.

3. **Inside the matcher, `=`/`<`/`>`/`and`/`or`/etc. are recognized
   without `:wat::core::` prefix.** Same convention as Clara's
   pattern syntax. The macro establishes a small special-form
   vocabulary inside `(TYPE-NAME ...)` clauses.

4. **Query-builder ergonomics: Clara-flavored constraint vec.**
   See §11 — settled on Option C. Same flat-list-of-constraints
   shape as the matcher; one mental model at two layers.

5. **Pivot to metrics is `foldl` over uuids.** Stages 3+4 in Query
   2. Each iteration spawns a fresh `stream-metrics` call. Correct
   but verbose. A future sugar `(stream-metrics-for-uuids db uuids)`
   could merge across uuids — but defer until a real script
   demands it.

## Open questions

These are the things we don't yet know that shape the rest. The
user is going to come at me with these; capturing them so the
answers land in the right place.

### §1. Form-matcher: sibling arc — SETTLED

**Settled.** Sibling arc (probably arc 097, or next free). Ships
the Clara-style `:wat::form::matches?` defmacro. Reusable beyond
telemetry. Arc 093 depends on it for slice 4's example scripts.

Per the user (2026-04-29): *"i have a such strong bias for clara
style - this is well understood? this is an amazing UX."*

What it ships: see the **Sibling arc — Clara-style form-matching
primitive** section above. Field-keyed bindings, constraint
interleaving, `?var` placeholders, recognized operator vocabulary
inside clauses (=, <, >, and, or, not, where).

What it does not ship: cross-fact joins, backtracking, full goal
language, forward-chaining rule firing. These arrive if real
consumers demand them.

### §2. `?var` syntax for placeholders — SETTLED

**Settled.** `?var` (Clara / Datalog / miniKanren / Prolog
tradition). No underscore-discard convention — Clara's field-keyed
binding makes `?_x` unnecessary; you only mention fields you care
about.

Lexer note: `?` as a leading character on a symbol probably already
parses (wat allows `?` in symbol bodies; predicates use it as
suffix). To verify in slice 1: write a probe with `(define ?x 1)`
and confirm it lexes. If not, slice 1 of the sibling arc adds the
lexer support.

### §3. Pagination size — SETTLED (no pagination; step-through)

**Settled.** No `page_size` knob. No LIMIT, no OFFSET, no keyset
cursor. The producer thread holds a prepared statement open and
calls `sqlite3_step` (via rusqlite's iterator) until `SQLITE_DONE`.
The bounded(1) channel between producer and consumer provides
backpressure naturally.

Per the user (2026-04-29):
> *"that is awesome — we need to use this — i think in streams"*

referring to SQLite's prepared-statement API, which streams one
row at a time without paging boundaries.

**The producer-thread loop:**

```rust
let mut stmt = conn.prepare(&sql)?;
let rows = stmt.query_map(params, |row| reify_event(row))?;
for row in rows {
    sender.send_blocking(row?)?;   // bounded(1) ack-paired backpressure
}
```

That's the whole thing. SQLite walks its index internally; the
prepared statement IS the cursor; rusqlite wraps `sqlite3_step`
as an iterator; the bounded channel rate-limits the producer to
the consumer's pace.

**WAL implications.** A long-running reader holds a transaction
open against a snapshot. In WAL mode (which arc 091 enables via
the writer's pragma policy), this:
- Does NOT block writers — they append to WAL.
- Does prevent WAL checkpoint from completing past the reader's
  snapshot point — WAL grows for the duration of the long read.
- Is fine for the debug use case: `runs/*.db` files are typically
  read post-hoc against frozen runs (no concurrent writer). When
  used against a live run.db, the long-read pins the WAL but
  doesn't impede write progress.

**Why we dropped the page_size knob.** Chapter 76 discipline:
don't ship a knob without a real consumer demanding it. With
step-through, there's nothing to size — each `step` is one row.
If a future consumer wants to throttle pre-fetch (e.g., bound
memory more aggressively than bounded(1) already does, or pre-
fetch ahead of the consumer for smoothness), add a channel-
buffer-size knob then. The substrate's bounded queue primitive
already supports any N.

**The wat-side UX is unchanged.** Stream<Event> from the consumer's
side. Filter, map, for-each as before. The pagination concept
the user explored earlier collapses entirely — there are no
pages, just a continuous SQLite-managed cursor walking the index.

### §4. Read-handle lifecycle — SETTLED

**Settled.** Drop closes the connection. Scope exit is the
cleanup event.

Per the user (2026-04-29): *"yes - scope exit is a clean up
event."*

No `(sqlite/close handle)` primitive. Same shape as every other
resource in the wat substrate:

- `ProgramHandle<T>` (arcs 011 / 012) — Drop joins the thread.
- `ThreadOwnedCell<T>` (arc 018) — Drop releases the cell.
- The writer-side `wat-telemetry-sqlite` Service handles
  (arc 091) — Drop closes the connection.

`(:wat::core::let* ((db (sqlite/open path)) ...) BODY)` opens at
let-binding; closes when the let scope exits. The user thinks
in scopes; the substrate enforces in scopes. Consistency over
explicitness.

### §5. Cross-database scripts — SETTLED (rejected)

**Settled.** Each script opens **one** read-handle against
**one** run.db. Cross-run analysis is not a feature.

Per the user (2026-04-29):

> *"not gonna happen for now... each run is an isolated environment
> - the sqlite is our ruby pry... our gdb into the system"*

Per-run isolation is the SHAPE. The script-against-one-db model
matches:

- **Ruby pry** — open a REPL into a frozen-state object graph;
  poke around; ask questions.
- **gdb** — attach to a stopped process; inspect state; step
  through.

Both are *deep dive into one frozen artifact*. Wat-as-scripting-
language for telemetry interrogation is the same shape: each
`runs/proof-003-1714514400.db` is one frozen environment; one
script attaches to it; questions get answered; script exits.

Cross-run comparison is a different question class (longitudinal
analysis) that wants different machinery — likely a higher-level
report tool that itself runs N scripts, not a single multi-db
handle. Out of scope for arc 093.

The substrate doesn't *prevent* a script from opening two
handles (the open primitive takes a path; nothing stops you
from calling it twice), but the design vocabulary doesn't
encourage it. No `LogQuery/across-dbs` builder; no merge
combinator across handles. If a real consumer demands it
later, that's its own arc.

### §6. Index set — SETTLED

**Settled.** Four indexes, set once, never touched again:

- `log.time_ns`
- `log.uuid`
- `log.namespace`
- `metric.start_time_ns`
- `metric.uuid`
- `metric.namespace`
- `metric.metric_name`

Per the user (2026-04-29):

> *"this is tied to our opinionated scheme for logs and metrics —
> we know what we these are because we are supplying them — i
> expect us to never touch these again"*

The schema is locked (substrate-defined Event::Metric +
Event::Log; arc 091). The columns are locked. The indexes match
the columns the query layer pushes down on. There's no tuning
surface — same as the schema itself doesn't have one. The
indexes are an *opinionated property of the substrate's
telemetry shape*, not a knob.

What this means concretely:
- `auto-spawn` (arc 085's schema derivation) gains
  `CREATE INDEX ... IF NOT EXISTS` statements alongside the
  `CREATE TABLE` it already emits.
- New databases get them at first-write time.
- Existing databases (already-shipped pulse runs, proof_002/3/4
  outputs) gain them when first opened by an arc-093 reader OR
  retroactively via a one-shot migration.
- We do not add a configuration knob like
  `(query/extra-indexes ...)`. Future indexes ship as substrate
  changes if the substrate's own queries demand them; not as
  consumer tunables.

No composite `(namespace, time_ns)` initially. The query
planner picks one index per query; with all the predicates in
slice 2 being equality-or-range single-column, the time index
plus column-equality is enough.

### §7. `run-with` vs `filter` over Stream — SETTLED (rejected)

**Settled.** No `run-with` primitive. Use `:wat::std::stream::filter`
(shipped via arcs 004 / 006 / 022) directly:

```scheme
(:wat::std::stream::filter
  (:wat::telemetry::sqlite/stream-logs db q)
  pred)
```

Per the user (2026-04-29):

> *"do not add - we'll run investigation tooling on the sqlite
> dbs after a run completes - these are decoupled from active
> runtime"*

The post-run / decoupled framing is worth recording: arc 093 is
purely an analysis layer against frozen .db files. There is no
in-process integration with the writer (arc 091); the writer
writes during runs and the reader reads after. The `stream-*` +
`filter` separation matches that boundary — `stream-*` is the
read-side primitive; `filter` is the substrate's general-purpose
in-app pruning combinator. Bundling them into `run-with` would
make the substrate look more entangled with telemetry-specific
sugar than it is.

The Ruby-Enumerator pipeline reads cleaner with the stages
visible separately:

```
stream-logs db q → filter pred → for-each
```

than fused:

```
run-with db q pred → for-each
```

The first form makes the *streaming* and the *filtering* visible
as separate stages; each composes with the rest of the substrate.
The second hides one inside the other and gains nothing.

### §8. Joining metric ↔ log via uuid — SETTLED (rejected)

**Settled.** No join sugar. The two-stage pivot (query one side
→ collect uuids → query the other side) is the user's analysis-
program shape, not boilerplate to hide.

Per the user (2026-04-29):

> *"i don't think we want this... our ux will demand it... i
> expect us to do a query on one side who'll reveal the need to
> query on the other side... our analysis programs will do this
> for us"*

The candidate sugars I'd considered and dropped:

- `(stream-scope db uuid) -> Stream<Event>` — both metric and
  log rows merged for a single uuid.
- `(stream-metrics-for-uuids db uuids) -> Stream<Event::Metric>`
  — multi-uuid fetch.
- `(stream-uuids db log-query) -> Stream<Bytes>` — cheap uuid
  projection without materializing full Events.

Each would collapse Stage 3+4 of Query 2 (uuid collection +
foldl over uuids) into one call. None ships.

**The pivot IS the analysis program's job.** The two-stage shape
matches the pry/gdb mental model exactly: a debugger doesn't
pre-fetch related state on your behalf; you ask for what you
want, look at it, ask the next question. The pivot from "I see
these uuids on the log side" to "now show me their metrics" is
the *content* of the debug session, not ceremony to hide. Sugar
that obscures it would make the substrate look like it knows
what the user wants before the user does.

Each query is a deliberate move. The script that walks logs →
uuids → metrics is doing the analysis; the substrate just
provides the two stream primitives and lets the script compose.

### §9. Sort order — SETTLED

**Settled.** Always ascending:
- `log` stream: `ORDER BY time_ns`
- `metric` stream: `ORDER BY start_time_ns`

Per the user (2026-04-29): *"sort order is by time for logs or
start-time for metrics."*

Rationale: time-indexed is the spec; ordering matches the index;
the SQLite query planner uses the time-column index for both the
WHERE pushdown and the ORDER BY (no separate sort step). Streams
are emitted oldest-first, which is the natural reading order for
post-hoc interrogation.

No `(query/sort-order ...)` builder; not user-overridable. If a
script wants reverse-time, it can `collect` and `Vec/reverse`.

### §10. Will indexes ride in slice 1 or slice 2? — SETTLED

**Settled.** Indexes ride in slice 1. Auto-spawn (arc 085's
schema derivation) gains the seven `CREATE INDEX ... IF NOT
EXISTS` statements alongside the table DDL. New databases get
the indexes at first write; existing pulse / proof databases
gain them when first opened by an arc-093 reader.

Per the user (2026-04-29): *"yes - we need the indexes and we
provide them."* Aligned with §6's "set once, never touch"
stance — the substrate provides the schema, provides the
indexes, owns the whole shape.

Why slice 1 and not slice 2:
- Slice 1 ships the read-handle and stream primitives. Even
  a "full table scan" benefits from the time index for ordering
  (the SQLite query planner uses it for ORDER BY).
- Slice 2 layers WHERE-clause assembly on top. Without the
  indexes already present, "push-down" is aspirational.
- Indexes have no downside at our scale — write throughput is
  ample, storage cost trivial, read benefit load-bearing.

### §11. Query builder ergonomics — SETTLED (Option C)

**Settled.** Option C — constraint-vec, Clara-flavored.

Per the user (2026-04-29): *"yes - C - we've already leaned into
clara - continue that lean."*

```scheme
(:wat::telemetry::log-query
  (:wat::core::vec :wat::telemetry::LogConstraint
    (:wat::telemetry::since hour-ago)
    (:wat::telemetry::caller :trading::test::proofs::003)
    (:wat::telemetry::level :error)))
```

The query is constructed by feeding a vec of constraints into a
single `log-query` (or `metric-query`) constructor. Each
constraint is a small value (variant of a `LogConstraint` /
`MetricConstraint` enum) produced by a per-field builder
function.

**Mental shape parity with the matcher (§1).** Both the QUERY
side and the MATCHER side have the same flat-list-of-constraints
shape — one model, applied twice:

```scheme
;; QUERY (SQL push-down):
(log-query
  (vec :LogConstraint
    (since hour-ago)
    (caller :trading::test::proofs::003)))

;; MATCHER (in-app prune):
(matches? data-value
  (:trading::PaperResolved
    (= ?outcome :outcome)
    (> ?grace 5.0)))
```

The script reads as two analogous flat lists at two layers.

**Substrate machinery.** Trivial:

```scheme
;; The constraint enums (substrate-defined, slice 2):
(:wat::core::enum :wat::telemetry::LogConstraint
  (Since (ts :i64))
  (Until (ts :i64))
  (Namespace (ns :wat::holon::HolonAST))
  (Caller (c :wat::holon::HolonAST))
  (Uuid (u :Bytes))
  (Level (lv :wat::holon::HolonAST)))

(:wat::core::enum :wat::telemetry::MetricConstraint
  (Since (ts :i64))
  (Until (ts :i64))
  (Namespace (ns :wat::holon::HolonAST))
  (Uuid (u :Bytes))
  (MetricName (n :wat::holon::HolonAST)))

;; The per-field builders (one-line each):
(:wat::core::define
  (:wat::telemetry::since (ts :i64) -> :wat::telemetry::LogConstraint)
  (:wat::telemetry::LogConstraint::Since ts))
;; ... etc, ten total across both enums.

;; The query constructors (one for each side):
(:wat::core::define
  (:wat::telemetry::log-query
    (cs :Vec<wat::telemetry::LogConstraint>)
    -> :wat::telemetry::LogQuery)
  ...)
```

The producer thread (slice 1) consumes the constraint Vec inside
the query value, walks each variant, assembles the WHERE clause.
No reflection, no introspection — direct match on enum variants.

**Why C, not A or B:**
- **C reads top-to-bottom**; A reads inside-out (visual hierarchy
  doesn't match constraint order).
- **C makes adding/removing constraints local**: add a line to
  the vec; remove a line; reorder freely. A requires identifying
  which layer to wrap/unwrap. B forces every callsite to
  enumerate all positions even when only 2-3 matter.
- **C is graceful under schema evolution**: adding a new
  filterable field is a new variant + a new builder; existing
  callsites don't break. B breaks every callsite.
- **C matches Clara's `[Type clause1 clause2 ...]` shape**, which
  §1 already committed to for the matcher. One mental model,
  two layers.

## Slice plan (tentative — firms up after open questions)

**Slice 1** — read-handle + step-through stream sources + sqlite indexes.
- `(sqlite/open path) -> ReadHandle`
- `(sqlite/stream-metrics handle q) -> Stream<Event::Metric>`
- `(sqlite/stream-logs handle q) -> Stream<Event::Log>`
- Producer-thread holds a prepared statement; `sqlite3_step`
  row-by-row (via rusqlite iterator); bounded(1) → consumer.
- Auto-spawn schema derivation gains indexes (time_ns / uuid /
  namespace / metric_name).
- Slice 1's `q` is a stub — empty Query — full-table scan via
  unconstrained SELECT. Slice 2 fills the WHERE clause assembly.

**Slice 2** — Constraint enums + Clara-flavored builders + push-down.
- `:wat::telemetry::LogConstraint` enum (Since / Until / Namespace
  / Caller / Uuid / Level variants).
- `:wat::telemetry::MetricConstraint` enum (Since / Until /
  Namespace / Uuid / MetricName variants).
- Per-field builder functions (`since`, `until`, `namespace`,
  `caller`, `uuid`, `level`, `metric-name`) — one-line `define`s
  that wrap their argument in the corresponding enum variant.
- Query constructors: `(log-query (vec :LogConstraint ...))` and
  `(metric-query (vec :MetricConstraint ...))` — consume the vec,
  walk variants, accumulate WHERE clauses.
- Producer in slice 1 now matches on each constraint variant
  inside the query → assembles SQL WHERE clause.
- See §11 for the full shape and rationale.

**Slice 3** — Materialization helpers.
- `(:wat::telemetry::Event::Log/data-ast e) -> :Option<HolonAST>`
- `(:wat::telemetry::Event::Log/data-value e) -> :Option<Value>`
  (lifts via `:wat::eval-ast!`).
- These are the bridge from row bytes to a struct value the
  Clara-style matcher consumes.

**Slice 4** — Example scripts (depends on sibling Clara-matcher arc).
- Direct port of the Ruby Enumerator loop (stream + filter +
  for-each).
- Both worked examples from the DESIGN above, runnable against
  a real `runs/pulse-*.db`. Lives in `wat-rs/examples/interrogate/`
  (TBD on exact location).
- This slice **gates on the sibling arc shipping** the Clara-style
  `:wat::form::matches?` macro. Either schedule the sibling first,
  or land slice 1+2+3 of arc 093 with the matcher TBD and pick up
  slice 4 once the sibling closes.

**Slice 5** — INSCRIPTION + CIRCUIT.md update + 058 FOUNDATION-CHANGELOG row.

---

## Predecessors / dependencies

**Shipped:**
- arc 091 (writer + sqlite schema + struct->form + eval-ast!) — shipped 2026-04-29
- `:wat::std::stream::*` (Stream<T> + combinators) — shipped via arcs 004 / 006 / 022
- arc 095 (Service<E,G> paired channels) — shipped 2026-04-29 (writer side; reader doesn't ride on it but the connection-pool concepts settled there)
- `:wat::time::*` shipped surface: `now`, `epoch-{nanos,millis,seconds}`, `at-{nanos,millis}`, `at`, `from-iso8601`, `to-iso8601`. Conversion primitives only.

**Sibling arcs:**

- **Arc 097 — `:wat::time::Duration` + arithmetic +
  ActiveSupport-flavored helpers — SHIPPED 2026-04-29**
  ([INSCRIPTION](../097-wat-time-duration/INSCRIPTION.md)).
  Provides `Value::Duration(i64)` runtime variant, 7 unit
  constructors (`Hour` / `Minute` / etc., PascalCase),
  polymorphic `:wat::time::-` (Instant - Duration → Instant;
  Instant - Instant → Duration), `:wat::time::+`,
  `:wat::time::ago` / `from-now` composers, and 14 pre-composed
  unit sugars (`hours-ago` / `days-from-now` / etc.). Arc 093's
  `Since(Instant)` / `Until(Instant)` constraint variants and
  the worked-example queries depend on these.

- **Clara-style form-matcher** (`:wat::form::matches?` macro;
  see Sibling arc — Clara-style form-matching primitive section).
  TBD arc number; slice 4 of 093 depends on it for example
  scripts.

## What this enables

- Post-hoc debugging via wat scripts against any run.db.
- Reusable script templates (Ruby's "I keep coming back to the same
  shape" use case).
- The form-matcher (sibling arc) reusable in non-telemetry contexts —
  any pattern-match against any HolonAST.
- Foundation for arc 094 (circuit.wat) and any future automated
  consumer that wants to ask the substrate "what happened during
  this run?"

**PERSEVERARE.**
