# BRIEF — a batch declares how many

Add `:max-entries [field N]` as a surface-feature option, enforce it client-side before the send
exactly as `:max-request-bytes` is enforced, and adopt it on `Queue::send` only. Read
`DESIGN.md` first — it carries the contract decision and the compile-error
wall.

## READ IN ORDER

| room | why |
|---|---|
| `src/types/surface.rs:441` | the `":max-request-bytes" =>` arm in the option `match` — your exemplar for parsing, duplicate-detection and the positive-literal check |
| `src/types/surface.rs:492` | the unrecognized-option error, which **lists the recognized options by name** — it must learn the new one, or the diagnostic lies |
| `src/types/surface.rs:504` | `max_request_bytes_explicit` — how explicitness is captured *before* defaulting. `:max-entries` has **no default**: absent means uncapped |
| `wat/service.wat:1720`, `:1733` | `cap-const-kw` and `rtl-ctor-kw` — how a cap constant and a response constructor are derived at macro-expand time. Yours are the same shape |
| `wat/service.wat:1927-1931` | emission site **one** |
| `wat/service.wat:2293-2297` | emission site **two** — the `peer-wire?`-gated guard quoted in the DESIGN |
| `wat-scripts/queue/sqs.wat:44` | `Queue::SendRequest [queue bodies now-ns]` — `bodies` is the field you cap |
| `wat-scripts/queue/sqs.wat:48-53` | `Queue::SendResponse` — where `RequestTooManyEntries` is added, beside `RequestTooLarge` |
| `wat-scripts/queue/sqs.wat:101` | the `send` feature — where the option is declared |

★ **There are TWO emission sites, not one.** Find both before writing either; a guard added to
one and not the other is a cap that fires depending on how you dialled.

## SKETCH

Surface declaration:

```wat
(send [self <- :queue::Queue  req <- :queue::Queue::SendRequest]
  -> :queue::Queue::SendResponse
  :max-request-bytes 524288
  :max-entries [bodies 10])
```

Response gains, beside `RequestTooLarge`:

```wat
:RequestTooManyEntries [entries <- :wat::core::i64  cap <- :wat::core::i64]
```

Generated guard — **before** the byte guard, and NOT behind `peer-wire?`. A count violation is a
contract violation at every tier; only the byte measure needs a wire because only it costs an
encode:

```wat
(:wat::core::let [~k-sym (:wat::core::count (~field-accessor-kw req))]
  (:wat::core::if (:wat::i64::> ~k-sym ~entries-cap-kw)
    (:wat::kernel::RecvOutcome::Message (~rte-ctor-kw ~k-sym ~entries-cap-kw))
    <the existing peer-wire? byte guard, unchanged>))
```

`~field-accessor-kw` is derived from the request type and the declared field name the same way
`rtl-ctor-kw` is derived from the response type at `wat/service.wat:1733`.

## THE WALL

Declaring `:max-entries` on a feature whose response enum has no `RequestTooManyEntries` variant
must be a **compile error naming the missing variant** — not a silent skip, not a fallback to
`RequestTooLarge`. Write the failing case as a test.

## BLAST RADIUS

`src/types/surface.rs`, `wat/service.wat`, `wat-scripts/queue/sqs.wat`, and probes/tests.
**`Store::put/delete`, `Queue::ack` and `Seen::mark` are NOT adopted here** — one adopter, then a
sweep in its own stone.

## STOP TRIGGERS

- **STOP-1** — the option value cannot be a vector (`[bodies 10]`) because the parser takes only
  scalar literals. Report the exact parser constraint; do **not** fall back to a bare
  `:max-entries 10` with field discovery — the DESIGN rejects that on Obvious.
- **STOP-2** — the field accessor cannot be derived at macro-expand time from the request type and
  a field name. Report what `rtl-ctor-kw`'s derivation has that this lacks.
- **STOP-3** — the two emission sites cannot take the same guard. That asymmetry is the finding;
  surface it rather than guarding one.
- **STOP-4** — enforcing the cap requires the count at a tier that cannot see the field (e.g. a
  decoded-but-untyped request). Report where.
- **STOP-5** — anything outside the blast radius. In particular: **do not adopt the option on a
  second surface.**

