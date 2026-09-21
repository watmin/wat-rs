# BRIEF — STONE 255.8: a namespace that is ALSO a type

**Drawn 2026-09-21 against `main` @ `0d6da5f19`** (floor 5957/5957, clippy 0, census `no STOP-8`).
Delta baseline **49**. ⛔ **251.8d-ii is STOPPED on this. It is the only thing between here and the
stdlib bootstrap.**

## ⭐ THE STOP WAS CORRECT AND THE STDLIB EARNED ITS KEEP

8d-ii converted all 64 stdlib files (`64/64 changed`, dry-run byte-identical to the applied run,
`cargo build` exit 0) — and then **the new binary would not start**. The executor restored `wat/`,
reported, and did not force it.

⭐ **This is exactly the value the redraw predicted:** the stdlib exercises four stones at once, and
it found a defect in **255.3** that 2,145 corpus files never surfaced.

⚠ **Two witnesses, different first names** (`wat.telemetry.Journal/QueryMetricsRequest`, then
`wat.query.Store/EnsureSchemaRequest`) — because `validate_named_type_annotations` walks
`env.iter()` and returns the **first** miss, and `HashMap` order changes per process. ⭐ **The
executor named that rather than calling it a flake.** Same class, two witnesses.

## ⛔ THE DEFECT — 255.3's join fires where identity belongs

```wat
(wat.core/defrecord wat.telemetry.Journal/QueryMetricsRequest …)   ; the DECLARATION
(query-metrics [req :- wat.telemetry.Journal/QueryMetricsRequest]) ; the ANNOTATION
```

| door | result |
|---|---|
| `canonical_identity` / `ns_to_wat_path` — **registers** the declaration | `:wat::telemetry::Journal::QueryMetricsRequest` |
| `reconstruct_call_path` (255.3) — **resolves** the annotation | `:wat::telemetry::Journal/QueryMetricsRequest` |

`Journal` **is** a known type, so 255.3 joins the tail as a **member**. But
`wat.telemetry.Journal` is **also a namespace**, and `QueryMetricsRequest` is a **nested type name**,
not a method. `canonical_identity` leaves a string that already contains `::` alone, so the `/` never
folds back — and the annotation wall reports `UnknownNamedType`.

⭐ **REPRODUCED MINIMALLY** (orchestrator, 3 lines):

```
(:wat::core::defrecord :my::Journal::Req …)  + annotation  → CLEAN
(wat.core/defrecord my.Journal/Req …)        + annotation  → :path ":my::Journal/Req"  ⛔
```

## ⛔⛔ THE FUNCTION'S OWN DOC ALREADY STATES THE RULE ITS BODY BREAKS

`src/types.rs:172-174`:

> *"Identity reconstruction (`canonical_identity` / `ns_to_wat_path`) stays `::` always:
> **a type name `my.Counter/Req` is not a method.** 255.4: a member join is `/`, always."*

⛔ **255.3 wrote that sentence and then applied the member join to a type name anyway.** The cure is
to make the code obey the comment — **not to weaken the comment.**

## ⭐ THE REAL QUESTION — and it is NOT "add a special case"

`Type/member` and `Namespace.Type/NestedType` are the **same shape**. 255.3 already measured that
last-segment-is-a-type is **necessary and not sufficient**, and 255.4 answered one half by unifying
the member join. **This is the other half.**

**Candidate discriminators — measure, do not pick from this list:**

| | |
|---|---|
| **position** | an ANNOTATION slot is never a call head. 255.5 plumbed `also_accept_type` for exactly this — ⭐ **does the annotation path reach `reconstruct_call_path` at all, and should it?** |
| **registration** | `:wat::telemetry::Journal::QueryMetricsRequest` **is registered**; the `/` spelling is not. **Ask which one the registry holds.** |
| **arity of the question** | `reconstruct_call_path` is named for CALL paths. ⚠ **If a type annotation is reaching it, the bug may be the ROUTING, not the join.** |

⛔ **Find which door the annotation should use, not which extra test the join needs.** A second
predicate inside `reconstruct_call_path` is the shape this arc has rejected four times.

## The work

1. **Locate the path** by which a type annotation reaches `reconstruct_call_path`. ⚠ **State it.**
   If it should never arrive there, the fix is routing and the join is innocent.
2. **Fix it at the door** the classification names.
3. **RE-RUN THE DELTA** (baseline **49**) + classification, one named tree, re-converted with the
   current codemod.
4. ⭐ **THEN RE-ATTEMPT 8d-ii's STEP 3** — convert `wat/` on **copies**, rebuild, and prove the
   binary starts. ⛔ **Do not apply to live `wat/` in this stone.** The bootstrap is 8d-ii's; this
   stone only has to make it possible.

## The gate

- ⭐ **The minimal repro above goes CLEAN**, and the colon spelling **still** does.
- ⛔ **NON-VACUITY:** `Option/expect` (a genuine `Type/method`) **still resolves as a member** —
  ⭐ **this stone must not undo 255.3.** 4,501 member sites depend on that join.
  ⚠ **Assert both in one test**, or a future hand will "fix" one by breaking the other.
- **A converted stdlib binary STARTS** — build from copies, run it. That is 8d-ii's gate arriving
  early, and it is the only proof that matters here.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
- ⛔ **NOT ONE `.wat` CONVERTED** in the live tree.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** ⭐ The executor proved this one is not, by naming
  the `HashMap`-order mechanism behind two different witnesses. **Do that again.**
- ⛔ **ONE DOOR.** Four stones have been spent removing second doors. Do not add one here.
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Eight stones running.
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** **Fourteen corrections across
  twelve stones** — the last was the orchestrator misreading its own glob and quoting the WRONG
  SCORE FILE back at you. **Assume a fifteenth.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

- **8d-ii** — it re-runs after this. Its conversion is already proven (64/64, byte-identical
  dry-run); only the load fails.
- **8d-iii**, the residue (49), and `fn` param annotations (7,028 sites, correctly still `<-`).
