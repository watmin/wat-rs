# BRIEF — STONE 218.7: the clj oracle runs to DRY (wat-edn conformance, generated not hand-listed)

**Drawn 2026-09-19 against `main` @ `53946dac3`** (floor 5920/5920, clippy 0, census `no STOP-8`).
Arc 218's mission, quoted: *"the beginning of our arc to extend wat-edn — **get it clean first** —
then we work upon it."* This stone is that sentence, four months later, with a measurement.

**Why now — the builder's pivot, verbatim:** *"sounds like our pivot is attack wat-edn crate as its
correctness is now in question."* It came out of the highlander question — *"there must be only one
source of truth for edn compliance"* — and the designated source of truth cannot anchor a merge
while its conformance is unmeasured.

---

## ⛔⛔ READ THIS FIRST — THE ORCHESTRATOR ASSERTED A SPEC SENTENCE THAT DOES NOT EXIST

While drawing this, the orchestrator told the builder:

> *"`wat-edn` refuses `clojure.core//` — the spec blesses it explicitly: '`/` by itself is a valid
> symbol, **as is `clojure.core//`**.' That's a genuine bug in the crate you believe is correct."*

**The quoted clause does not appear in the EDN spec.** Fetched from `github.com/edn-format/edn`:

> *"`/` has special meaning in symbols. It **can be used once only** in the middle of a symbol to
> separate the prefix (often a namespace) from the name, e.g. `my-namespace/foo`."*
>
> *"`/` by itself is a legal symbol, but otherwise **neither the prefix nor the name part can be
> empty** when the symbol contains `/`."*

`clojure.core//` contains **two** `/`. Read literally, **wat-edn's refusal may be CORRECT** and the
orchestrator's "bug" report was wrong. What the builder proved in his REPL — `(clojure.core// 2 1)`
⇒ `2` — is **Clojure's LANGUAGE reader**, which special-cases this symbol. It is *not* evidence about
`clojure.edn`, and `clojure.edn` is our oracle.

⚠ **`crates/wat-edn/src/value.rs:312` carries the same false claim** in a code comment —
*"The name `/` itself (division, Clojure `clojure.core//`) is the one exception"* — so this belief is
in the tree, not only in the orchestrator. **Whichever way the oracle rules, that comment is a defect:
it states a spec fact without a test.**

⛔ **This retraction is the brief's whole argument.** Two "authoritative" readings of the spec — arc
219's and the orchestrator's — were both wrong, in opposite directions, about the same few sentences.
**Human reading of the EDN spec is not a conformance instrument. The differential oracle is.**

---

## WHAT IS ESTABLISHED (measured, reproducible)

A 64-row conformance sweep against `wat_edn::parse_owned`, oracle written from the spec:
**56 conform, 8 deviate.** Triaged against the fetched spec text:

### ✅ CONFIRMED deviations — the spec is unambiguous

| # | case | ours | spec |
|---|---|---|---|
| 1 | `a:b` | **REJECT** | *"Additionally, `: #` are allowed as constituent characters in symbols other than as the first character."* |
| 2 | `a#b` | **REJECT** | same sentence |
| 3 | `{:a 1 :a 2}` | **accept**, keeps BOTH | *"Each key should appear at most once."* |
| 4 | `#{1 1}` | **accept**, keeps BOTH | *"A set is a collection of unique values."* |

⛔ **On 3 and 4 the value is worse than the verdict.** `Map` is a `Vec<(Value, Value)>`, so a
duplicate key **survives into the parsed value**: `{:a 1 :a 2}` → `Map([(a,1),(a,2)])`. Any consumer
doing a linear lookup silently gets whichever it scans first. **This is a data-integrity defect, not
a strictness preference**, and it should be sized before it is fixed: something may depend on it.

### ⚠ UNRESOLVED — the oracle decides, not the orchestrator, not the builder's memory

| # | case | ours | why it is open |
|---|---|---|---|
| 5 | `clojure.core//` | REJECT | spec says "once only"; Clojure's *language* reader special-cases it. **`clojure.edn` untested.** |
| 6 | `123456789012345678901234567890` | REJECT (i64 cap) | spec only says the **`N` suffix** requests arbitrary precision; it does not say plain integers must be arbitrary. `clojure.edn` promotes automatically. |
| 7 | `#inst "1985-04-12"` (date-only) | REJECT | spec says `#inst` is an RFC-3339 timestamp; RFC-3339 wants a full date-time. Clojure's instant reader is believed lenient — **believed, not measured.** |

