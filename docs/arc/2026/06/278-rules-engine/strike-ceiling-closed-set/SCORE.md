# SCORE — E4, weighed against the orchestrator's own re-run

> Re-run here at `452953cb9`.

## Row 1 — the strike, re-driven, and it fires in FOUR places

Adding a fifth `ReteCeiling` variant:

```
error[E0004]: non-exhaustive patterns: `&ReteCeiling::OrchestratorProbeCeiling { .. }` not covered
  --> src/rete/kernel/outcome.rs:96:55     (fire)
  --> src/rete/kernel/outcome.rs:169:55    (insert)
  --> src/rete/kernel/outcome.rs:234:55    (compile)
  --> src/value/signal.rs:790:55           (fmt_with_span — NO wildcard at all)
```

My scorecard said "exactly 3". The measured answer is **3 converters + 1 message**: a fifth ceiling
cannot compile until it is both **routed** and **given a message**. That is better than the row asked
for, and the rider scored it as "a pass with a correction" — correctly.

## The rest of the scorecard

| # | after |
|---|---|
| 2 | ✅ disjoint sets preserved, each now a written arm |
| 3 | ✅ **zero** wildcards inside any `ReteCeiling` match (grepped here) |
| 4 | ✅ the three outer `_ =>` kept |
| 5 | ✅ outcome enums byte-identical — no constructor, type-path const, or `wat/` file touched |
| 6 | ⚠ **prose messages byte-identical; the EDN TAG changes** — the finding below |
| 7 | ✅ lint green **and unmodified** — see thin spot A |
| 8 | ⚠ **30 refs, not my 36** — thin spot B |
| 9 | ✅ lint 116/116 |
| 10 | ✅ `Summary [ 386.066s] 5230 tests run: 5230 passed, 21 skipped`, zero FAIL rows |
| 11 | ✅ clippy rc=0 |

## ⚠ THE DECISION I OWED: the derived EDN tag changes

`RuntimeErrorKind` derives `ToEdn` and `Display` **is** `to_wire_edn`, so nesting four flat variants
changes the rendered error:

```
before  #wat.runtime/FixpointRoundCapExceeded {:cap 50 :still-deriving 12 :span …}
after   #wat.runtime/ReteCeiling {:ceiling #wat.runtime/FixpointRoundCapExceeded {…} :span …}
```

The rider raised it as a finding rather than absorbing it, and put the call to me. **Accepted**, on
measured grounds it supplied and I checked: the derive's grammar is namespace / literal / via / key /
skip — **no flatten or transparent exists**, so it is unavoidable under the ★; **the prose messages
are byte-identical** (verified by extracting both revisions' literals); no test and no `.edn` golden
asserts these tags; every wat-level match is on the outcome enums; and all four convert to an outcome
before reaching wat, so the tag is visible only to a Rust embedder's `Debug`/`Display`.

**Recorded at the enum**, with the measurement and the cure, so it cannot rot into folklore. The
`#[to_edn(transparent)]` directive (~30 lines) is filed as its own strike — it is a new wire
directive, not a side effect of this one.

## ⛔ Where MY brief was thin

- **A. ★ My sketch would have DEFANGED THE LINT.** It renamed the variants to `SessionMemory` /
  `FixpointRoundCap` / `SessionMemoryOnInsert`. `no_ceiling_raise_in_rete` matches by
  `line.contains(v)` and asserts `allowed_hits >= 4` — those renames stop matching at **all four
  doors**, and my trap 5 told the rider to "update them so the lint still fires", i.e. to re-point a
  live gate at strings I had just invented. **The rider did the safer opposite**: kept all four names
  verbatim, so `CEILING_VARIANTS` needed no edit and the gate stayed live and unmodified. A gate
  re-pointed at the strings of the change it is meant to police is not a gate.
- **B. My radius was wrong a third time, and the failure mode now has a name: SUBSTRING.**
  `SessionMemoryCeilingExceeded` is a **prefix of** `SessionMemoryCeilingExceededOnInsert`, so my
  `grep -c` counted those six twice. Word-boundary count: **30 across 7 files**, not 36. All three
  wrong estimates came from naive greps, not from failing to look. Memory updated: use `-w`/`\b`
  whenever a name family shares a prefix — which error families almost always do.
- **C. Trap 3's "only the error side is re-typed" reads as reassurance and is not.** True of the
  *outcome* enums; but it invites reading the error side as internal, and it is a wire surface. The
  rider named this as the one thing to add to the stone, and it is the sentence that would have let
  the EDN change slip past unremarked.
- **D. DESIGN cites `:103/:161/:213` in § Why while its own table says `:90/:148/:200`** — wildcard
  lines versus fn lines. Harmless, but one document disagreeing with itself.

## Arms not driven, named

**All eight cross-converter refusal arms** — **not reachable, and why**, with the call graph checked
rather than assumed: each ceiling has exactly one construction site (lint-enforced) and one route —
`check_insert_ceiling`'s two callers both land in `insert_result_to_outcome`,
`refuse_non_terminating`'s single call site (`arm.rs:1329`) lands in `compile_result_to_outcome`, and
`delta.rs`'s two land in `fire_result_to_outcome`. All 10 converter call sites checked; every
wat-facing verb converts, so no ceiling escapes as a raise. **These arms exist to make a fifth
variant a build failure, not to run** — which is exactly the ★.
