# DESIGN — STONE: wat-grep never lies

> **Builder ruling, 2026-08-25:** *"fix all three - wat-grep must never lie again"*
>
> Findings and their measurements: `278/NOTE-wat-grep-is-defective-three-findings.md`.

## THE THESIS

All three findings are one sentence: **wat-grep's fact base cannot express something its consumer
needs, so silence gets read as an answer.** F1 is silence about a file it could not read; F2 is
silence about whether a name is written where it says; F3 is why neither was ever caught.

The stone gives the fact base the two facts it was missing and the gates that make both falsifiable.

---

## F1 — an unreadable file must be LOUD

`wat/grep.wat:218` binds a parse error to `__cause` and returns an empty fact base:

```wat
((:wat::core::ReadOutcome::Malformed __cause) (:wat::grep::empty-acc))
```

★ **The house already knows how to do this.** `wat/fix.wat:351` — the SAME `ReadOutcome` match, one
file over — **raises** with `(:wat::core::Error/message __cause)`. Two files in one subsystem, one
honest and one not. The fix is not an invention; it is applying the sibling's own answer.

### The form

```wat
(:wat::core::defrecord :wat::grep::Unreadable
  [file   <- :wat::core::String
   reason <- :wat::core::String
   line   <- :wat::core::i64
   col    <- :wat::core::i64])
```

- `Facts` gains `unreadable <- (:wat::core::PersistentVector :- [:wat::grep::Unreadable])`, holding
  zero or one. `facts-as-records` conjs it, so **a rule can join it** and reason about coverage.
- `run-one` prints it to **stderr**, unconditionally, whether or not any rule joined it.

⚠ **Both halves are load-bearing and the reason is F1's own lesson.** An opt-in fact does nothing for
the consumer who does not know to opt in — which is exactly how today's silence works. The fact
serves the rule author; the unconditional print serves everyone else.

### ⛔ THE ONE PINNED CONTRACT — the run reports EVERY bad file, then EXITS NON-ZERO

```
for each path:  unreadable → emit the fact, print to stderr, keep going
at the end:     if any file was unreadable → raise, so the process exits non-zero
```

**Deliberate divergence from `wat/fix.wat`, which raises immediately** — and the divergence is the
job, not an inconsistency. `fix.wat` is an APPLIER: a partial migration is worse than none, so it
must stop at the first bad file. wat-grep is a FINDER: stopping at the first bad file in a 1567-file
corpus hides the other 1566 answers. A finder's honest shape is *report everything, then fail.*

Exit non-zero is not decoration. Per the IPC triangle (`COMPACTION-AMNESIA-RECOVERY.md` § 13) the
exit code tells the parent which channel holds the result, and **a run that skipped files did not
fulfil its contract.** A zero exit on an incomplete census is precisely the lie this stone is named
for — and FM 20 is the record of a green-read exit code costing this project a day.

★ The cost is real and is accepted: a path list containing a `.wat.bad` fixture now fails loudly
instead of silently under-counting. That is the correct trade — the fixture was never grep-able, and
today the run says it was.

---

## F2 — `Written` is the fact that means "this name is SPELLED here"

`Named` and `Span` are both true and answer different questions. A rule joining them silently claims
*the name is spelled at that span*, which is false for the **1411** reader-synthesized nodes
(`unquote` 980, `quasiquote` 310, `unquote-splicing` 119, `holon::literal` 2).

### The predicate is EXACT, and that is what makes this cheap

`:wat::core::ast-name` returns **verbatim token text**, not a normalized name — measured on the
faithful-Clojure surface, where a normalization would have shown:

```
(:wat::core::ast-name <head of (wat.core/if true 1 2)>)  ->  "wat.core/if"
```

So for a single-line named node, `end-col − col == length(name)` **iff** the name is written at that
span. Not a heuristic: 172/172 genuine and 50/50 phantom, and now grounded in the mechanism too.

### The form

```wat
;; ONLY when the span holds exactly this node's own name — the fact a REWRITING rule joins.
(:wat::core::defrecord :wat::grep::Written
  [id       <- :wat::core::i64
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])
```

★ **It carries the coordinates rather than just `{id}`, and that is the design.** A rewriting rule
then joins **one** fact and never touches `Span` at all — the right path is the shorter one, so the
wrong path stops being the convenient one. A `{id}`-only marker would have left `Span` as the
obvious source of coordinates and `Written` as an extra chore.

`Named` is unchanged (a `~` **is** an unquote, and a querying rule should still find it). `Span` is
unchanged, so `Span == Node` survives as the non-vacuity control.

### ⚠ WHAT THIS STONE DOES **NOT** DO — named, measured, and not deferred

