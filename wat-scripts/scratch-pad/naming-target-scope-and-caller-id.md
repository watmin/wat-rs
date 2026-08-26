# NAMING TARGET — (1) the relocated correlation surface's home, (2) the caller-identity field

> **Materialized for an intueri cast** (R17 self-prompt-injection: give the ward a real artifact).
> Two DISTINCT decisions in one target because they will be read together, in one form, by one
> reader. Judge them separately; note any interaction.

---

# DECISION 1 — the namespace + type name for the relocated correlation surface

## What it is, and why it is moving

`:wat::telemetry::Scope` exists today (`wat/telemetry.wat:71-79`) and is described in its own header as
**"the EXACT surface every telemetry record satisfies (identity + when)"**:

```clojure
;; namespace (facility), uuid (correlation id), tags (dimensions), time-ns (event time).
(:wat::core::defsurface :wat::telemetry::Scope
  :nature :wat::core::Record
  :features [namespace <- :wat::core::String
             uuid      <- :wat::core::Uuid
             tags      <- :wat::telemetry::Tags       ;; typealias: HashMap<keyword,String>
             time-ns   <- :wat::core::i64])
```

`Metric` and `Log` splice it (`~@:wat::telemetry::Scope`). **It is about to gain a second, unrelated
consumer:** a per-CALL context handed to every `defservice` op handler, which needs the SAME
correlation id the logs carry (otherwise "correlation id" correlates nothing).

**Load order forces the move.** `wat/service.wat` registers at `src/stdlib.rs:324`;
`wat/telemetry.wat` at `:422`. Service loads ~100 entries FIRST, so it cannot name a
`:wat::telemetry::` type. The surface must relocate to a file that loads before both.

**Precedent for exactly this move** — `stdlib.rs:134`, `wat/capability.wat`:
> *"Relocated here (from wat/service.wat's old position ~328) so it loads BEFORE wat/spawn.wat and
> wat/bracket.wat, both of which name it. Deps only on core.wat builtins."*

## The problem with keeping the name

If the surface lives in a shared, early-loading file but is still called `:wat::telemetry::Scope`,
**the namespace says "telemetry" while the thing is shared substrate vocabulary.** A service handler
would name a `telemetry::` type to talk about its own request. That is a small lie of the exact kind
this substrate keeps pulling out.

## Where the name is READ

- In a `defservice` handler's own signature/body, talking about the CURRENT REQUEST.
- In `Metric`/`Log`'s splice line: `[~@<TheName> …]`.
- As a field type in a service's ctx record.
- It is a **surface**, so it also reads as a contract: *"anything that satisfies `<TheName>` carries an
  identity and a time."*

## ANCHORS — the live `:wat::` namespace set, by weight (from the corpus)

```
core(8341)  kernel(783)  rete(654)  query(431)  telemetry(330)  stream(208)  fix(201)
spawn(150)  holon(127)   sqlite(111) cache(110)  lint(93)        test(75)    deporder(57)
service(56) bracket(56)  enum(50)   io(31)      program(28)     edn(27)     capability(21)
source(20)
```

Note the shape of the small ones: `capability`, `program`, `source`, `edn`, `enum` — each names a
**concept**, not a subsystem that owns it.

## CANDIDATES — decision 1

Namespace + type, e.g. `:wat::<ns>::<Type>`:

1. `:wat::telemetry::Scope` — **keep, just move the file.** (Costs: the namespace lies about ownership.)
2. `:wat::scope::Scope` — stutters.
3. `:wat::core::Scope` — put it in the existing core vocabulary.
4. `:wat::trace::Scope` — names the concern (correlation/tracing).
5. `:wat::correlation::Scope`
6. `:wat::scope::Correlated`
7. `:wat::core::Correlation`
8. `:wat::observe::Scope`
9. Keep `Scope` as the type, pick the namespace only.
10. Rename the TYPE too — `Scope` may itself be the mumble (a "scope" in most languages means lexical
    scope, which this is not).

**Weigh #10 seriously.** The word `scope` is heavily loaded in a programming language substrate — this
project already uses "scope" for lexical scope (`HygieneScopeDivergence`, `SandboxScopeLeak`,
`outer_symbols`), for `#[wat_dispatch(scope = "thread_owned")]`, and for the sandbox. Is a correlation
surface called `Scope` a collision the reader must disambiguate every time?

---

# DECISION 2 — the caller-identity field on the per-call context

## What it is

The ctx record handed to a caller-ful handler is `{...<the surface above>, <this field>}`. This field
**names WHICH connected client is making this call** — a value minted when a connection is accepted,
never reused, stable across the connection's whole life, and used to look up that tenant's
per-connection state (their compiled rules, their cursor).

Critically: it must be **STABLE**, because the substrate's existing identifier for a client is its
POSITION in a vector (`idx`), and every eviction is `remove-at selectables idx` — so positions shift
and a position-keyed lookup silently hands one tenant another's state.

## Two names are already in circulation for this ONE thing — that is the problem

- **`ConnId`** — used throughout `DESIGN-STONE-the-connection-scoped-world.md`.
- **`resource-id`** — the builder's word, 2026-08-09.

A substrate that ends up with both has two names for one concept. **Pick one.**

## Where the name is READ

```clojure
(sift-rules [s ctx req]
  (:wat::core::let [who   (:wat::service::Ctx/<the-field> ctx)
                    world (:wat::hashmap::get (worlds s) who)]
    …))
```

## ANCHORS — how identity-bearing fields are named in this corpus today

```
node-id(71)  next-id(68)  parent-id(41)  uuid(37)  pid(28)  alpha-id(26)
child-id(16) from-alpha-id(11) prod-id(7)
```

The house style is plainly `<noun>-id`, kebab, with `uuid` and `pid` as the two bare exceptions
(both borrowed, both universally understood).

## CANDIDATES — decision 2

1. `resource-id` — the builder's word.
2. `conn-id` — the stone's word, kebab-cased to house style.
3. `client-id`
4. `peer-id` — the substrate calls a connected counterpart a `Peer` everywhere (`Peer'<I,O>`, `poll'`,
   `connect'`); this would reuse the existing noun.
5. `caller-id`
6. `session-id`
7. `tenant-id` — the multi-tenancy word; the whole stone exists to stop tenants intermingling.
8. `origin-id`

## The questions for the cast

**For decision 1:**
- Which namespace keeps its promise for a reader in a `defservice` handler who is NOT doing telemetry?
- Is `Scope` the right TYPE name at all, given this substrate's existing heavy use of "scope" for
  lexical scope, sandbox scope, and `wat_dispatch(scope=…)`? If it is a collision, say so and propose.
- Does moving it into `:wat::core::` overload core, or is that exactly what core is for?

**For decision 2:**
- Which of `resource-id` / `conn-id` / `peer-id` / `tenant-id` names what the value IS, rather than
  what it is used for? (The value identifies a connected client; it is USED to select tenant state.)
- `peer-id` reuses the substrate's own noun for a connected counterpart — is that a strength
  (consistency) or a hazard (a `Peer` is a live resource; this is a pure id, and conflating them is
  precisely the error the purity wall exists to prevent)?
- Does `resource-id` name a resource? The thing identified is a *connection*, and the id itself is
  pure data. Is "resource" honest here, or is it borrowing a word that means something specific in
  this substrate (a live handle that cannot cross the wire)?

Return: ONE recommendation per decision, each with a runner-up and why it lost. Flag any interaction
between the two (e.g. if the surface's name makes one field name read better or worse).
