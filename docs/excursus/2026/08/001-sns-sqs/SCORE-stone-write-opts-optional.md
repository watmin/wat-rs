# SCORE — excursus 001 stone WO-OPT: the opts argument becomes OPTIONAL

**STRUCK.** Executor: grok, 2026-08-30. Corrects WRITE-OPTS' required arity of 2 — that
was the sketch, not the intent. Floor is the expected one-failure shape.

```
Summary [ 304.823s] 5119 tests run: 5118 passed (3 slow), 1 failed, 17 skipped
FAIL [   0.708s] (3552/5119) wat::services probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close
FLOOR=100
```

That one failure is the pre-existing journal key-collision arm. **Not this stone's. Not re-run.**
ARM: `.floor/2026-08-30T21-55-06Z/ARM.txt`.

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | `(write-json v)` type-checks | CHECK=0 | ✅ identity deftest type-checks and runs; render.wat 1-arg sites PASS |
| 2 | ★ 1-arg ≡ 2-arg-with-default | byte-identical | ✅ `write-json-one-arg-equals-two-arg-with-opts` PASS (`2727/5119`) — `assert-eq` of the whole string |
| 3 | same for `write-json-natural` | identical | ✅ `write-json-natural-one-arg-equals-two-arg-with-opts` PASS (`2728/5119`) |
| 4 | 3 args still rejected | type error | ✅ `write_json_three_args_is_a_check_error` + natural twin PASS |
| 5 | 0 args still rejected | type error | ✅ `write_json_zero_args_is_a_check_error` + natural twin PASS |
| 6 | the guard is in the CHECKER | `infer_` fn | ✅ `infer_edn_write_json` / `infer_edn_write_json_natural` in `check.rs`; dispatch arms intercept both verbs |
| 7 | the exception is documented | intrinsic header | ✅ `edn.rs` module header: "eleven Exact, two optional-arity" — Variadic census reason sits next to the verbs, like `reader.rs:80` |
| 8 | ⛔ `write` / `write-pretty` unchanged | Exact(1), `params: vec![t_var()]` | ✅ still that scheme; JSON verbs now share the 1-arg loop (the 2-arg loop is gone) |
| 9 | no registry reshape | `src/intrinsic/mod.rs` empty | ✅ empty — Variadic already exists; no `Range`/`AtLeast` |
| 10 | `wat/edn.wat` unchanged | empty | ✅ empty |
| 11 | `crates/wat-edn/` unchanged | empty | ✅ empty |
| 12 | floor | exactly one failure, the known arm | ✅ exactly that one. 5119 = 5113 + 2 identity deftests + 4 arity tests |
| 13 | prior stones | `probe_ex001_*`, 6 inst arms, write-opts arms | ✅ store_delete / delete_differential / reput_differential PASS; inst arms PASS; clamp/digits arms PASS |

## The exemplar, copied

`:wat::io::IOReader/read-frame`:

- Variadic handler (`xs: &[WatAST]`)
- named `infer_` arity guard (`args.is_empty() \|\| args.len() > 2`) producing `MalformedForm`
- dispatch arm that intercepts, comment stating 1-or-2
- header note that it is the optional-arity exception

JSON verbs are that shape, twice (they do not share a renderer — trap-door 2). Runtime still has a 1-or-2 `MalformedForm` like `eval_ioreader_read_frame` for calls that skip the checker; check-time authority is the `infer_` fn.

## The 8 live call sites

Reverted to 1-arg where they wanted the default (`wat-tests/edn/render.wat` ×2, four scratch-pad files ×6). Proof of row 1. 2-arg with explicit opts remains on the clamp/digits probes, which are choosing.

## STOP triggers

- **STOP-1** (`write` / `write-pretty`): did not fire.
- **STOP-2** (`Range`/`AtLeast`): did not fire.
- **STOP-3** (floor reds outside the journal arm): did not fire.
- **STOP-4** (`.contains(`): identity tests use `assert-eq` of the whole string. Arity tests match `reason.as_str() == "…"` — no `.contains(`.