`Written` makes the corruption **expressible-against**; it does not make it **impossible**. A rule
that joins `Span` and rewrites still corrupts. The wall for that lives one component over, in the
APPLIER: `wat/fix.wat`'s `fix-text-apply` receives `(offset, old-len, new-text)` — it knows the old
**length** but not the old **text**, so it cannot verify that what it is about to overwrite is what
the rule meant. Carry the old TEXT and it can, and then no codemod can corrupt silently whether or
not it knows `Written` exists.

**That is the top rung and it is a different stone: 43 files call `fix-text-apply`** (of 68 recorded
codemods). It is wat-fix's, not wat-grep's, and it is written down here so it is a work item rather
than a mental note. This stone's claim is exactly its title — **wat-grep** never lies.

---

## F3 — the gates, because nothing has ever run

Measured: **0** tests name `:wat::grep::*`, **0** exercise `--grep`, the two scratch probes contain
**0** assertions, and the file's own declared control (*"★ NON-VACUITY: Span == Node"*) has never
run. The loader gate proves `wat/grep.wat` **type-checks**, not that it **works**.

CLI modes are tested by spawning the real binary (`tests/cli/wat_cli.rs`:
`Command::new(env!("CARGO_BIN_EXE_wat"))`, asserting stdout / stderr / exit code). That harness is
the shape.

| # | gate | why it must exist |
|---|---|---|
| G1 | `Span` count **==** `Node` count on a real corpus file | the file's own declared control, never run |
| G2 | `Named` count **<** `Node` count on that file | the other half of it |
| G3 | a malformed file yields an `Unreadable` fact, a stderr line naming the file AND the parse reason, and a **non-zero exit** | F1 |
| G4 | **the positive control for G3** — the SAME content, balanced, yields no `Unreadable`, empty stderr, exit 0 | without it G3 passes on a broken build that calls everything unreadable |
| G5 | on a file containing `~`, `Written` count **<** `Named` count | F2's non-vacuity: the phantom class must actually be present |
| G6 | on a file with no reader macros, `Written` count **==** `Named` count | F2's negative control: `Written` must not be systematically under-emitting |
| G7 | `--grep` end-to-end through the real binary: a rule over a fixture prints the expected `Match` | the mode itself has never been exercised |

★ **G4 and G6 are the ones that would have caught today's bugs**, and they are the two a rider is
most likely to skip because they assert that nothing happened. They are not optional.

---

## THE FOUR QUESTIONS

- **Obvious?** YES — three facts for three questions: *what is this node called* (`Named`), *where is
  it* (`Span`), *is its name spelled there* (`Written`); and a file it could not read says so.
- **Simple?** YES — two records and a predicate. No existing fact changes meaning; both controls
  survive.
- **Honest?** YES — this is the entire point. Every silence that could be mistaken for an answer
  becomes a fact and a line on stderr.
- **Good UX?** YES — a rewriting rule joins one fact instead of two, and a census can no longer
  under-count without saying so.

---

## ACCEPTANCE — bars derived this session on a freshly-built binary

1. **The malformed-vs-balanced control flips.** Measured at HEAD: same content, balanced → 1 match;
   unbalanced → 0 matches, **empty output, exit 0**. After: unbalanced → an `Unreadable` fact, a
   stderr line naming the file and the reason, **non-zero exit**; balanced → unchanged.
2. **The stderr line names the parse reason**, not just the file. `wat --check` on those bytes says
   `#wat.parse/UnclosedParen … :line 2 :col 3`; the cause is already bound at `grep.wat:218` and is
   currently thrown away.
3. **`Written` emitted for 1411 fewer nodes than `Named`** on the corpus — the exact population the
   span-disagreement probe reports today (`probe-span-narrower-than-name.wat`, measured 1411 after
   arc 300 stone D).
4. **G1–G7 all present and all passing**, each a real test in the floor.
5. **The existing corpus still works** — `wat-scripts/grep/`'s five programs and
   `wat-scripts/fixes/rename-core-string-to-string.wat` are unchanged and still run. `Named` and
   `Span` keep their meaning; nothing that joins them today breaks.
6. Floor green **accounted BY NAME** (baseline 5046/5046, 19 skipped); clippy 0 under `-D warnings`.

## OUT OF SCOPE — affirmatively cut

- **`fix-text-apply`'s old-text verification** — the top rung, 43 call sites, wat-fix's stone. Named
  above with its measurement.
- **The `Span`/`Extent` field-lockstep pin** (`grep.wat:47`, *"nothing pins them together"*). A
  convention-rung weakness the file admits; not a defect today, and closing it is a separate question
  about whether `Extent` should exist at all now that `Written` carries coordinates.
- **Retro-fitting `Written` into the existing `wat-scripts/fixes/` corpus.** Those 68 codemods have
  run and their migrations are recorded history. New codemods join `Written`; old ones are not
  rewritten to prove a point.
