# FINDING — the malformed census

Drawn `a6da4c256`. Executor: grok, 2026-09-13, branch `sns-sqs`. **No substrate change.**
`git diff --stat -- src/ wat/` is empty. Nothing migrated. No `wat/lint.wat` rule.

Instrument: `wat-scripts/census-malformed-raising.wat` (report-only, form-walk).
Raw stdout (EDN-quoted lines): `census-run.txt` beside this file.

---

## 0. Instrument — one correction before any count

DESIGN's CONTROL A said "must not flag circuit.wat / sns-fanout.wat". Both files still carry **other** raising Malformed placeholders. A file-level control would have shown PRESENT on a correct walker.

CONTROL A is the **migrated ARM**, not the file:

| arm | line | body | raising-rows |
|---|---|---|---|
| circuit.wat poisoned-call | **544** | string `"malformed"` | **ABSENT** |
| sns-fanout.wat twin | **369** | string `"malformed"` | **ABSENT** |

Those two arms appear under **unclaimed** with `raises=string`. A line-oriented pass would have flagged them: the same physical line holds a raising `Stopped` arm.

CONTROL B: `wat/service.wat:3281` **PRESENT** (`raises=assertion-failed!`). Enclosing top-level head is **`other`** — the arm lives inside `defmacro :wat::service::defservice`, not a `defservice` form. Not resolved as a service item (STOP-4).

STOP-5: no unclaimed body is `let` / `do` / `if` / a defmacro call. Unclaimed shapes are string, `println`, a pass-through `RecvOutcome::Malformed` constructor, and `owner-recv-loop` (recursive continue, not a raise).

---

## 1. Controls (stated in the census stdout)

```
CONTROL-A circuit.wat:544 raising-Malformed: ABSENT
CONTROL-A sns-fanout.wat:369 raising-Malformed: ABSENT
CONTROL-B wat/service.wat raising-Malformed: PRESENT
STOP-5 unclaimed let/do/if/macro body: ABSENT
```

Two runs, same sorted path list, `diff` empty.

Path list: `find wat wat-scripts tests docs -name '*.wat' | sort` → **1753** files.
`wat-tests/` is **out** of this list (DESIGN homes; 80 grep tokens, fixtures). Stated, not walked.

---

## 2. The 487 → findings gap

Token counts are `grep -o RecvOutcome::Malformed | wc -l` (occurrences, never `grep -c`).

| layer | n | what it is |
|---|---|---|
| DESIGN's 487 | 487 | draw-time grep over `wat/` + `wat-scripts/` + `tests/` |
| + `docs/` | +3 | path list includes docs |
| + this census program | +3 | 1 comment + 2 string literals in `census-malformed-raising.wat` |
| **grep on this path list** | **493** | comments + strings + keyword nodes |
| form-tree keyword nodes | **482** | walker; comments and string contents are invisible |
| Malformed **match-arms** | **479** | keyword is the arm's pattern (list-head or the keyword itself) |
| of which body **raises** | **464** | findings |
| of which body does not | **15** | unclaimed |

493 − 482 = **11** tokens that are not keyword nodes (comments + string literals).
482 − 479 = **3** extra keyword nodes (pattern + a constructor in value position on the same arm, plus two similar).
479 − 464 = **15** match-arms whose body is not a raise form.

`raise!` and `panic!` are recognised by the walker. **0** Malformed arms use them. All **464** findings are `assertion-failed!`. The 10 `raise!` + 1 `panic!` in the corpus are not on Malformed arms.

---

## 3. Classification — the builder's filter is a column

Raising-arms **464**. No blank `head`. No blank `home`.

### by raise form

| form | n |
|---|---|
| `assertion-failed!` | 464 |
| `raise!` | 0 |
| `panic!` | 0 |

### by enclosing top-level head

| head | n |
|---|---|
| `defn` | 420 |
| `defservice` | 39 |
| `other` | 5 |
| `defsurface` | 0 |
| `deftest` | 0 |

`deftest` = 0: every `tests/` probe here is a top-level `defn`. `wat-tests/` (excluded) is where `deftest` lives.

### by home

| home | n |
|---|---|
| `tests` | 271 |
| `scratch-pad` | 78 |
| `other` | 54 |
| `wat-scripts/service` | 32 |
| `wat` | 26 |
| `docs` | 3 |

`home=other` is `wat-scripts/probes/**` (53) + `wat-scripts/query/faulting-store.wat` (1). Not resolved as service items.

### head × home

The cut the builder named is a **cell**, not a judgement. Not resolved here.

