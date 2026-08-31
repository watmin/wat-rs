# SCORE — excursus 001 stone 5: the surface guard's reach

**STRUCK.** Executor: grok, 2026-08-31. The `:messages` completeness guard now sees a
parametric field type. **The queue is broken on purpose.** Envelope does not move into
`:messages` here — that is stone 6. `wat-queue` was not edited.

```
Summary [ 302.666s] 5126 tests run: 5124 passed (2 slow), 2 failed, 17 skipped
FLOOR=100
```

Log: `.floor/2026-08-31T02-42-31Z/`. ARM: `.floor/2026-08-31T02-42-31Z/ARM.txt`.
5126 = 5122 + 4 WALL-2 reach tests. **Do not re-run.** The two fails are the queue.

`--check` of the committed repros and of the queue, after the reach landed:

```
repro/direct-field-type.wat       EXIT=1   names :p::Item
repro/parametric-field-type.wat   EXIT=1   names :p::Item     (was 0)
wat-scripts/queue/sqs.wat         EXIT=1   names :queue::Envelope
```

The parametric repro and the queue now fail at the **defsurface**, with the existing
message, located at authorship. Stone 4's runtime `unknown callee: :queue::Envelope/id`
in a forked child is this class made unrepresentable.

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | ★ the parametric repro now fails | `--check` → **1** | ✅ EXIT=1, names `:p::Item` |
| 2 | ★ the direct repro STILL fails | `--check` → **1**, unchanged | ✅ EXIT=1, same reason as row 1 |
| 3 | the message is the existing one | names `:p::Item`, byte-identical, no `.contains(` | ✅ `assert_eq!` on the whole reason in `wall2_*` tests; `--check` prints the same scalar |
| 4 | a clean surface still freezes | Vector of a **declared** user type; type-var `(Vector :- [K])` | ✅ `wall2_parametric_of_a_declared_user_type_still_freezes`; `parametric_message_is_recognized_as_declared` still PASS |
| 5 | `collect_user_type_paths` untouched | `git diff` shows no change to the function | ✅ only call sites in the new List branch; the `fn` body at `:986-995` is identical |
| 6 | the queue now fails to freeze | **expected and correct** — do not fix it | ✅ `--check` EXIT=1 at `sqs.wat:41`, names `:queue::Envelope` |
| 7 | nothing ELSE started failing to freeze | any other surface is a **finding** | ✅ see Row 7 below — no second surface |
| 8 | blast radius | `src/types/surface.rs` + its test + SCORE | ✅ `git diff --stat`: `src/types/surface.rs` only (+137/-8). Queue/topic/fanout **untouched** |
| 9 | floor | RED on the queue only | ✅ two fails, both the queue (see arms) |

## The two reds — both the queue

```
FAIL [   0.423s] (3570/5126) wat::services probe_ex001_queue::queue_lifecycle_mem_and_sqlite_agree
FAIL [ 302.660s] (5126/5126) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
```

**Arm 1** — `probe_ex001_queue.rs:16` (`startup_from_file("wat-scripts/queue/sqs.wat")`):

```
startup should succeed (queue + mem-store' + sqlite-store' baked): #wat.type/MalformedDecl
  surface :queue::Queue :messages type references :queue::Envelope which is not declared
  in this surface's :messages — … Add a (defrecord :queue::Envelope …) to :messages …
  :location sqs.wat:41:3
```

**Arm 2** — `wat_scripts_fixes_load.rs:52` (`2 of 528` files):

```
wat-scripts/fanout/circuit.wat
    … type references :queue::Envelope …  :location sqs.wat:41:3
wat-scripts/queue/sqs.wat
    … type references :queue::Envelope …  :location sqs.wat:41:3
```

`circuit.wat` is not a second surface. It `load-file!`s `sqs.wat`; the span is the
queue's defsurface. Topic (`sns-fanout.wat`) still loads. Nested parametric of an
undeclared user type is caught (`wall2_nested_*` PASS) — the reach is not one level
deep.

## Row 7 — the census, not a discouragement

The guard has been blind to parametric field types since arc 278. Widening it is
what would have surfaced any other fork-failure waiting in the tree.

**Measured: none.** 528 files under `wat-scripts/`. Two fail. Both name
`:queue::Envelope` at `sqs.wat:41`. Stdlib `:wat::` Vector-of-X (telemetry Metric/Log,
cache GetResult/Entry, query StoredRow, …) stays exempt by the existing
`starts_with(":wat::")` filter. Type-var `(Vector :- [K])` stays exempt because `K`
has no `::`.

