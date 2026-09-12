# DESIGN — the store says what it deleted

Builder: *"so do it?"* — opening owed-ruling **#2**, and overruling my hesitation about reaching into
`wat/` with the branch's own charter: *"we have done massive break fixes all over wat in this branch....
if its broken when we find it, then we fix it, this branch exists to find problems from forcing ourselves
through our own UX in anger."*

**Drawn 2026-09-12. NOT STRUCK.**

## Why

`the-queue-knows-its-own-depth/SCORE.md` §11 ends on one sentence: *"`Store::DeleteResponse::Success` is
nullary. That single missing `i64` is why a queue cannot know its own depth by addition."* Its STOP-2
fired on a measured number — the maintained row count reaches **−40 on three of five queues** in the
standard run with zero injected faults — and the ruling it owed is this stone.

⭑ **I reproduced the refutation on my own run this session**, before drawing anything:

```
tier=sub[1] accepted=2000 acks=2040   tier=sub[2] accepted=2000 acks=2040
tier=sub[3] accepted=2000 acks=2040   ack-retries=12
```

`excess acked ids = 10 × ack-retries` → `10 × 12 = 120 = 3 × 40`. Exact, on `9a6ee5b76`.

## ⭑⭑ THE FINDING THAT SIZES THIS STONE — the count is not missing, it is DISCARDED

I expected to have to *produce* a count. It already exists at every layer:

| layer | what it already has | citation |
|---|---|---|
| rusqlite | `stmt.execute()` → rows affected | `src/rust_deps/sqlite.rs:216` |
| Rust shim | `Ok(n as i64)` | `src/rust_deps/sqlite.rs:219` |
| `:rust::sqlite::Connection::execute` | `Result<i64, …>` | `src/rust_deps/sqlite.rs:286` |
| `:wat::sqlite::execute` | `-> (Result :- [i64 Error])`, binds `((Ok n) (Ok n))` | `wat/sqlite.wat:114` |
| ⛔ `:wat::query::delete-one-key` | **`((Ok _) …)` — DISCARDED**, and `-> Result<nil, Error>` cannot carry it | `wat/query/sqlite-store.wat:344` |
| ⛔ `:wat::query::delete-rows` | discards again, same return type | `wat/query/sqlite-store.wat:355` |
| ⛔ `DeleteResponse::Success` | nullary — nowhere to put it | `wat/query.wat:534` |

**mem's count is free too**: `delete` folds `durable-swap-remove` into a `MemWrite`, so
`length(rows before) − length(rows after)` is the answer with no read
(`wat/query/mem.wat:658–677`).

★ **So zero Rust changes.** `execute` already returns the number. This is not a feature; it is three
discard sites and a nullary variant. And per `wat/fix.wat`'s BOOTSTRAP header — *"No Rust change in your
strike? Skip the stash"* — **the stash-dance does not apply.**

## ⭑ And the file already has the precedent, in its own words

`wat/query.wat` has five `:Success` variants. **Two already carry data**, and the file's own header
§"the pages" (`:53–55`) states it as the design: *"`Store::ScanResponse::Success` /
`ScanIndexResponse::Success` carry rows+cursor directly."*

```
:513  EnsureSchemaResponse::Success  []                     ← nullary
:523  PutResponse::Success           []                     ← nullary
:534  DeleteResponse::Success        []                     ← nullary, THIS STONE
:549  ScanResponse::Success          [rows next-cursor]      ← informative
:565  ScanIndexResponse::Success     [rows next-cursor]      ← informative
```

This is not a new idea. It is the file's own pattern, unapplied to `delete`.

## The one contract decision

```
:Success [deleted <- :wat::core::i64]
```