## Porcelain at report time

```
 M src/check.rs
 M src/edn/render.rs
 M src/intrinsic/edn.rs
 M wat-scripts/scratch-pad/probe-json-natural-record.wat
 M wat-scripts/scratch-pad/probe-mcp-reply-emit.wat
 M wat-scripts/scratch-pad/probe-mcp-response-shape.wat
 M wat-scripts/scratch-pad/probe-mcp-wire.wat
 M wat-tests/edn/render.wat
 M wat-tests/edn/write-opts.wat
?? tests/rete/probe_ex001_write_opts_arity.rs
?? tests/rete/probe_ex001_write_opts_arity__*.wat.bad
?? docs/excursus/2026/08/001-sns-sqs/SCORE-stone-write-opts-optional.md
```

Uncommitted. Not pushed. `wat/edn.wat`, `crates/wat-edn/`, `src/intrinsic/mod.rs` empty.

---

# ORCHESTRATOR GRADING — re-run, not read

```
Summary [ 294.947s] 5119 tests run: 5118 passed (2 slow), 1 failed, 17 skipped     FLOOR=100
FAIL (3552/5119) probe_arc278_span_macros…    ← the known journal key-collision arm
```

5113 → 5119, +6. **STRUCK.**

## The two gates, measured by my own probes rather than read from the rows

**STOP-1** — `(:wat::edn::write 1 (:wat::edn::opts))` → `expected 1 argument`. Rejected.
Three stones running, `write` stays `Exact(1)` and the Store sort-key path is untouched.

**Row 2, the real gate** —

```
json-identical=true   natural-identical=true
a={"#inst":"1970-01-01T00:00:01.200000000Z"}
```

Byte-identical for both verbs, and the default is visibly **nanos** — `.200000000Z`, which is
the exact value whose variable-width rendering broke lexicographic ordering before stone INST.
The excursus closes on itself there.

Rows 9/10/11 confirmed empty: no `Range`/`AtLeast` in the registry, `wat/edn.wat` untouched,
`crates/wat-edn/` untouched.

## The executor improved on the exemplar

`read-frame` has one `infer_` fn. This stone has two verbs, and rather than duplicate the
guard, both delegate to a shared `infer_edn_json_verb(op, …)`. The twin's doc comment answers
EXPECTATIONS trap-door 2 **in the code** rather than in a report:

> *"The two verbs do not share a renderer (natural stringifies Instant before
> `to_json_string`); they share this arity contract."*

The arity guard produces a real `MalformedForm` naming the expected shape —
`"expected 1 or 2 args (value [opts :wat::edn::WriteOpts]); got 0"` — and the negative fixtures
assert on that string with `==`, not `.contains(`.

## ★ Two repairs demonstrated themselves, one stone apart

1. **The census-command habit** (WRITE-OPTS): the BRIEF showed its command, the executor
   audited the number down from 23 to 8. Auditable, and audited.
2. **The `.contains(` warning** (this stone): carried into the BRIEF up front instead of being
   rediscovered. `no_loose_string_assert` had caught the two previous stones; **it did not fire
   here.** The negative fixtures used exact equality from the start.

Both were written down after a failure and then worked on their first use. That is the
difference between a note and a habit.

## And nothing of mine was wrong in this one

No scoping miss, no census error, no sketch that quietly over-specified. The difference is
structural, not virtue: **this BRIEF pointed at a live exemplar (`IOReader/read-frame`, three
exact `file:line`s) instead of describing a shape from memory.** There was less of mine for the
executor to work around.

That is the lesson to carry: when a pattern already exists in the tree, cite it and stop
writing. My three scoping misses today were all in briefs where I described rather than pointed.

## Owed, unchanged

1. The journal `SortKey` — the remaining red, and the excursus's largest finding.
2. `time-sk` — now a workaround for a defect that no longer exists.
3. The both-backends census over every `journal` fixture.
