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

`clojure.core//` contains **two** `/`, so on the spec text alone the refusal looks defensible.

⭐ **THE ORACLE LATER RULED THAT `clojure.edn` DOES ACCEPT IT — so the CONCLUSION was right.**
⛔ **That does not rescue the claim.** The orchestrator asserted a verbatim spec sentence that does
not exist, and happened to land on the right answer. **Being accidentally right is not being right**,
and a fabricated quotation is worse than a wrong conclusion because it is unfalsifiable by the reader
— it *looks* like evidence. `[[feedback_the_authority_you_cite_decides_who_can_catch_you]]`.

⚠ **`crates/wat-edn/src/value.rs:312` carries the same false claim** in a code comment —
*"The name `/` itself (division, Clojure `clojure.core//`) is the one exception"* — so this belief is
in the tree, not only in the orchestrator. **Whichever way the oracle rules, that comment is a defect:
it states a spec fact without a test.**

⛔ **This retraction is the brief's whole argument.** Two "authoritative" readings of the spec — arc
219's and the orchestrator's — were both wrong, in opposite directions, about the same few sentences.
**Human reading of the EDN spec is not a conformance instrument. The differential oracle is.**

---

## ⭐ THE ORACLE HAS RULED — 57 rows run through real `clojure.edn` (2026-09-19)

⛔ **Every "unresolved" row in the first draft of this brief is now RESOLVED, by running the oracle.**
`clj` is at `/usr/local/bin/clj` **in this environment** — the orchestrator asked the builder to run
three REPL lines it was fully capable of running itself. That was the process failure; the builder
named it. **The oracle is a tool on this machine, not a favour to ask for.**

Corpus of 57 rows → `clojure.edn/read-string` → diffed against `wat_edn::parse_owned`:

### **45 parity · 9 WAT BUGS (clj:OK / wat:ERR) · 3 supersets (clj:ERR / wat:OK)**

#### The 9 bugs — clj accepts, we refuse. Per the ward's own doctrine each is a wat bug.

| # | input | clj reads it as | our defect |
|---|---|---|---|
| 1 | `9223372036854775808` | `…808N` | **i64 cap**; `num-bigint` is already a dependency |
| 2 | `123456789012345678901234567890` | `…N` | same |
| 3 | `clojure.core//` | `clojure.core//` | multi-slash refusal |
| 4 | **`a/b/c`** | `a/b/c` | **`clojure.edn` TOLERATES multi-slash** despite the spec's "once only" |
| 5 | `a:b` | `a:b` | arc 219 removed `:` from symbol bodies |
| 6 | `a#b` | `a#b` | arc 219 removed `#` |
| 7 | `#inst "1985-04-12"` | `#inst "1985-04-12T00:00:00.000-00:00"` | date-only promoted; we refuse |
| 8 | **`'x`** | `'x` (quote) | **`clojure.edn` reads QUOTE** |
| 9 | **`^:m x`** | `x` (meta attached) | **`clojure.edn` reads METADATA** |

#### The 3 supersets — clj refuses, we accept. Two are bugs; one is already exempt.

| input | clj | ours | verdict |
|---|---|---|---|
| `{:a 1 :a 2}` | `ERR: Duplicate key: :a` | accepts, **keeps both** | ⛔ **BUG — accepting invalid EDN** |
| `#{1 1}` | `ERR: Duplicate key: 1` | accepts, **keeps both** | ⛔ **BUG — accepting invalid EDN** |
| `#myapp/Person {:first "F"}` | declines unknown tag | reads generically | ✅ already exempted, with reason |

**⇒ 11 real defects, 1 justified exemption, on a 57-row probe.**

⛔ On the duplicates the *value* is worse than the verdict: `Map` is a `Vec<(Value, Value)>`, so the
duplicate **survives into the parsed value** and a linear lookup silently takes whichever it scans
first. Size what depends on that before changing it.

---

## ✅ THE BUILDER RULED IT (2026-09-19) — TWO LAYERS, EACH CORRECT AT ITS OWN JOB

> *"wat-edn needs to be a correct edn impl."*
> *"this is screaming wat's source code is not edn.. its code.... so.... we need another extension…
> let's just get our edn tooling corrected and wat's reader to be correct to handle macros and
> whatever."*

**The ruling, as the orchestrator reads it** (⚠ builder corrects if wrong):

