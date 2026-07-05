# BRIEF — S-mem.gate: the baked MemStore round-trip functional proof

> **Executor: one sonnet SHADOWDANCER.** The orchestrator walked the room and PROVED the construction + round-trip
> already (a disconfirming probe returns the right page). Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first;
> anchor git with `git -C /home/watmin/work/holon/wat-rs`; `.claude/worktrees/` is illegal). Dogfood `cargo wat <file>`;
> test `cargo nextest run --release` (NEVER `cargo test`). **Commit NOTHING** — the orchestrator weighs + commits.

## The work, in one paragraph

Build the **functional proof** that the baked `:wat::query::MemStore` (a real `:wat::service::defservice`-backed
`Store`/`ReadStore` satisfier) round-trips **put → scan → keyset-paginate → scan-index**. It is a co-located
`deftest'` fixture + a `.rs` harness that loads and runs it, mirroring the S0 gate exactly. S0 used an in-file STUB
that returns empty pages; this uses the REAL MemStore and asserts real data flows through it.

## The exemplars to mirror (read both first)

1. **`tests/rete/probe_arc278_query_contract.{wat,rs}`** — the S0 gate. COPY its shape: the `.wat` is a
   `(:wat::test::deftest' :user::<name> () <body>)` with `(:wat::test::assert-eq actual expected)` calls and
   `(:wat::core::Result/expect res "msg")` unwraps; the `.rs` is a tiny harness that loads the fixture and runs the
   deftest'. Your new pair (`tests/rete/probe_arc278_smem_roundtrip.{wat,rs}`) mirrors it 1:1 in structure.
2. **`tests/services/probe_arc209_c3_defservice_client_face.wat`** — the defservice construction pattern:
   `start :locus (thread) :record (Record …)` → `connect' (Handle/addr h)` → use the client peer; `h` stays bound
   for the whole `let` (the service lives until scope exit).
3. **`wat/query/mem.wat`** — the MemStore itself: the `mem-store'` defservice (ops EnsureSchema/Put/Scan/ScanIndex),
   the `MemStore` struct wrapping `peer`, the `extend-type` to `Store` + `derive` to `ReadStore`. Its header carries
   the load-bearing scope note: **start + connect' + every call must share one lexical scope.**
4. **`wat/query.wat`** — the contract records (StoredRow/Row/IndexRow/ScanRequest/IndexScanRequest/Page/IndexPage/
   TableSchema/IndexSchema/IndexKey) and the Store/ReadStore method sigs. All methods return
   `:wat::core::Result<T,:wat::query::Error>`.

## The PROVEN construction (orchestrator's disconfirming probe — copy this shape verbatim; it type-checks + runs)

```wat
(:wat::core::let
  [h        (:wat::query::mem-store'/start :locus (:wat::spawn::thread)
              :record (:wat::query::mem-store'::Record (:wat::core::PersistentVector)))   ;; empty PV — see gotcha #1
   c        (:wat::kernel::connect' (:wat::query::mem-store'::Handle/addr h))
   store    (:wat::query::MemStore c)
   empty-ik (:wat::core::HashMap :wat::core::String :wat::query::IndexKey)
   rows     (:wat::core::Vector :wat::query::StoredRow
              (:wat::query::StoredRow "u#1" "a" "{:v 1}" empty-ik)
              (:wat::query::StoredRow "u#1" "b" "{:v 2}" empty-ik)
              (:wat::query::StoredRow "u#1" "c" "{:v 3}" empty-ik))
   _es      (:wat::core::Result/expect
              (:wat::query::Store/ensure-schema store (:wat::query::TableSchema "pk" "sk")
                (:wat::core::Vector :wat::query::IndexSchema)) "ensure-schema failed")   ;; empty index Vector OK
   _p       (:wat::core::Result/expect (:wat::query::Store/put store rows) "put failed")
   page     (:wat::core::Result/expect
              (:wat::query::Store/scan store (:wat::query::ScanRequest "u#1" "a" "z" 2 :wat::core::None)) "scan failed")
   pg-rows  (:wat::query::Page/rows page)]
  ...)   ;; this returns a Page whose rows count is 2, first Row/sk is "a" — VERIFIED by the orchestrator
```