**`deleted` counts BASE-TABLE rows actually removed — never GSI projections.** For a request of `k`
keys, `0 ≤ deleted ≤ k`; a key that was not there contributes 0. This is already both stores' behaviour
in prose — `wat/query/sqlite-store.wat:333` (*"A missing key is DELETE of 0 rows then the same clear
(also 0) — Success"*) and `wat/query/mem.wat:659` (*"Missing key is a no-op"*) — so the stone makes an
existing fact reportable rather than changing any semantics.

★ **Named `deleted`, not `n`.** The SCORE §7 sketched `[n <- i64]`; `n` fails *Obvious?* at the match
site. `((… ::Success deleted) …)` reads; `((… ::Success n) …)` does not.

⛔ **The GSI trap is the one way to get this silently wrong.** `delete-one-key` runs the base `DELETE`
*and then* `clear-index-projections`, which DELETEs from each `index_<name>`. Summing those gives
`1 + |indexes|` per existing key. **Only the base statement's count is `deleted`.**

## The four questions

**Obvious?** YES — the file's own two informative `:Success` variants are the precedent, and `deleted`
names itself. **Simple?** YES — one field, one meaning, three discard sites stop discarding. **Honest?**
YES, and it is the point: a `Success` that cannot say how many rows went is *strictly less informative
than its referent* (DynamoDB's `DeleteItem` reports consumed capacity and can return the old item).
**Good UX?** YES — the caller stops estimating a number the callee knew.

## Scope

**IN:** the variant gains `deleted` · `sqlite-store` threads the base-statement count through
`delete-one-key` → `delete-rows` → `delete-response` · `mem-store` reports its length delta · every
`DeleteResponse::Success` **match arm** in the corpus gains a binder, **via a recorded fix-wat codemod**
· a differential probe that proves the number is real on **both** backends.

## Out of scope = REJECTED

- ⛔ **Landing `PATCH-measure-variant.diff` (the −17.1 % `rt-store`, −60.9 % `count`).** The sequencing
  is forced: the patch reads a maintained counter that is only correct once this stone exists. And
  SCORE §7.3 shows it *also* needs an explicit `:Unknown` state for the §2d unknowable-put path, which
  is a second design decision. **This stone makes the win possible; it does not bank it.** ⚠ Builder:
  that means this stone shows **no `rt-store` improvement**, by construction. Say so in the grading.
- **`PutResponse::Success` and `EnsureSchemaResponse::Success`** — the two sibling nullary successes.
  ⭑ **Asked FIRST this time** rather than found in a floor red. Rejected with a reason, not by
  omission: `put` is clear-then-insert, so a "rows written" count is always `|rows|` and carries no
  information the caller lacks; `ensure-schema` is idempotent DDL. Neither hides a number. **If either
  ever does, it is the same stone shape and this DESIGN is the template.**
- **The two mis-named counters in `sqs.wat`** (`redeliveries` misses two of three delivery paths;
  `sends-accepted` misses the retry-put path — SCORE §2e/§2f). Real, recorded, separate.

## Blast radius, measured across every carrier

```
DeleteResponse         40 occurrences / 11 files     .rs 0     .jsonl 0
DeleteResponse::Success 12 occurrences / 9 files      3 constructions + 9 match arms
```

⚠ **My first census saw 9 files and MISSED THE DECLARATION ENTIRELY** — `wat/query.wat:534` writes
`:Success` on its own line inside the `defenum`, so the token `DeleteResponse::Success` never appears
there. The positive control caught it. **Count with the broader token.**

⛔ **`.wat` lives in FOUR homes here, and two of them have bitten this campaign before:**

```
wat/          3 files   mem.wat(1 ctor) sqlite-store.wat(1 ctor) query.wat(the declaration)
wat-scripts/  3 files   queue/sqs.wat :1110 :1614 · scratch-pad/probe-entry-three-of-ten.wat
                        :137(ctor) :143 :243 · scratch-pad/probe-no-orphan-gsi-rows-after-delete :130
tests/        2 files   rete/probe_ex001_delete_differential.wat:73 · rete/probe_ex001_store_delete.wat:71
docs/         2 files   excursus/…/PROBE-reput-divergence.wat:73 · …/PROBE-store-has-no-delete.wat:71
```

`tests/**/*.wat` omitted from a codemod path list reddened the floor with **217 failures** earlier in
this campaign. All four homes go in the list.

## ⭑ The codemod rule must be STRUCTURAL, not a file allow-list

`(…::Success)` is the *same token* in a construction and in a match-arm head, and
`probe-entry-three-of-ten.wat` contains **both** (:137 ctor, :143/:243 arms) — so a per-file allow-list
cannot separate them. The rule:

> rewrite `(…::DeleteResponse::Success)` → `(…::DeleteResponse::Success _)` **only in match-arm-head
> position** — a list whose sole child is that keyword and which is itself the FIRST child of an arm
> list. Never in value position. Idempotent: a head that already has a binder is untouched.

`fix.wat` already has `arm-head-name` (`:1012`) and `arm-heads-contain?` (`:1025`) for reasoning about
arm heads. Shape to copy: **`wat-scripts/fixes/add-event-id-to-metric-log-ctors.wat`** — insert a token
inside a matching list, idempotent, comment-faithful via `fix-text-apply` span splices.

## ⭑⭑ THE PROOF ROW, WRITTEN BEFORE THE STRIKE — because a green floor is silence here

Every match arm will bind `_`, and both stores will construct a real number **that nothing reads**. So
**the floor is green whether the count is right, always-1, or base+GSI.** The floor cannot see this
stone, exactly as it could not see the capability stone.

The strike is unproven without a probe that **reads** the value:

> delete a batch of **3 keys of which only 2 exist** → `deleted = 2`, **on mem AND on sqlite**.

Both, because stone 2b established the differential discipline and because the two backends compute the
number by completely different routes (length delta vs `stmt.execute()`). `deleted = 3` means missing
keys are counted; `deleted = 2 + 2·|indexes|` on sqlite means the GSI clear was summed in. Either way
the single number names the defect.

## Trap-doors named up front

1. **The GSI clear must not be summed.** Only the base `DELETE FROM main` count is `deleted`.
2. **`delete-one-key` / `delete-rows` return types must widen** from `Result<nil, Error>` to
   `Result<i64, Error>`, and `delete-rows` must **accumulate** across the recursion, not return the
   last key's count.
3. **A stale zero-binding arm is a hard refusal, not a silent pass** — verified by probe this session:
   exit **3** on **stderr**, `"(:X::Ok ...) takes 1 field(s), got 0 binder(s)"` plus a
   non-exhaustiveness error, both with `file:line:col`. The cascade enumerates every site; nothing can
   be silently missed. ⚠ And the errors go to **stderr with exit 3** — a piped `$?` reported **0** for
   that same run and lied to me. Never gate on a piped exit code.
4. **No stash-dance** (no Rust change) — but `wat/query.wat` is stdlib frozen at build time, so the
   three `wat/` files must change **together** or the rebuild refuses.
5. **`cargo build --release` does not compile tests.** Use `cargo nextest run --release --no-run`.
