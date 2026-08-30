# BRIEF — excursus 001 stone 2: `:wat::query::Store` gains `delete`

**Builder's ruling 2026-08-30: option (a).** Add `delete` to the Store surface. Not tombstones.

## The work, in one paragraph

`:wat::query::Store` today declares four features — `ensure-schema`, `put`, `scan`,
`scan-index` — so it can append and read but never remove. SQS's `ack` must remove a message,
so it is not expressible. Add a fifth feature, `delete`, that takes a batch of `(pk, sk)` keys
and removes exactly those rows atomically, mirroring `put` in every respect: batch-shaped,
one transaction, and the shared `:Success/:Constraint/:Transient/:Fatal/:RequestTooLarge/
:RequestMalformed` response vocabulary. Then satisfy it in both backends.

## Read in order — the rooms, and why you are being sent there

1. **`wat/query.wat:30`** — `StoredRow`. Its `pk` and `sk` are both `:wat::core::String`.
   **`:wat::query::Key` does NOT exist yet** — you are minting it, and it is exactly those two
   fields. (Verified absent 2026-08-30: `grep -n 'Key' wat/query.wat` returns only `IndexKey`
   and prose.)
2. **`wat/query.wat:510–520`** — `Store::PutRequest` / `PutResponse`. This is the shape to
   mirror. `PutRequest` is a single field `rows <- (Vector :- [StoredRow])`; `PutResponse` is a
   `:wat::enum::Pure` whose first arm is `:Success` and whose remaining arms are the shared
   vocabulary. `DeleteRequest` is `keys <- (Vector :- [Key])`; `DeleteResponse` is
   `PutResponse`'s arm list, unchanged.
3. **`wat/query.wat:551–569`** — the `:features` block. Add `delete` after `put`. Copy `put`'s
   `:max-request-bytes 10485760` — a delete batch is key-sized, not row-sized, but the store
   is the bulk-write backend and asymmetry here would be a second number to explain.
4. **`wat/query/mem.wat:107`** — `mem-store`'s `put`. It `foldl`s the batch onto
   `Record/rows` with `:wat::vector::conj` and replies with a NEW `State`. `delete` is the
   same shape with the fold inverted: keep the rows whose `(pk,sk)` is not in the key set.
   Note it must mutate durable state visible to a LATER, SEPARATE `scan` — that is the whole
   reason this is a `defservice` actor and not a mutable cell (see that file's header, lines 9–24).
5. **`wat/query/sqlite-store.wat:224`** (`put-rows`) and **`:276`** (the `put` impl) —
   `put` is `begin → put-rows → commit`, chained through `Result` with an `Err` short-circuit,
   and the reply goes through `put-response` (`:51`). Write `delete-rows` beside `put-rows`
   with the identical recursive shape, and reuse `put-response` if its type admits the new
   response; if it does not, mint `delete-response` beside it rather than widening `put-response`.

## The acceptance gate — it already exists and it is RED

**`docs/excursus/2026/08/001-sns-sqs/PROBE-store-has-no-delete.wat`.**

Run it at HEAD and it fails with exactly two errors, the second a cascade of the first:

```
unknown callee: :wat::query::Store/delete
malformed match: keyword variant pattern Store::DeleteResponse::Success on a :?N scrutinee
```

Everything else in that file — `mem-store/start`, `connect`, `ensure-schema`, `put`, `scan` —
is copied from the green `tests/rete/probe_arc278_smem_roundtrip.wat` and type-checks clean.
**The stone is done when that probe is GREEN with no edit to the probe.** It puts 3 rows,
deletes the middle one by `(pk,sk)`, and asserts the scan count goes 3 → 2.

The probe lives in the arc dir, not `wat-scripts/`, because
`every_wat_scripts_file_loads` type-checks everything under `wat-scripts/` and a RED-by-design
probe would take that gate red. **When the stone lands, promote it** — into
`tests/rete/` beside the roundtrip it was copied from, with a `.rs` harness, so it becomes a
permanent floor test rather than an arc artifact.

## Implementation sketch — you fill it, do not invent the shape

```wat
;; wat/query.wat, beside StoredRow
(:wat::core::defrecord :wat::query::Key [pk <- :wat::core::String  sk <- :wat::core::String])

;; wat/query.wat, in Store's :messages, beside PutRequest/PutResponse
(:wat::core::defrecord :wat::query::Store::DeleteRequest
  [keys <- (:wat::core::Vector :- [:wat::query::Key])])
(:wat::core::defenum :wat::query::Store::DeleteResponse :wat::enum::Pure
  :Success [] :Constraint [...] :Transient [...] :Fatal [...]
  :RequestTooLarge [...] :RequestMalformed [...])   ;; PutResponse's arms, verbatim

;; wat/query.wat, in :features, after put
(delete [self <- :wat::query::Store  req <- :wat::query::Store::DeleteRequest]
  -> :wat::query::Store::DeleteResponse :max-request-bytes 10485760)
```

## Blast radius

`wat/query.wat`, `wat/query/mem.wat`, `wat/query/sqlite-store.wat`. **No new dependencies, no
change to `put`/`scan`/`scan-index`/`ensure-schema`, no change to any consumer.**
`:wat::telemetry::journal` holds a Store peer and must keep working untouched — it is the
regression canary, and `tests/services/probe_arc278_journal_backend_differential.wat` already
runs it against both backends.

## STOP triggers

1. **If `delete` cannot reuse `PutResponse`'s arm list** — because the shared error vocabulary
   does not admit it — STOP and report which arm does not fit. Do not invent a parallel error
   vocabulary; `wat/query.wat:484`'s note ("derive-is-the-wall, NOT a parallel telemetry one")
   says why that is the wrong move.
2. **If deleting a row requires touching its GSI projections** and the shape for that is not
   obvious from `put-rows` — STOP. `StoredRow` carries `index-keys`, but a `Key` does not, so
   the backend may need to read before it deletes. That is a real design question about whether
   `Key` is sufficient, and it is the builder's, not yours.
3. **If the floor goes red on anything outside the three files above** — STOP, capture the arm
   whole, do not re-run. There is no such thing as a known flake.
4. **If you find yourself editing the probe** — STOP. The probe is the gate. If the gate is
   wrong, say so and why; do not move it.

## Prior comparable result to copy for shape

`docs/arc/2026/06/255-builtin-registry/SCORE-STONE-the-type-registry-holds-the-builtin-types.md`
— an arc-255 stone that added to a registry surface and reported honest deltas.

## Verify like this — never through a pipe

```bash
./target/release/wat --check docs/excursus/2026/08/001-sns-sqs/PROBE-store-has-no-delete.wat
echo "CHECK=$?"                  # 0 when the stone lands
./scripts/floor.sh; echo "FLOOR=$?"
```

`cmd | tail` returns `tail`'s status, and `grep -c` exits 1 on zero matches. Read the Summary
line, never a piped exit code. This exact mistake shipped a "green" report on a 41-failure
floor in this repo on 2026-08-30.