| layer | is | must |
|---|---|---|
| **`wat-edn`** | the **EDN data** reader | be **spec-correct EDN**. NOT `clojure.edn` parity — `clojure.edn` is itself a superset (it reads quote, metadata, multi-slash). |
| **`wat-reader`** | the **wat code** reader | be a correct **Clojure-dialect** reader: EDN **plus** wat's reader macros. |

⛔ **This kills the ward's doctrine sentence as written.** *"wat must accept everything clj accepts"*
is the wrong obligation for `wat-edn`, because `clojure.edn` accepts things EDN does not. The ward
needs a **third category**: *"clj accepts, we deliberately refuse, because it is not EDN."*

### ⭐ RE-TRIAGE UNDER THE RULING — 6 bugs, 3 rulings, 2 we get RIGHT

| row | clj | wat-edn | verdict under "spec-correct EDN" |
|---|---|---|---|
| `9223372036854775808` | OK | ERR | ⛔ **BUG** — spec: integers are arbitrary precision |
| `123456789012345678901234567890` | OK | ERR | ⛔ **BUG** — same; `num-bigint` already a dep |
| `a:b` | OK | ERR | ⛔ **BUG** — *"`: #` are allowed as constituent characters… other than as the first"* |
| `a#b` | OK | ERR | ⛔ **BUG** — same sentence |
| `{:a 1 :a 2}` | ERR | **accepts** | ⛔ **BUG** — *"Each key should appear at most once"* |
| `#{1 1}` | ERR | **accepts** | ⛔ **BUG** — *"A set is a collection of unique values"* |
| `'x` | OK | ERR | ✅ **CORRECT** — quote is not EDN; it belongs to `wat-reader` |
| `^:m x` | OK | ERR | ✅ **CORRECT** — metadata is not EDN; `wat-reader`'s |
| `a/b/c` | OK | ERR | ⚠ **RULING** — spec: `/` *"can be used once only"*. clj is lenient. Strict ⇒ we are right. |
| `clojure.core//` | OK | ERR | ⚠ **RULING** — two `/`, but the name part IS `/`. Spec is genuinely ambiguous here. |
| `#inst "1985-04-12"` | OK | ERR | ⚠ **RULING** — spec says RFC-3339; a bare date is not a timestamp. clj promotes it. |
| `#myapp/Person {…}` | ERR | accepts | ✅ already exempted, with reason |

⭐ **Two rows flipped from "bug" to "correct" purely by the ruling.** That is the measure of how much
the doctrine question was worth: an executor guessing parity would have implemented quote and
metadata in the EDN layer and pushed `wat-edn` toward being a Clojure reader — the wrong direction.

⚠ **`a/b/c` matters beyond this crate.** The builder's first-slash ruling tolerates multi-slash names
in **wat source**. Under the two-layer split that is consistent: `wat-reader` tolerates them,
`wat-edn` (strict) refuses them, and wat source is **code, not EDN**, so nothing is violated.

## ⛔ NOT-NOW, RECORDED SO IT IS NOT LOST

- **The file extension.** `.wat` collides with **WebAssembly Text**. The builder was going to mass-
  rename to `.edn` and ruled against it on this measurement: *"wat's source code is not edn.. its
  code."* ⛔ **`.edn` would be a claim that is 2% true today and would still be wrong at 100%,** for
  the same reason Clojure ships `.clj` and not `.edn`. **A new extension is owed; it is deferred.**
  ⚠ Blast radius, measured: **1,274 hardcoded `.wat` occurrences across 307 `.rs` files**, 1,326
  `.wat` files referencing `.wat`, plus `extension() == Some("wat")` checks in at least 4 load-bearing
  places (`distribution/staleness.rs:138`, `host/test_runner.rs:585`, `wat-macros/discover.rs:265`).
- **Regex is COMING.** Builder: *"wat very limited regex support, it will be necessary later. wat is
  meant to be a general purpose lang, regex needs to be in there."* `#"…"` is **not EDN**
  (`No dispatch macro for: "`). ⛔ **Nobody may argue "wat source is EDN" as a durable property** —
  regex will break it by design. It is `wat-reader`'s, never `wat-edn`'s.