Surface dispatch is `(:wat::query::Store/<method> store …)` and `(:wat::query::ReadStore/scan store …)`. `h` MUST
stay bound for the whole `let` (the scope note). `connect'` is `:wat::kernel::connect'`.

## The two form gotchas (the orchestrator hit these; do NOT re-derive)

1. **Empty typed PersistentVector is bare `(:wat::core::PersistentVector)`** — NOT
   `(:wat::core::PersistentVector :wat::query::StoredRow)` (that reads the type-name as a *constructor-fn element*,
   `PersistentVector<Fn(...)->StoredRow>` — a TypeMismatch). The bare form unifies to the expected element type.
2. **`(:wat::core::first v)` returns the ELEMENT, not an `Option`** (arc-278 R13 — "first is not an option"). Do NOT
   wrap it in `Option/expect`. Use `(:wat::query::Row/sk (:wat::core::first pg-rows))` directly.

## What the gate must assert (the full round-trip — extend the proven shape)

Put a table of 5 rows on one `pk` (`"u#1"`), sk `"a".."e"`, and give at least 2 of them a projected GSI index-key
(build the `index-keys` HashMap as `(:wat::core::HashMap :wat::core::String :wat::query::IndexKey "by-v"
(:wat::query::IndexKey "u#1" "<isk>"))`), then assert:

1. **ensure-schema** returns `Ok` (the idempotent no-op) — pass `TableSchema "pk" "sk"` + a Vector with one
   `IndexSchema "pk" "sk" "ipk" "isk"`.
2. **scan page 1** (limit 2, cursor None): 2 rows, `Row/sk` in `["a" "b"]` ASC, `Page/next-cursor` = `Some "b"`.
3. **scan page 2** (limit 2, cursor `Some "b"`): 2 rows `["c" "d"]`, next-cursor `Some "d"`.
4. **scan page 3** (limit 2, cursor `Some "d"`): 1 row `["e"]`, next-cursor `None` (keyset exhausted — `full?` false).
5. **scan-index** on `"by-v"` (an `IndexScanRequest index ipk isk-lo isk-hi limit cursor`): the `IndexPage` returns
   exactly the rows that projected that GSI, ordered by `isk` ASC. Assert row count + first `IndexRow/isk`.

Use `(:wat::test::assert-eq <actual> <expected>)`. To assert a cursor: `(:wat::test::assert-eq (:wat::query::Page/next-cursor page) (:wat::core::Some "b"))`. To count: `(:wat::core::count (:wat::query::Page/rows page))`.

## STOP triggers (halt + report, do not improvise)

1. **STOP-NO-STUB:** the gate MUST drive the REAL baked `:wat::query::MemStore` (start/connect'/MemStore). Do NOT
   fall back to an in-file stub satisfier (that is S0, already done — it proves nothing new).
2. **STOP-NO-MEM-EDIT:** do NOT modify `wat/query/mem.wat` or `wat/query.wat` — they are baked + green. If the round-trip
   reveals a real MemStore bug (e.g. pagination off-by-one), STOP and report it (it's a mem.wat finding, not a test to
   bend).
3. **STOP-SCOPE:** if construction fails with "channel disconnected"/dead-peer, the `let` scope broke (the peer/`h`
   went out of scope before a call) — keep everything in ONE `let`, `h` bound first; do not factor construction into a
   helper fn (the scope note forbids it).

## The gate (EXPECTATIONS)

| what | command | expected |
|---|---|---|
| the new round-trip deftest' passes | `cargo nextest run --release -E 'test(smem_roundtrip)'` | passed |
| S0 still green | `cargo nextest run --release -E 'test(query_contract)'` | passed |
| whole floor | `cargo nextest run --release` | report the Summary line VERBATIM; 0 failed modulo the known `no_inlined_wat_in_tests` reminder |

## Your final report MUST contain

1. The two new files (paths) + the deftest' body.
2. The verbatim `smem_roundtrip` + `query_contract` results and the whole-floor Summary line.
3. Any STOP trigger hit (esp. STOP-NO-MEM-EDIT — a real MemStore bug found), or "no STOP triggers hit".
Your final message IS the return value — raw facts, no ceremony.