## THE PROBE YOU WILL NEED

A scratch probe that sends 11 bodies to a `Queue::send` capped at 10 and shows
`RequestTooManyEntries{11,10}` **with no message enqueued** — depth unchanged — at **both**
`:locus (:wat::spawn::thread)` and `:locus (:wat::spawn::process)`, because the byte guard is
wire-gated and this one must not be.

## GRADE AGAINST

`docs/excursus/2026/08/001-sns-sqs/a-single-value-is-a-batch-of-one/SCORE.md` — the arc's prior surface-shape stone.

Write `SCORE.md`, then `pulsare_yield kind=scored`.

---

# ⛔⛔ AMENDED MID-STRIKE — THE FIELD MUST BE A SEQUENCE

Raised by the builder against the in-flight work: *"this 'type' is unqualified — is this for maps?
for sets? for vectors? lists? record members?"* It is a real hole, and the BRIEF above never
closed it. **This section overrides nothing above; it adds a requirement.**

## WHAT IS ALREADY RIGHT — keep it

`src/macros/expand.rs` verifies the **response** carries `RequestTooManyEntries` with the exact
field shape (`RTE_VARIANT`, `rte_fields`). That is the wall the DESIGN asked for and it is built.
Nothing here changes it.

## DEFECT A — "entries" counts six things and means three

The guard emits `(:wat::core::count (~field-acc-kw ~req-binder))`. Measured, this session, by
driving `count` on a `String`:

```
:wat::core::count: expected (Vector :- [T]), (HashMap :- [K V]), (PersistentMap :- [K V]),
                            (PersistentVector :- [T]), (HashSet :- [T]), or (List :- [T])
```

| field type | what `count` returns | what "entries" would mean |
|---|---|---|
| `Vector` / `PersistentVector` / `List` | elements | ✅ the intended unit |
| `HashMap` / `PersistentMap` | key–value **pairs** | a different unit |
| `HashSet` | **distinct** members | ⛔ 15 duplicates pass a cap of 10 |

★ `String` is rejected, so character-counting is unreachable. That is the only case luck covered.

★★ `RequestTooManyEntries [entries cap]` is **one word for three units**. A caller cannot tell
which it was told. That is the failure class this arc has spent itself closing — the three `_`
arms speaking for four store outcomes, `Lost` asserting *"transient, exhausted after 3"*.

## DEFECT B — the field is validated only as "a name"

`src/types/surface.rs:668` accepts any `Symbol` or `Keyword`. **Nothing checks the field exists on
the request record**, and nothing checks its type. `:max-entries [now-ns 10]` on an `i64` field
declares cleanly and fails later, far from the declaration that caused it.

## ⛔ THE REQUIREMENT — one check closes both

At **declaration time**, resolve the named field on the request record and require its declared
type to be a **sequence**: `Vector`, `PersistentVector`, or `List`.

- resolving it proves the field **exists** → closes B
- restricting it makes **"entries" mean elements, and only elements** → closes A

Two errors, each naming its own cause — never one message for both:

```
method member `send`: `:max-entries` field `bodys` is not a field of
  `:queue::Queue::SendRequest` (fields: queue, bodies, now-ns)

method member `send`: `:max-entries` counts ELEMENTS; field `tags` is a
  (HashSet :- [String]), whose count is distinct members. Declare the cap on a
  sequence field (Vector, PersistentVector or List).
```

★★★ The name stays `:max-entries`. Renaming it `:max-vector-entries` would be a longer label that
**still permits a `HashMap` field and still never checks existence** — a label where a wall
belongs. `entries` is also SQS's own vocabulary (`TooManyEntriesInBatchRequest`). The ambiguity is
removed by making the wrong declaration **unrepresentable**, not by describing it in the name.

## STOP-6 (new)

If the request record's field types are **not reachable** from where `:max-entries` is parsed —
i.e. the surface parser sees the option before the request type is resolved — **STOP and report
where the check would have to live instead.** Do not fall back to a runtime check: a declaration
error that surfaces at runtime is the defect, not the fix.

## ROWS THIS ADDS

Both are gates.