- **Ruled OUT:** `#(…)` anonymous-fn shorthand, and `#'` var-quote (*"i don't like what `#'` actually
  does"*).

## ⭐ WHAT WAT ACTUALLY USES THAT IS NOT EDN — measured, and it is ONE family

Every Clojure reader construct, tested against `clojure.edn`, then censused over the corpus
(comments stripped):

| construct | EDN? | wat files |
|---|---|---|
| `'x` quote · `^:m` meta · `#_` discard · `#{}` · `#inst` | ✅ EDN-OK | — |
| **`` ` `` `~` `~@`** quasiquote family | ❌ `Invalid leading character` | ⛔ **105 files (union)** |
| `@` deref | ❌ | **0** — all 34 hits were string literals (`"… @ {size}"`) |
| `#"…"` regex | ❌ | **0 today** — all 9 hits were string literals (`:sk-lo "#"`). **Coming.** |
| `#(…)` · `#'` | ❌ | **0** — and both ruled out |

⭐ **wat's ONLY non-EDN construct today is the quasiquote family, in 105 files.** Everything else the
language uses is already EDN. ⚠ The 105 is comment-stripped but **not** string-stripped; treat it as
a near-bound, and derive the real list from the reader, not from grep.

## ⛔ THE SUPERSEDED QUESTION (kept for the record)

Rows 8 and 9 are the problem. **`clojure.edn` reads `'x` and `^:m x`** — quote and metadata are
**Clojure reader features, not EDN**. The ward's doctrine says, verbatim:

> *"wat must accept everything clj accepts — a `clj:OK / wat:ERR` row is a wat bug."*

Read literally, that obligates `wat-edn` to grow **quote and metadata support**. That is not a bug
fix; it is a feature, and it moves `wat-edn` from "an EDN reader" toward "a Clojure reader" — which
is the opposite direction from the highlander, where `wat-edn` is meant to be the EDN core and the
reader-macro layer sits ABOVE it.

⛔ **THE BUILDER RULES THIS BEFORE THE STONE IS STRUCK.** Three coherent answers exist:
- **(a)** the obligation is to `clojure.edn` as-is → implement quote + metadata in `wat-edn`;
- **(b)** the obligation is to **EDN**, and `clojure.edn`'s reader-macro support is itself a superset
  → add a THIRD ward category ("clj superset we deliberately refuse") and exempt rows 8–9;
- **(c)** the ward's doctrine sentence is wrong and gets rewritten.

**An executor guessing here builds the wrong stone.** Rows 1–7 and the two duplicate-key bugs are
unaffected by the ruling and can proceed regardless.

## ⚠ AND IT REOPENS A RULING THE BUILDER ALREADY MADE

Row 4 matters beyond `wat-edn`. The orchestrator told the builder his multi-slash examples
(`u/f/g/a/g`, `a/b/c`) were *"not EDN"* and therefore outside his own stated compliance bar.
**`clojure.edn` accepts them.** The builder's instinct — *"pathological names are allowed"* — was
correct and the orchestrator's correction was wrong. The first-slash ruling
(`251/RULING-the-first-slash-separates-namespace-from-name.md`) is therefore **load-bearing after
all**, not moot: multi-slash names are readable, so *where* the split falls is a real decision.

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

⛔⛔ **THAT DOCTRINE SENTENCE IS NOW WRONG AND YOU MUST FIX IT — see the builder's ruling above.**
`clojure.edn` accepts **quote, metadata and multi-slash symbols**, none of which EDN sanctions, so
*"wat must accept everything clj accepts"* would drag the EDN layer toward being a Clojure reader.
**Rewrite the header to three categories**, then make the ward enforce them:

| category | meaning | example |
|---|---|---|
| `clj:OK / wat:OK`, `clj:ERR / wat:ERR` | parity | most rows |
| `clj:OK / wat:ERR` | **a wat bug**, UNLESS the construct is not EDN | `a:b` (bug) vs `'x` (not EDN) |
| `clj:ERR / wat:OK` | **a wat bug** — we accept invalid EDN — unless exempted with a reason | `{:a 1 :a 2}` (bug) vs unknown tag (exempt) |
| ⭐ **NEW: `clj:OK / wat:ERR` and NOT EDN** | **CORRECT.** clj's superset, deliberately refused. | `'x`, `^:m x` |

⛔ **The new category needs the same discipline as `exemption()`: each entry names the spec clause
that makes the construct non-EDN.** "It feels like Clojure" is not a reason. Without that rule the
third category becomes a place to hide real bugs.

**The ward's SHAPE is right — differential, golden-backed, runs without Clojure. The corpus is the
other hole:**

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

**2 — REGENERATE the golden against real `clojure.edn`.** ⭐ **`clj` is at `/usr/local/bin/clj`
(Clojure 1.12.4) ON THIS MACHINE — run it yourself.** The orchestrator asked the builder to run three
REPL lines it could have run itself; do not repeat that. `regen.clj` runs as documented. ⛔ **Do not hand-write a golden verdict. Ever.** The whole value of the
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
