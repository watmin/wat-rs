# SCORE — a batch declares how many

**SCORED.** Executor: grok, 2026-09-06. Tree dirty, uncommitted.
The intrinsic is gone. The cap is two defs.

```
Summary [ 409.417s] 5220 tests run: 5220 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T07-06-03Z/`

## THE SURFACE

`:max-entries [bodies 10]` on `Queue::send` only. `SendResponse` gained
`RequestTooManyEntries [entries cap]` beside `RequestTooLarge`. Absent option
emits neither def and no guard. There is no default.

Two defs from `build_op_budget_constants`, beside the byte-cap def, only when
declared:

```
:queue::Queue::SEND-MAX-ENTRIES        10
:queue::Queue::SEND-MAX-ENTRIES-FIELD  "bodies"
```

`defservice` interpolates those names the same way it interpolates
`SEND-MAX-REQUEST-BYTES`. The wrap sits **before** the byte guard, not behind
`peer-wire?`.

## THE MECHANISM THAT WAS DELETED

`:wat::types::max-entries` was a `thread_local` HashMap written at defsurface
expand and read at defservice expand. Four-no: wrong namespace, never cleared
(stale across programs in `wat --mcp`), not a value a caller can ask for, and
a walk-stash-retrieve against name→value.

Deleted: the intrinsic, `stash_max_entries_from_defsurface`, the
`MAX_ENTRIES` thread_local, the `:wat::types::` registration. Expand-time
wrap-or-not walks the satisfied surface's `:features` (the form is attached as
`:$surface-form` by `expand.rs` from a per-freeze map on `MacroRegistry` — not
a process-lifetime cell). The numeric cap is looked up at runtime from the
def.

STOP-7 did not fire: `WatAST::StringLit` is a legal `def` body. The field
name rides next to the i64.

## PATH B

`src/runtime.rs` already had the byte-guard twin for `:queue::Queue/send`.
The entries wrap is there too, using `member.max_entries` (SurfaceDef is
available). BRIEF listed the two `wat/service.wat` sites; without Path B a
`Queue/send` call would only reject server-side. Noted, not a second surface.

## FIRST FLOOR — RED, captured, not re-run

`.floor/2026-09-06T06-49-20Z/`

```
Summary [ 408.400s] 5220 tests run: 5214 passed (6 slow), 6 failed, 22 skipped
```

| arm | what fired | fix |
|---|---|---|
| `no_loose_string_assert` | `reason.contains(...)` in the three declaration-time tests | `assert_eq!` on the whole reason |
| `every_dispatched_verb_is_classified_or_disposed` | Path B wrap spelled `":wat::core::count"` inside the dispatch scan | classified `count` in `intrinsic_meta` (pure, deterministic, not total) |
| four `peers_bijection_*` | helper inserted **before** the bijection walls; goldens pin `service.wat:896` / `:913` | helper moved **after** `_peers-extra`. Goldens untouched |

## THE PROBE

`wat-scripts/scratch-pad/probe-a-batch-declares-how-many.wat`:

```
cap=10;field=bodies;thread=RequestTooManyEntries(11,10);depth 0->0;Ok;depth 0->10;process=RequestTooManyEntries(11,10);depth 0->0;Ok;depth 0->10
```

11 rejected, depth unchanged, at both loci. 10 is Ok, depth +10. The defs
evaluate from ordinary wat.

## CIRCUIT ×5

`ps` before: python3 28.5 %, grok 13.9 %, claude 2.3 %, else < 1 %.

Every run: `total=8000;distinct=8000;dup=0`.

```
publish  23778  23760  23840  23816  23776     median 23778
drain      227    210    245    204    183     median   210
```

Prior 934-ms SCORE publish median **23782**. Within noise. The circuit sends
≤ 4 bodies; the guard never fires.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ cap rejects, nothing enqueued | ✅ `RequestTooManyEntries(11,10)`, depth 0→0 |
| 2 | ★★ both loci | ✅ thread and process identical |
| 3 | ★★ 10 still passes | ✅ Ok, depth 0→10 |
| 4 | ⛔ option without variant is a compile error | ✅ `max_entries_without_request_too_many_entries_is_a_located_error` names `RequestTooManyEntries` |
| 5 | ⛔ absent means uncapped | ✅ `BAR-MAX-ENTRIES` not emitted; ack/receive/stats unwrapped |
| 6 | ⛔ both emission sites | ✅ `wat/service.wat` serve-op-arms and op-methods; Path B in `runtime.rs` |
| 7 | ⛔ unknown-option lists the name | ✅ surface.rs: `` `:max-request-bytes`, `:max-entries` `` |
| 8 | ⛔ one adopter | ✅ `sqs.wat` `send` only. Macro + probe mention the name; no Store/ack/Seen |
| 9 | ⛔ floor | ✅ `5220 passed (6 slow), 22 skipped`. Count reported |
| 10 | ⛔ delivery exact | ✅ ×5 `total=8000;distinct=8000;dup=0` |
| 11 | blast | ▪ also `runtime.rs` (Path B), `types.rs` (defs + walls), `expand.rs`/`registry.rs` (no-stash attach), `rete/purity.rs` (count). SCORE. |
| 12 | ⛔ missing field is declaration-time | ✅ names `bodys`, lists `queue, bodies, now-ns` |
| 13 | ⛔ HashSet/HashMap is declaration-time | ✅ names `(HashSet :- [String])`, says ELEMENTS vs distinct members |
| 14 | ⛔ the cap is a readable def | ✅ `:queue::Queue::SEND-MAX-ENTRIES` → `10` |
| 15 | ⛔ no expand-time mutable state in surface.rs | ✅ `grep -n "thread_local\|RefCell" src/types/surface.rs` empty |
| 16 | ⛔ no `:wat::types::` intrinsic | ✅ `grep -rn 'wat_intrinsic(":wat::types::' src/` empty |

STOP-1 did not fire: the option value is a vector, parsed per-key.
STOP-2 did not fire: field accessor is interpolated at expand time from the
declared name, same shape as `rtl-ctor-kw`.
STOP-6 did not fire: the sequence wall lives in `synthesize_surface_protocol`
after `request_ty` is extracted (parser has no TypeEnv).
STOP-7 did not fire: StringLit is a legal def body.

## REPORTS

| ▪ | what |
|---|---|
| a | publish median 23778 (prior 23782) |
| b | unmeasured separately; circuit within noise |
| c | dup=0 every run |
