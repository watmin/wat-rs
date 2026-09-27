# WEIGH — STONE 255.50: O1, the tuple binder, and what reaches the classes — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `708cd1c5c`** (the SCORE only; the tree was clean, and grok removed its
worktree). Weighed by the orchestrator on 2026-09-26.

## Re-run by the orchestrator (`wat --check` / run, session scratchpad)

| probe | result |
|---|---|
| a bare `[t p]` (Thread, Process) to a `(Vector :- [(Spawned :- [S R])])` parameter | rc 0: the literal takes `Spawned` from the parameter |
| `(Vector :- [(Thread …)] t t)` to the same parameter | rc 1: *"expects (Vector :- [(Spawned :- [_ _])]); got (Vector :- [(Thread :- [i64 i64])])"* |
| a `defstruct` holding `[i64 :-> i64]`, compared with `=` | check passes; run rc 1: *"expected matching comparable pair, got wat::core::Struct `:probe::Box{#0: <fn>}`"* |
| `peel_type_binder` drops every non-symbol binder entry without an error | read: `src/function/metadata.rs:52-62` (`filter_map`, `_ => None`) |
| `derive` is "the edge-only half of `extend-type`" | `src/intrinsic/special/derive_form.rs` header |

## The reading

1. **O1 holds.** Every `poll` vector (177) is already a `Peer` vector. For `select`, the one stdlib owner vector is
   already `Spawned` (`wat/bracket.wat:603`). Four test literals up-cast from the parameter. **One site needs an
   annotation:** `tests/comms/probe_select_flood_no_deadlock.wat:34` builds a `Process` vector with the
   constructor. No `mapv` pool reaches `select` today. A future one would need its element type declared.
2. **The tuple rest binder does not exist in any spelling.** `&` and `...` are ordinary type-parameter names, and
   `:<` is refused as an angle bracket in type position.
3. **Two defects surfaced that no ruling is needed for:**
   - **A bound written today silently vanishes.** `peel_type_binder` drops any binder entry that is not a bare
     symbol, with no error, so `(defn f :- [[T :< X]] …)` declares nothing and says nothing. This must refuse
     before `:<` means anything.
   - **Structs holding functions break `=` at run time.** A struct holding a function passes `=` at check time and
     raises at run time. No corpus site does it (the four real function fields, in `Gen`, `ThreadOpts` and
     `ProcessOpts`, are never compared), but the class admits it. A record cannot hold a function (the purity
     gate refuses it).
4. **Newtypes:** 9, all in tests, none compared. Ordering refuses them today; equality recurses into the inner type.
5. **Two spellings of one act:** a bodiless membership edge is written both as `derive` (11 corpus uses) and as a
   bodiless `extend-type` (`wat/spawn.wat:261`, the 255.48 declarations). That breaks the "exactly one way" ruling.

## Decisions for the builder

- O1, with its one annotation.
- Which aggregates are `Equatable`: pure ones only (the `Record` root), or structs too (and then how a struct's
  function field is excluded).
- `derive` or bodiless `extend-type` as the one spelling of an edge.
- The tuple rest binder's spelling. None exists; `&` is the value-level precedent.

## Correction (2026-09-26, after the builder asked what D1 breaks)

Reading 5 ("two spellings of one act") was wrong. `derive` registers a hierarchy between **names** (bare
`register_subtype`, no binder, no checks; parents such as `:t::Marker` are declared nowhere, and `wat/service.wat`
derives an enum from a service-op keyword). `extend-type` registers **surface membership** with the binder and its
checks. They are Clojure's `derive` (`isa?`) and `extend-type` (protocols), two acts. D1 is withdrawn. Q1 is
ruled: only pure data is `Equatable`/`Orderable`.
