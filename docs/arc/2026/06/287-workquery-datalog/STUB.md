# Arc 287 — WorkQuery v2: a Datalog query surface (over the rete kernel) replacing forms (STUB, banked)

**Status:** STUB / banked. Not built. Future arc. Captured 2026-06-19 from the forms-vs-rete assessment + the
builder's reframe. (Forward-marker, not a commitment — refine the open questions when you open it.)

## The realization (the trigger)
The "forms tooling" is `:wat::telemetry::WorkQuery` (arc 093) + `:wat::form::matches?` (arc 098), proven in
`examples/interrogate/` against SQLite. Its actual job: a **pull-query over stored record collections** —
*narrow by time at the SQL layer (index scan / candidate reduction), then filter in-app with `form::matches?`
constraint predicates over lifted struct values.* DDB-server-side-filter / "select * where index-constraints,
then client-filter" — SQL-avoidant by design.

**We were reaching for a Prolog when we wrote forms** — arc 091's INSCRIPTION named WorkQuery verbatim:
*"Time-indexed queries; prolog-y unify; combinators; bidirectional join."* The SQL-narrow + `form::matches?`
filter was the v1 stand-in for a logic-query never built. §8 (metric↔log uuid join) was deferred to
"the analysis program's job" — i.e. a relational join we punted.

## The verdict (paradigm)
- WorkQuery is a **declarative PULL query** (ask a frozen db, get matching records). RETE is forward-chaining
  **PUSH**. Opposite directions — BUT **Datalog unifies them**: a query *is* a rule evaluated bottom-up to
  fixpoint. Datalog ≈ rete (semi-naive forward fixpoint over joins) = the exact algorithm arc-278 P4b shipped.
- **Datalog is the fit**, not full Prolog/core.logic. Datalog: terminating, set-semantics, joins, index-friendly
  — precise for "records where constraints + joins." Prolog/core.logic (unification + backtracking + function
  terms) is more power than a record-query needs and risks non-termination; reach for it only if open-ended
  search / generative relations are needed (a filter-query isn't).
- Distinct from the **lint/AST codemod** (`wat/lint.wat`) — that's term-rewriting, stays as-is, no logic-prog.

## The shape (the kicker — reuse, don't rebuild)
A WorkQuery is expressible as a rete rule TODAY: constraints + the metric⋈log join as LHS → a `Result` fact as
RHS; fire → every match derives a `Result`; `query-by-type` → the answer set. The machinery already exists:
- **the join** = the hash-join shipped in stone 3b (§8's bidirectional uuid join is just a keyed join).
- **the filter** = the α-matcher (Clara-style constraints).
- **the fixpoint** = `fire-rules'` (semi-naive delta, P4b).

So arc 287 is plausibly a **Datalog query SURFACE over the rete kernel**, NOT a second engine. What rete lacks,
and 287 must add:
1. **An ad-hoc QUERY surface** — a query DSL (not standing `defrule`s you recompile per question); a query is a
   transient rule, fired, collected. Ergonomics of "ask a one-off question."
2. **An index access-path** — rete loads ALL facts into working memory; WorkQuery's whole point is
   candidate-reduction FIRST (the SQL-narrow). 287 needs an indexed/time-bounded scan that feeds only candidate
   records into the query, not the whole store. This is the storage/access-path layer (SQLite was v1; modern
   could be wat-native or keep an external store).
3. **Unify the matcher** — the constraint-matcher exists TWICE (rete's α + `form::matches?`, parallel Clara-style
   impls). One Clara-style constraint language should serve both rete's α-layer and WorkQuery's filter.

## Open questions (decide at arc-open)
- Reuse the rete kernel (query = transient rule) vs a standalone Datalog engine? (Reuse is the strong prior —
  the kernel IS semi-naive Datalog — but ad-hoc query ergonomics + the index access-path may argue for a thin
  dedicated surface.)
- The access-path: wat-native indexed store, or keep an external (SQLite/other) candidate-reducer feeding the
  query? (WorkQuery v1's SQL-narrow proved the two-layer model; v2 decides where the index lives.)
- Recursion/transitivity needed (Datalog recursive rules — graph reachability over records) or flat
  filter+join only? (Flat is what v1 did; recursion is the Datalog upside if a use appears.)
- Does this retire `form::matches?` and `:wat::telemetry::WorkQuery`, or layer over them?

## Four-questions (sketch)
- **Obvious?** YES — "query records by constraints + joins" is a declarative query; Datalog is its canonical form.
- **Simple?** The SURFACE is simple if it reuses the rete kernel; the access-path is the real new work.
- **Honest?** A real logic-query replaces the SQL+client-filter stand-in we admitted was a stand-in.
- **Good UX?** SQL-avoidant declarative queries in wat — the "ruby of sorts / pry-gdb over a frozen db" UX arc
  093 wanted, now first-class.

## The query loop (rete IS the evaluator — grounded 2026-06-19)
A WorkQuery runs ON the rete engine, no separate datalog runtime:
1. **ingest** — `insert` a page of records (from the index narrow) as facts into a Session (working memory).
2. **query** — a rule: LHS = constraints (+ joins via stone-3b hash-join), RHS = `(insert (Result …))`. `fire-rules`.
   Recursive/transitive queries fall out of the P4b fixpoint.
3. **collect** — `query-by-type Result` → the answer set.
"Shoot a page of data into WM, run the constraints, query what fell out." The joins are 3b; recursion is P4b.

## Remote managed-DB service (later — substrate already exists)
The two-phase model is **DDB's own**: `KeyConditionExpression` (index, server-narrows) + `FilterExpression`
(post-narrow filter) ≡ our **SQLite/index narrow + rete-rule filter**. That makes a *remote managed query DB*
a real, near-term-possible composition — and the service substrate is already built:
- **store**: sqlite `ReadHandle` (arc 093) or a wat-native index.
- **engine**: arc 278 rete (this).
- **service + wire**: `defservice` (actor/gen_server), peers, UDS + the **rendezvous capability** (arc 272),
  **SO_PEERCRED** capability gating, **EdnRepresentable** wire (arc 280). A remote caller sends
  `{key-conditions, rules}` as EDN; the service narrows + rete-filters; returns records as EDN.
- **the bigger face**: the SAME service = the DDoS/anomaly engine — ingest + rete exact-match **∪ VSA
  similarity-match** (the designed matcher seam, DESIGN.md:52) → "what's anomalous," answered over the wire at
  line rate. The holonic endgame: rules at the line, exact ∪ similar, capability-secure, remote.
- NOT a now-thing; a real later-arc. Depends on 287 (the query surface) + 280 (the wire) + 278 (the engine).

## Relations
- Builds on arc 278 (the rete kernel — the join/fixpoint/matcher it reuses).
- Supersedes/revisits arc 093 (`:wat::telemetry::WorkQuery`) + arc 098 (`:wat::form::matches?`).
- NOT the lint codemod (`wat/lint.wat`) — orthogonal, stays.
- Open any time after 278 closes; not a 278 dependency.
