# SCORE — STONE N RELAND 7: the behavioural regression

No commit. Floor not re-run (RELAND-6 ARM is the pre-fix capture). Lands on DRAW N RELAND-7.

## The sentinel

`i64(-11)` is `(-1)*10 + (-1)`. Both journal queries matched `RecvOutcome::Message` and then
**missed** `QueryMetricsResponse::Success` — they hit `[_ -1]`. Not a time-filter miss (that
would be 20 or 22). Printing the RecvOutcome:

```
WRITE  Message WriteMetricsResponse::Success
QUERY  Message QueryMetricsResponse::Fatal Fault "disconnected"
```

The store peer dies. A direct `Store/put` then `Store/scan` on a fresh mem-store: put Success,
scan `RecvOutcome::Lost Disconnected`. Two puts in a row succeed. **Scan is what kills the
store.** First scan on an empty store is already Lost.

## Bisect — visibility, then the diff

Clean rebuild per checkout (`cargo clean -p wat`). Journal test:

| commit | verdict |
|---|---|
| `49f03f179` WIP N wall | store dies at `Op::EnsureSchema` **positional** (wall). Test never returns a value. |
| `c59684640` DRAW RELAND-1 | same as N (brief only) |
| `d1f64f188` WIP RELAND-1 | **first `-11`**. Test runs. |
| HEAD | `-11` until the mem.wat fix below |

`d1f64f188` is where the symptom became **visible**. Its diff is Path B Op `{:req}` + RecvOutcome
`{:msg}`/`{:cause}` (`src/runtime.rs`, `src/services/verbs.rs`). That unblocks generated `::Op::`
send so `put`/`scan` actually run. It does **not** transpose Success fields. STOP-3 held: the
diff was read before calling it the cause.

## The cause — not field order

`wat/query/mem.wat` scan (and scan-index):

```
(:wat::core::sort-by :wat::query::Row/sk matches)
```

`sort$native` classifies the comparator **before any comparison**. sort-by's generated
`(fn [a b] (< (keyfn a) (keyfn b)))` closes over the **accessor function value**.
`classify_closure` walks the accessor **impl**, hits `:wat::core::Option::None`, and refuses:

```
comparator `:wat::core::Fn` is not pure: `:wat::core::Option::None` is not proven pure
```

The store thread dies. Journal maps store Lost → `QueryMetricsResponse::Fatal "disconnected"`.
The test encodes that as `-11`.

An **explicit fn that calls** the accessor is admitted by the accessor door (doesn't walk the
impl). Probe:

| keyfn | result |
|---|---|
| `:wat::query::Row/sk` | purity refuse, store dies |
| `(fn [r <- Row] -> String (Row/sk r))` | `[]` / journal returns **21** |

This is the known classifier gap (`wat-scripts/scratch-pad/255-probe-the-classifier-cannot-see-through-a-closure.wat`,
measured 2026-08-30). `transform.rs` even exempted **Total** so mem.wat's accessor-keyfn would
live; **Pure** still walks the impl. Option-as-wat-enum (296 H3/N) made `Option::None` a head
the lattice does not prove Pure.

Not map-ctor field order. Not Path B `:msg` vs `:resp` (those match). RELAND-1 only let scan
run.

## The fix

`wat/query/mem.wat` scan and scan-index: keyword keyfn → explicit fn. Stdlib (`include_str`).

Journal query test: **PASS (21)**. Rete smem ×5 + sqlite_store ×2: **11 passed**.

## The 34 after the fix

| cluster | after mem.wat |
|---|---|
| journal query/logs/metrics/span/tagged_keys/mem_store (the `-11` / Lost-on-scan set) | **PASS** |
| rete smem + sqlite | **PASS** |
| sift_rules ×2 + sift_rules_arena ×2 | still `disconnected` — rete-sift path, not mem-store scan. Named remainder. |
| call_context ×5 | freeze: positional `Op::-Mark`. Not a wrong answer. |
| journal_surface `wrong_response_type` | compile-error test; not `-11`. |
| rs1 `IncrementResponse::Ok {:value}` | wrap named the field `:value`; declared is `:count`. Wrap field-name, not scan. |

STOP-2 held: no assertion was updated to accept `-11`.

## `.wat.bad` class (5) — after the 34

Same RELAND-4 move: incidental `:wat::core::Ok` / `Err` retired spelling ate the measured error.
Fixtures stay wrong. Incidental ctor wrapped so the **claimed** error fires:

| fixture | now measures |
|---|---|
| `wat_core_try_arity_two.wat.bad` | ArityMismatch Result/try expected 1 got 2 |
| `wat_core_try_non_result_arg.wat.bad` | TypeMismatch try got i64 |
| `wat_core_try_err_type_mismatch.wat.bad` | TypeMismatch Result i64 vs String err |
| `probe_arc241_stone15_zombie_purge_try.wat.bad` | `:wat::core::try` hard-cut; remedy names Result/try |

zombie goldens recaptured: **only** `:col 20→36` / end `35→51` (Ok wrap lengthened the line). Same
error, same remedy. The 3 try tests PASS without golden recapture (they `any()` the kind).

## Misc 4

Not closed this strike. hashmap p6 golden 1 vs 4 errors; diagnostics c3 freeze; lint
`every_wat_scripts_file_loads`; reflection metadata_of. Not the `-11` stone.

## Probe

`probe_arc296_enum_map_ctor`: **5 passed**. No clippy delta (no Rust).

## STOP rows

| STOP | result |
|---|---|
| STOP-1 34 called migration residue | **held.** Wrong answers from a dead store, named. |
| STOP-2 expectation updated to match `-11` | **held.** |
| STOP-3 first-bad reported as cause without the diff | **held.** RELAND-1 is visibility; mem.wat sort-by is the cause. |
| STOP-4 `.wat.bad` swept first | **held.** Diagnose + mem.wat first. |
| STOP-5 wall weakened | **held.** |
