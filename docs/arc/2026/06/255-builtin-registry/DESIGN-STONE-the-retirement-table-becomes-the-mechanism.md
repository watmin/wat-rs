# DESIGN — STONE: the retirement table becomes the mechanism it looks like

> **Builder ruling, 2026-08-25:** *"fix the retirement table too"*
>
> Census, mechanism and the four-questions comparison:
> `255/NOTE-the-retirement-table-is-inert-for-half-its-rows.md`.

## THE THESIS

`RETIREMENT_TABLE` looks like a lookup the substrate performs. It is not. **It is a lookup that
thirteen hand-written arms perform, and the table is the data they happen to share.** Adding a row
without an arm adds nothing — which is exactly what the four-homes stone did, in good faith, ten
times.

## THE CENSUS — every row RUN, not grepped

All 35 `retired:` names invoked in head position against the built binary:

```
13   retirement message      every one a BARE `:wat::core::<word>`
13   ⛔ bare UnknownFunction  every one carrying a `/` or a further `::` in its leaf
 7   TypeMismatch, diagnosed  vec · list · tuple · Some · Ok · Err · :None
```

★ **The middle column is the whole stone**, and its membership is a derivation rather than a list:
*a row fires iff its name is bare.* A bare name is caught by a hand-written arm — `check.rs:955`'s
`if s == ":wat::core::Char"`, or one of `infer_list`'s — and those arms call `remedies_for`, the only
production caller of `retirement_lookup`. A slash-form or nested name reaches none of them, falls
through to `check.rs:5628` which **silently accepts** (*"HARVEST (236.2): silent-by-intent"*), and
dies at runtime as a bare `UnknownFunction` on a path that never consults the table either.

⚠ **The right column is NOT a gap** — corrected from the note's first draft, which called those seven
"an artifact of my probe's shape". Measured: they produce `TypeMismatch`, and at least `tuple`'s
message already names its own retirement (*"the comma dies in the reader"*). They are diagnosed by a
third path. Nothing to do; recorded so nobody re-opens them.

**Three of the thirteen predate the four-homes stone:** `:wat::core::Record::def`,
`:wat::core::to-struct`, `:wat::holon::Record::def`.

## THE TWO DOORS

**Door 1 — CHECK TIME, and it is the primary one.** `src/check.rs:5628`, the silent-accept
fallback: a `:wat::`-prefixed callee with no registered scheme is accepted and passed through.
Consult `retirement_lookup(k)` before accepting; a hit becomes a located `MalformedForm` carrying
`remedies_for(k, …)` — **byte-identical in shape to what the working thirteen already deliver**. A
retired name should be diagnosed where the rest are, not survive to runtime.

**Door 2 — RUNTIME.** The `RuntimeErrorKind::UnknownFunction` construction sites. A dynamically
built head (`eval-ast!`, `keyword/from-string`) never passes the checker, so door 1 alone leaves a
hole.

### ⛔ THE ONE PINNED CONTRACT — door 2 improves the MESSAGE, and does NOT widen the type

`RuntimeErrorKind::UnknownFunction(String)` is a **tuple variant carrying only the path**
(`src/value/signal.rs:195`). It cannot hold a structured `:remedies` list without changing the
variant, and changing it is a question about the error type's shape that belongs to `conformare`,
not here. So door 2 folds the replacement into the message text and stops:

```
unknown function: :wat::core::Uuid/v4 — ':wat::core::Uuid/v4' is retired; use ':wat::uuid::v4' instead
```

Door 1 delivers the structured remedy. Door 2 delivers the sentence. **Do not widen the variant to
make them symmetric** — that is a different stone and it would drag every `UnknownFunction` call site
with it.

## ★ THE GATE — over the TABLE, never over a copy of it

This is the part that makes the stone hold, and it is the lesson the table itself failed to learn:

> **A test that iterates `RETIREMENT_TABLE` and, for every row, asserts the substrate diagnoses that
> name — naming its replacement.**

Not a hand-list of names to check. Not a count. The table, walked.
`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`

**It must be end-to-end, driving the real binary** — not an in-process `check_program` call. The
thirteen pass the checker silently today and fail only at runtime; a check-only gate would report
them green. `tests/cli/wat_grep.rs` is the pattern.

**And its assertion is the negative, so it needs no exemption list:** for each row, the outcome must
not be *a bare `UnknownFunction` with no replacement named*. A retirement message passes. A
`TypeMismatch` that names the replacement passes. That admits the seven without special-casing them,
and it is exactly the defect stated as a property.

★ **Write the gate FIRST.** It goes red on thirteen rows today and names every one. That is the
worklist — `docs/SUBSTRATE-AS-TEACHER.md`, and a gate that has never been red is a claim.

## THE FOUR QUESTIONS

- **Obvious?** YES — the table says "this name is retired, here is its replacement"; after this, that
  is what the substrate says too.
- **Simple?** YES — two call sites and one gate. No new type, no new table, no per-name code.
- **Honest?** YES. Today the table is a document that *looks* like a mechanism, and the four-homes
  stone was misled by it in good faith. This makes the appearance true.
- **Good UX?** YES — every retired name, past and future, names its replacement without anyone
  remembering to add an arm.

## ACCEPTANCE — bars derived this session on a freshly-built binary

1. **The gate exists, walks `RETIREMENT_TABLE`, and is RED before the fix on exactly 13 rows** — the
   thirteen named in the census. If it is red on a different number, the census moved and that
   difference is the finding.
2. **After the fix it is green on all 35**, and each of the thirteen names its replacement.
3. **`(:wat::core::Uuid/v4)` names `:wat::uuid::v4`.** Measured at HEAD: bare `UnknownFunction`.
   This is four-homes' unmet row 1b, finally reachable.
4. **The three pre-existing inert rows also close** — `Record::def`, `to-struct`,
   `holon::Record::def`. They are the proof this is a substrate fix and not ten more arms.
5. **The working thirteen are unchanged.** Their arms already deliver a `MalformedForm` with
   remedies; door 1 must not double-report or change their message. A diff in their output is a
   regression.
6. **The seven `TypeMismatch` rows are unchanged.**
7. Floor green **accounted BY NAME** (baseline 5056/5056, 19 skipped); clippy 0.

## OUT OF SCOPE — affirmatively cut

- **Widening `RuntimeErrorKind::UnknownFunction` to carry remedies.** Pinned above.
- **The bare-alias rows** (`Some`/`Ok`/`Err`/`:None`). They are diagnosed, and the reason they read
  oddly is R9's unfinished anneal — 6346 sites still on a bridge — which is `296/STONE-H`'s, not this.
- **Auditing whether every row's `replacement` is still correct.** This stone makes the table
  *reachable*; whether each row is *true* is a separate audit, and rows written years apart deserve
  it.