⭐ **Each is settled by one line in the builder's REPL** — and the executor must run them, not assume:

```clojure
(require '[clojure.edn :as edn])
(edn/read-string "clojure.core//")
(edn/read-string "123456789012345678901234567890")
(edn/read-string "#inst \"1985-04-12\"")
```

### ✅ DELIBERATE supersets — NOT bugs, but they must be EXEMPTED EXPLICITLY

| case | status |
|---|---|
| `1/2` ratios | wat extension (arc 300 stone A). Not in the EDN spec. |
| `x'` primed symbols | documented at `writer.rs:421` — *"wat is a Clojure dialect: a trailing prime `'` is a legal symbol/keyword BODY character."* ⚠ The doc says **trailing**; `a'b` (MEDIAL) is also accepted. **Measure whether medial is intended.** |

---

## ⛔ THE FINDING FOR THE BUILDER — ARC 219's PREMISE DOES NOT HOLD

Deviations 1 and 2 were **introduced on purpose**. `docs/arc/2026/05/219-.../DESIGN.md` states:

> *"**EDN spec symbol body chars** (per `github.com/edn-format/edn`): alphanumeric + `. * + ! - _ ? $
> % & = < >`. NO `:` (except as keyword prefix at position 0). NO `#` (reserved for tag/discard
> prefixes)."*

and landed under the builder's standing instruction: *"open 219 and do it now - edn now demands it -
we satisfy it - 218 is blocked until 219 is done."*

**That summary omits the spec's very next sentence** — the "Additionally, `: #` are allowed…" clause.
So 219 removed characters the spec explicitly permits, believing it was tightening toward the spec.

⛔ **The ruling STANDS until the builder rules otherwise**
(`[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`). There may be an independent reason
to refuse `:`/`#` in wat symbol bodies — the wat surface uses `::` structurally, and 8d is retiring
exactly that. **The executor's job is to report the premise failure and the measured blast radius,
NOT to revert 219.** A revert is the builder's call and only his.

**Measure before asking:** how many symbols in the tracked corpus would a re-permitted `:`/`#`
change the lexing of? If the answer is zero, this is a cheap correctness win; if it is large, it
collides with 8d and must queue behind it.

---

## ⭐ THE ACTUAL STONE — the corpus, not the bug list

**The ward already exists and is correctly designed.** `crates/wat-edn/tests/clj_oracle_parity.rs`
is a differential against `clojure.edn`, golden-file backed so CI runs without Clojure, with the
right directional doctrine already written into its header:

> *"The obligation is directional: **wat must accept everything clj accepts** — a `clj:OK / wat:ERR`
> row is a wat bug. A `clj:ERR / wat:OK` row means wat read valid EDN clj's default declined:
> examine it — a wat superset of valid EDN is allowed (exempt it with a reason), accepting invalid
> EDN is a bug."*

**Nothing about that needs changing. The corpus is the hole:**

```
corpus.txt: 72 hand-written cases
  ABSENT  clojure.core//          ABSENT  {:a 1 :a 2}
  ABSENT  a:b                     ABSENT  #{1 1}
  ABSENT  a#b                     ABSENT  123456789012345678901234567890
lines with 2+ slashes: 0      lines with 19+ digit numbers: 0
```

Every failing case is absent, and whole regions of the grammar are unprobed. The ward's own header
says *"grow the corpus until it stops finding divergences (**loop-until-dry**)"* — **that loop was
never run to dry.** `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`.

### The work

**1 — GENERATE the corpus from the grammar, do not extend the hand-list.**
Enumerate systematically, from the spec's own structure. At minimum:
- **symbols**: each constituent char from the spec list, in first vs non-first position; `:` and `#`
  in both; slash counts 0 / 1 / 2 / 3; empty prefix (`/x`), empty name (`x/`); leading `-`/`+`/`.`
  followed by numeric vs non-numeric; the bare `/`; `clojure.core//`
