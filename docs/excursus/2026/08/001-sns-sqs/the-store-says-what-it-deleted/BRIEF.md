# BRIEF — the store says what it deleted

**Read `DESIGN.md` beside this first.** It carries the contract decision (the field is `deleted` and it
counts BASE-TABLE rows only), the finding that sizes the work (the count already exists — it is
discarded three times), and the proof row a green floor cannot give you.

## The work, in one paragraph

`:wat::query::Store::DeleteResponse::Success` is nullary, so a caller cannot know how many rows a delete
removed and must estimate — which drives a queue's maintained row count to **−40**. The number already
exists at every layer and is thrown away three times. Give the variant a `deleted <- i64`, stop
discarding the count in both store implementations, migrate every match arm in the corpus with a
**recorded fix-wat codemod**, and prove the number is real with a differential probe that reads it.

## The rooms, in the order to walk them

| where | why you are going there |
|---|---|
| `wat/query.wat:534` | the declaration. `:Success []` → `:Success [deleted <- :wat::core::i64]`. ⭑ Read `:549` and `:565` first — `ScanResponse::Success [rows next-cursor]` is this file's own precedent for an informative success. |
| `wat/query.wat:53–55` | the header states that precedent as the design. Your change is the file's own pattern, applied to `delete`. |
| `wat/sqlite.wat:114` | ⭑ **THE PROVEN SHAPE.** `:wat::sqlite::execute` already returns `(Result :- [i64 Error])` and already binds `((Ok n) (Ok n))`. The count is in your hand before you write a line. |
| `wat/query/sqlite-store.wat:336–346` | `delete-one-key`. `((Ok _) (clear-index-projections …))` — discard site #1. Widen to `-> (Result :- [i64 Error])`, keep the base statement's `n`, and **return `n` regardless of what the GSI clear removes**. |
| `wat/query/sqlite-store.wat:348–356` | `delete-rows`. Discard site #2, and the recursion must **accumulate** — `n + (delete-rows … rest)`, not the last key's count. |
| `wat/query/sqlite-store.wat:88–99` | `delete-response`. Its `((Ok _) (…::Success))` becomes `((Ok n) (…::Success n))`. Its parameter type widens with `delete-rows`. |
| `wat/query/sqlite-store.wat:418–433` | the `delete` impl that calls `delete-rows` and hands the result to `delete-response`. Check the threading still type-checks end to end. |
| `wat/query/mem.wat:658–677` | mem's `delete`. The fold builds a `MemWrite`; `length` of `mem-store::Record/rows dur` minus `length` of `MemWrite/rows written` is `deleted`, free, no read. |
| `wat/fix.wat:23–52` | the BOOTSTRAP header. **Read it, then note that it does not apply** — you make no Rust change, so its own escape clause holds: *"No Rust change in your strike? Skip the stash — just `cargo build --release` once."* |
| `wat-scripts/fixes/add-event-id-to-metric-log-ctors.wat` | ⭑ **COPY THIS SHAPE** for your codemod: insert a token inside a matching list, idempotent, comment-faithful via `fix-text-apply` span splices, a non-matching head never touched. |
| `wat/fix.wat:1012`, `:1025` | `arm-head-name` / `arm-heads-contain?` — the existing verbs for reasoning about match-arm heads, which is how you separate an arm from a construction. |
| `wat-scripts/queue/sqs.wat:1110`, `:1614` | the two real consumers. Arms only for this stone — bind `_`. The stone that *uses* the number is the next one. |

## The codemod, and its rule is structural

`(…::Success)` is the identical token in a construction and in an arm head, and
`wat-scripts/scratch-pad/probe-entry-three-of-ten.wat` has **both** (`:137` construction, `:143`/`:243`
arms). So a per-file allow-list cannot do this. The rule:

> `(…::DeleteResponse::Success)` → `(…::DeleteResponse::Success _)` **only** where that list is the
> FIRST child of a match-arm list. Never in value position. Idempotent.