No second surface to name. The count did not discourage the widening.

## The one branch

`src/types/surface.rs` `collect_message_form_type_refs` — the `<-` handler kept the
Keyword path (`parse_type_expr`) and gained a List path (`parse_type_node`, the
existing door for a parametric type form). Both feed `collect_user_type_paths`.
STOP-1 did not fire: that collector was not changed.

## What this stone does not do

- **Does not move `Envelope` into `Queue`'s `:messages`.** Stone 6. The floor is
  RED between them on purpose. Bundling them would hide that the guard is what
  broke the queue, and that the queue was already wrong.
- **Does not touch `UnresolvedReference`'s `&'static str` context.** With this
  guard fixed, no correct program reaches that runtime error by this route.

---

# The queue is broken on purpose

`ReceiveResponse::Ok` carries `(:wat::core::Vector :- [:queue::Envelope])` and
`Envelope` is declared above the surface, not inside `:messages`. That froze
clean because the collector skipped the List after `<-`. It now fails to freeze,
at the defsurface, with the fix in the message. Stone 6 puts Envelope in
`:messages`. Until then the floor stays red on the queue, and only on the queue.

---

# ORCHESTRATOR GRADING — re-run, not read

```
BUILD=0
direct     --check = 1   (unchanged — a widening, not a rewrite)
parametric --check = 1   (WAS 0 — the fix)
queue      --check = 1   "references :queue::Envelope which is not declared"

Summary [ 295.140s] 5126 tests run: 5124 passed (2 slow), 2 failed, 17 skipped   FLOOR=100
FAIL probe_ex001_queue::queue_lifecycle_mem_and_sqlite_agree
FAIL wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
```

**Both reds are the queue, and both are on purpose.** 5122 → 5126 = +4, the `wall2_*` tests.
`collect_user_type_paths` untouched (STOP-1). `wat-scripts/` untouched. **STRUCK.**

## The fix is a widening, and its shape protects row 2

The `<-` handler became a `match` with **both** arms — `Keyword` via `parse_type_expr`, `List`
via `parse_type_node` — rather than an `if` bolted beside the existing one. That is what keeps
trap-door 2 shut: the keyword path cannot silently stop working while the parametric path starts.
Row 2 confirms it did not (`direct` still `1`).

And it **reused `parse_type_node`** — *"the existing door for a List in a type slot"*, per its
own comment — instead of writing a parser. The BRIEF asked for that; it is the difference
between a fix and a fork.

## ★ Row 7 came back ZERO, and the instrument is why that is believable

I wrote in the BRIEF that *"any surface in the tree could carry the same latent defect"* and
told the executor not to let the count discourage the widening. **The count was zero.** My
expectation was wrong, in the safe direction.

It is believable because the coverage is not a grep — it is three instruments spanning the tree:

- **`cargo build` succeeding** covers every **stdlib** surface: `wat/` is `include_str!`'d and
  frozen at build time, so a stdlib surface carrying the defect could not compile.
- **`every_wat_scripts_file_loads`** type-checks **528 files** under `wat-scripts/` and reported
  exactly 2 failures, both the queue.
- **The floor's own runs** cover the surfaces in `wat-tests/` and `tests/`.

So *"no second latent fork-failure"* is backed by the substrate refusing to compile, not by
someone searching. That is the distinction this excursus has had to learn twice.

★ And the zero is consistent with the earlier census: the userland surfaces here carry only
builtins, and the ones with real domain vocabulary are stdlib, where the exemption genuinely
applies. **`wat-queue` is the only place it ever mattered** — because it is the first userland
surface whose messages carry a userland type.

## What this stone actually bought

Stone 4's executor met a runtime `unknown callee` in a forked child, correctly inferred *"make
the name available another way"*, and spent a workaround before reaching the real cause. After
this stone, that program **fails at the `defsurface`**, with the type named and the fix in the
message.

The wrong turn is not signposted better. **It is no longer reachable.**

## Owed — and the floor is red until it lands

**Stone 6: `Envelope` moves inside `:queue::Queue`'s `:messages`.** Small. The floor is red on
the queue between these two stones, by design and by the BRIEF's instruction not to bundle them.