- **integers**: 0, ±1, i64::MAX, i64::MAX+1, i128 boundary, 30 digits, each with and without `N`
- **floats**: exponent forms, `M` suffix, `.5`, `5.`
- **strings**: every legal escape, an illegal escape, `\uNNNN` valid and truncated
- **chars**: every named char, `\uNNNN`, a bare `\`
- **collections**: unbalanced, odd-length maps, duplicate keys, duplicate set elements, commas
- **`#` dispatch**: `#{}`, `#_`, `#inst` (full / date-only / malformed), `#uuid` (valid / malformed),
  unknown tags
- **NOT-EDN controls**: `'x`, `` `x ``, `~x`, `~@x`, `@x`, `^:m x` — these MUST be refused; they are
  the negative controls that keep the generator honest
  (`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`)

⚠ **A generator with no negative controls will happily generate only what we already accept.**

**2 — REGENERATE the golden against real `clojure.edn`.** The builder has `clj` installed as of
today, so `regen.clj` runs. ⛔ **Do not hand-write a golden verdict. Ever.** The whole value of the
ward is that a human never decides what is legal.

**3 — LOOP TO DRY.** Grow → regen → run → fix or exempt → repeat, until a generation round finds
**zero new divergences**. ⛔ **"Dry" is a measured result, not a feeling** — the SCORE states how
many rounds ran and how many divergences each round produced, and the last one must be 0.

**4 — Fix what falls out, exempt what is deliberate.** Every exemption carries a reason in
`exemption()`, as the existing ones do. ⛔ **An exemption without a load-bearing reason is how a
conformance suite dies** — it is the hand-list growing back.

### ⚠ Two known limits of the harness — report, do not silently work around

- **One case per LINE.** `corpus.txt` is line-delimited, so an input containing a newline (a
  multi-line string, a `;` comment followed by a form) **cannot be expressed**. That is a real blind
  spot; say whether it matters and propose the smallest fix if it does.
- **`edn/read-string` reads ONE value.** Trailing content is not an error to it, so `1 2 3` reads as
  `1`. Accept/refuse parity on multi-form input is therefore **not** what this ward measures.

---

## The gate

- `scripts/floor.sh` green, clippy `-D warnings --all-targets --workspace` 0 — **the orchestrator's
  row**, run uncontended. Predict the test-count delta from the diff, confirm with
  **`cargo nextest list`**.
- The parity ward green with the **generated** corpus, and the SCORE names the corpus size before
  and after (72 → N).
- **Round-to-dry table**: rounds run, divergences per round, last round 0.
- Every remaining divergence is an `exemption()` with a reason a reader can check.
- ⛔ The three REPL one-liners (5, 6, 7 above) **answered with their actual output pasted in**.

## Doctrine — `wat-rs/CLAUDE.md` does not reach a subagent

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture it whole the first time; never re-run.
- ⛔ **THE ORACLE DECIDES, NOT THE SPEC-AS-READ.** This brief exists because two careful readings of
  four sentences produced two opposite wrong answers. If `clojure.edn` and your reading of the spec
  disagree, **`clojure.edn` wins and you report the discrepancy.**
- ⛔ **DO NOT REVERT ARC 219.** Report its premise failure and the blast radius. The builder rules.
- ⛔ **ASK THE TOOL THAT OWNS THE FACT.** Test counts → `cargo nextest list`.
- ⛔ **If this brief contradicts the code, the spec text, or the oracle, THEY WIN** — land what is
  true and report the brief's error. This brief already contains one retracted orchestrator claim;
  assume there is another.
- Work only in `/home/john/work/holon/wat-rs`. No `git filter-branch`. Do not push.

## Out of scope — affirmatively cut

- **`wat-reader`'s conformance.** It diverges from `wat-edn` on 12 of 29 probed cases and cannot read
  back the tagged literals wat itself prints (`#wat.core/Span …` reads as 2 forms: a symbol named
  `"#wat.core/Span"`, then a map). **That is the highlander, and it is arc 300's**, not this stone's.
  This stone makes the *source of truth* trustworthy so that merge has an anchor.
- **The `::` retirement.** 251.8d.
- **Performance.** The builder's claim is *"the fastest correct EDN parser in rust's ecosystem"*;
  this stone addresses only the second word. ⛔ **Do not trade conformance for speed here, and do not
  benchmark as part of this stone** — a perf change landing beside a correctness change makes both
  unreviewable.