| # | what must hold | expected |
|---|---|---|
| 12 | ⛔ a field that does not exist is a **declaration-time error** | names the field and lists the record's actual fields |
| 13 | ⛔ a `HashMap` / `HashSet` field is a **declaration-time error** | names the type and says the cap counts elements |

---

# ⛔⛔⛔ AMENDED AGAIN — CARRY THE CAP AS A DEF, NOT A STASH

Builder, on the in-flight `:wat::types::max-entries` intrinsic: *"this 'max' is a service
thing... it has no meaning for types?"* It does not, and pulling that thread found three
compounding problems. **This replaces the mechanism, not the requirement.**

## THE EXEMPLAR I POINTED AT WAS THE WRONG HALF

The rooms above cite `wat/service.wat:1720`, where `cap-const-kw` is **consumed**. The half that
matters is where it is **emitted**:

**`src/types.rs:3647-3664`** — `defsurface` registration builds one real wat def per op:

```
(:wat::core::def :<Surface>::<OP>-MAX-REQUEST-BYTES <n>)
```

★ It is a **value in the program**. `wat/telemetry.wat:387` reads it by name —
`:wat::telemetry::Journal::WRITE-LOGS-MAX-REQUEST-BYTES`. `defservice` merely rebuilds that name
by string interpolation. No intrinsic. No reflection. No state.

## WHY THE STASH MUST GO — four questions, 4-NO

`src/types/surface.rs:43` is a `thread_local! { static MAX_ENTRIES: RefCell<HashMap<...>> }`,
written at `:100`, read at `:165`, **never cleared**, behind a new `:wat::types::max-entries`
intrinsic — the ONLY intrinsic in that namespace.

- **Obvious? NO.** A request cap is a surface/service concept. `:wat::types::` is where it has no
  meaning, and the namespace was created to hold this one thing.
- **Simple? NO.** walk → stash → key → retrieve → new namespace, against name → value.
- **Honest? NO — and this is a live bug.** The stash is never cleared and is keyed by
  `(surface, op)`. In a long-lived process — `wat --mcp` — a later program whose surface omits
  `:max-entries` finds an earlier program's entry under the same key and **emits a guard it never
  declared.** That directly violates EXPECTATIONS row 5, *absent means uncapped*.
- **Good UX? NO.** The byte cap is readable by any wat program; this one is visible only to the
  macro. A caller cannot ask *"what is the max batch size?"* — the one question a batch API exists
  to answer. SQS publishes its limits.

## ⛔ THE MECHANISM

Emit **two** defs per op from surface registration, beside the byte-cap def at `src/types.rs:3664`:

```
:<Surface>::<OP>-MAX-ENTRIES        <n>            ;; i64
:<Surface>::<OP>-MAX-ENTRIES-FIELD  "<field>"      ;; String
```

Absent option → **emit neither**, and `defservice` emits no guard. Absence is the absence of a
def, not a zero in a table — there is then no state in which a stale value can be read.

In `defservice`, build those two names beside `cap-const-kw` (`wat/service.wat:1720`) with the
same `keyword::from-string` + `string::interpolate` shape.

**Delete** `:wat::types::max-entries`, `stash_max_entries_from_defsurface`, the `MAX_ENTRIES`
thread_local, and the `:wat::types::` namespace registration.

## WHAT IS UNCHANGED

Everything in the first amendment stands: the field must resolve on the request record and its
type must be a **sequence** (rows 12 and 13). And the response-side wall in `src/macros/expand.rs`
— `RTE_VARIANT` with its exact field shape — is correct and stays.

## ROWS THIS ADDS

| # | what must hold | expected |
|---|---|---|
| 14 | ⛔ **the cap is a readable def** | `:queue::Queue::SEND-MAX-ENTRIES` evaluates to `10` from ordinary wat, as the byte-cap const does |
| 15 | ⛔ **no expand-time mutable state** | `grep -n "thread_local\|RefCell" src/types/surface.rs` finds none added by this stone |
| 16 | ⛔ **no `:wat::types::` intrinsic** | `grep -rn 'wat_intrinsic(":wat::types::' src/` is empty |

## STOP-7 (new)

If a def cannot carry the field **name** (a String const beside an i64 const is not supported at
`types.rs:3664`), **STOP and report what that emitter accepts.** Do not reintroduce a stash to
carry the half a def cannot.