**The three construction sites are yours to hand-edit as logic** (`mem.wat:677`,
`sqlite-store.wat:91`, and `probe-entry-three-of-ten.wat:137` — that last one is a probe's hand-built
reply; give it any honest literal).

Census first, diff it, then apply — and **list EVERY path across all four `.wat` homes**:

```
wat/          wat/query.wat wat/query/mem.wat wat/query/sqlite-store.wat
wat-scripts/  wat-scripts/queue/sqs.wat
              wat-scripts/scratch-pad/probe-entry-three-of-ten.wat
              wat-scripts/scratch-pad/probe-no-orphan-gsi-rows-after-delete.wat
tests/        tests/rete/probe_ex001_delete_differential.wat
              tests/rete/probe_ex001_store_delete.wat
docs/         docs/excursus/2026/08/001-sns-sqs/PROBE-reput-divergence.wat
              docs/excursus/2026/08/001-sns-sqs/PROBE-store-has-no-delete.wat
```

⚠ **Generate the list, do not hand-type it.** `tests/**/*.wat` left out of a path list reddened this
campaign's floor with **217** failures. Commit the codemod as the recorded migration.

## Blast radius, measured

```
DeleteResponse          40 occurrences / 11 files   ·   .rs 0   ·   .jsonl 0
DeleteResponse::Success 12 occurrences / 9 files    ·   3 constructions + 9 match arms
```

⭑ Count with `grep -o … | wc -l`, never `grep -c` (it counts LINES and reported 1 where there were 11
earlier in this campaign). And note the broader token `DeleteResponse` is the one that finds the
declaration — `DeleteResponse::Success` does **not** appear at `wat/query.wat:534`.

## ⭑⭑ THE PROOF — a green floor is silence for this stone, so build the instrument

Every arm binds `_`, so **nothing reads the number** and the floor is green whether the count is right,
always-1, or base+GSI. Write a probe under `wat-scripts/scratch-pad/` that reads it:

> delete a batch of **3 keys of which only 2 exist**, and print `deleted`. Expect **2**. Run it against
> **mem AND sqlite** — the two backends compute the number by completely different routes (length delta
> vs `stmt.execute()`), and stone 2b established that a differential is how this store is proven.

`deleted = 3` → missing keys counted. `deleted = 2 + 2·|indexes|` on sqlite → the GSI clear was summed.
Either way the one number names the defect.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run` — the build does NOT compile
  tests, and that gap reddened this floor once already.
- `./scripts/floor.sh`, and read the **Summary line**. ⛔ Never a piped exit code: this session a
  type-error run reported `$?` = **0** through a `| head` and its true exit was **3**.
- Happy path: `./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000` →
  `distinct=8000;dup=0`.
- Chaos still green: `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → exit 0.
- `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- Run everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — if the base `DELETE`'s count cannot be separated from the GSI clear's**, STOP and report
   the shape. `deleted` means base-table rows; a number that includes index projections is worse than
   no number, because it looks plausible.
2. **STOP-2 — if `deleted` cannot be made exact for some delete path**, STOP and name the path. Do
   **not** invent an estimate, and do **not** add an `:Unknown` variant on your own authority — SCORE
   §7.3 of `the-queue-knows-its-own-depth` says such a state is needed for the unknowable-*put* path,
   and that is a builder decision, not this stone's.
3. **STOP-3 — if a floor test asserts on `DeleteResponse::Success`'s arity or rendered form**, STOP and
   report it. Do not patch a golden to fit; a prior stone reverted `src/freeze.rs` and another restored
   `wat/core.wat`'s exact line count rather than patch one. Copy that.
4. **STOP-4 — `PutResponse::Success` and `EnsureSchemaResponse::Success` are OUT.** If you find
   yourself widening either, stop: they are rejected with a reason in DESIGN §Out of scope.

## Shape to copy

`the-gate-outcome-outlives-its-file/SCORE.md` — the last stone that changed a stdlib type's shape and
proved it with a probe because the floor could not. And
`the-queue-knows-its-own-depth/SCORE.md` §11 — the measurement that owed this ruling.