| | wat | wat-scripts/service | scratch-pad | tests | docs | other |
|---|---|---|---|---|---|---|
| **defservice** | 6 | **18** | 3 | 8 | 0 | 4 |
| defsurface | 0 | 0 | 0 | 0 | 0 | 0 |
| defn | 15 | 14 | 75 | 263 | 3 | 50 |
| deftest | 0 | 0 | 0 | 0 | 0 | 0 |
| **other** | **5** | 0 | 0 | 0 | 0 | 0 |

---

## 4. Ambiguous rows (STOP-4 — marked, not resolved)

The five `head=other` raising rows. All `home=wat`. The enclosing top-level is **not** `defservice`/`defsurface`/`defn`/`deftest`:

| file:line | actual enclosing form | not resolved as |
|---|---|---|
| `wat/service.wat:3281` | `defmacro :wat::service::defservice` (CONTROL B — the template) | service item |
| `wat/query.wat:486` | `defmacro :wat::query::sift-rules-defsvc` (generates a defservice) | service item |
| `wat/bracket.wat:508` | `defclause :wat::bracket::process-work-forms` | service item |
| `wat/spawn.wat:539` | `extend-type :wat::spawn::ThreadOpts :wat::spawn::Locus` | service item |
| `wat/spawn.wat:596` | `extend-type :wat::spawn::ProcessOpts :wat::spawn::Locus` | service item |

---

## 5. `defservice` raising rows (the column, all 39)

### home=wat-scripts/service (18)

```
wat-scripts/fanout/circuit.wat:666:80
wat-scripts/fanout/circuit.wat:839:94
wat-scripts/fanout/circuit.wat:1106:377
wat-scripts/fanout/circuit.wat:1152:788
wat-scripts/fanout/circuit.wat:2178:58
wat-scripts/fanout/circuit.wat:2272:151
wat-scripts/queue/sqs.wat:346:90
wat-scripts/queue/sqs.wat:362:83
wat-scripts/queue/sqs.wat:404:294
wat-scripts/queue/sqs.wat:444:281
wat-scripts/queue/sqs.wat:614:22
wat-scripts/queue/sqs.wat:930:32
wat-scripts/queue/sqs.wat:1387:92
wat-scripts/topic/sns-fanout.wat:164:38
wat-scripts/topic/sns-fanout.wat:211:621
wat-scripts/topic/sns-fanout.wat:528:918
wat-scripts/topic/sns-fanout.wat:575:177
wat-scripts/topic/sns-fanout.wat:605:629
```

### home=wat (6) — stdlib journal

```
wat/telemetry/journal.wat:173:271
wat/telemetry/journal.wat:217:271
wat/telemetry/journal.wat:269:271
wat/telemetry/journal.wat:320:271
wat/telemetry/journal.wat:388:278
wat/telemetry/journal.wat:454:278
```

### home=tests (8) / scratch-pad (3) / other (4)

tests: `tests/services/probe_arc278_{peers_bijection_case{1..5},s2s_peer_on_{process,thread},sift_arena}.wat`
scratch-pad: `probe-chaos-is-a-rate.wat:152`, `probe-handler-issues-request-and-returns.wat:160`, `probe-worker-claim-drop.wat:95`
other: `wat-scripts/probes/arc-278/s2s-{midlife-vec,process,revoke,thread}-probe.wat`

Raising in probes and tests is where dying loudly is correct. Separable at a glance via `home`.

---

## 6. Unclaimed Malformed arms (15) — body does not raise as head

| file:line | body shape | what it is |
|---|---|---|
| circuit.wat:**544** | string | CONTROL A — migrated |
| sns-fanout.wat:**369** | string | CONTROL A twin |
| wat/service.wat:2633 | `:wat::kernel::RecvOutcome::Malformed` | pass-through in the defservice macro's client method |
| wat/service.wat:3873 | `:wat::service::owner-recv-loop` | recurse with last="Malformed"; not a raise |
| 8 scratch-pad probes | string | crash-surface / accepted-is-a-count / store-can-fail probes return a label |
| 3 scratch-pad probes | `:wat::kernel::println` | print the outcome, don't raise |

---

## 7. What this census does not claim

- **Not a work list.** 464 is a population. The builder sequences.
- **Not "service items = 18."** That cell is readable; whether journal, the defservice macro, or `sift-rules-defsvc` belong is theirs.
- **Not wat-tests.** 80 tokens left on the other side of the path list.
- **Not the other RecvOutcome variants** (Lost, Closed, TimedOut, Stopped). Same shape, larger population; this census is Malformed.

Reproduce:

```
find wat wat-scripts tests docs -name '*.wat' | sort \
  | python3 -c 'import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))' \
  | ./scripts/capped.sh --limit 8g ./target/release/wat ./wat-scripts/census-malformed-raising.wat
```
