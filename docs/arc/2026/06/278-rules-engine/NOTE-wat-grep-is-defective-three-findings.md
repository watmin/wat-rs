# NOTE — wat-grep is defective: three findings, and the third explains the other two

> Builder, 2026-08-25: *"is wat-grep defective?… if yes… i want that more than anything else right now."*
> **Answer: yes.** Audited this session against the disk. Below is what I checked, what I found, and
> what I checked that turned out to be FINE — because a defect list without its negative space is
> just a list of the things I happened to look at.

---

## ⛔ FINDING 1 — a file that READS but does not PARSE is silently empty, and the cause is discarded

`wat/grep.wat:218`:

```wat
((:wat::core::ReadOutcome::Malformed __cause) (:wat::grep::empty-acc))
```

The parse error is **in hand** and thrown into an unused binding. A malformed file yields an empty
fact base, so every rule sees nothing, so the driver prints nothing, and the process exits 0.

**Positive control — same content, one balanced and one not:**

```
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::Uuid/v4)))      ->  1 match
  (:wat::kernel::println (:wat::core::Uuid/v4)        ->  0 matches, EMPTY OUTPUT, EXIT 0
```

And the information is fully available one call away — `wat --check` on the same bytes says
`#wat.parse/UnclosedParen … :line 2 :col 3`.

**So wat-grep cannot distinguish *"this file has no matches"* from *"I could not read this file."***
Over a 1567-file corpus that is unfalsifiable absence — the exact class this repo has been paying for
all session (`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`,
`[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`). **Every census run through wat-grep
so far, mine included, has an unknown and unknowable denominator.**

The comment above that line calls it *"the no-hidden-failures law"*. It is the opposite: the failure
is hidden from the rule AND from the user.

## ⛔ FINDING 2 — `Named` + `Span` cannot say "this name is WRITTEN here"

Both facts are true. They answer different questions, and nothing in the fact base says so. A rule
that joins them is implicitly claiming *the name is spelled at that span* — and for a
reader-synthesized node it is not.

Post-arc-300-stone-D the population is **1411** nodes: `:wat::core::unquote` 980, `quasiquote` 310,
`unquote-splicing` 119, `:wat::holon::literal` 2. (`char/of`'s 50 are GONE — stone D deleted them at
the source rather than guarding them.)

For a **querying** rule this is arguably correct: `~` *is* an unquote, and asking "where is unquote
used" should find it. For a **rewriting** rule it is silent source corruption — measured at 50 sites
before stone D, one of them a test asserting two different chars unequal, which would have been
*inverted* rather than broken. See `255/NOTE-a-name-the-reader-manufactured-has-no-text-to-rewrite.md`.

★ **The predicate is EXACT, not a heuristic** — checked this session, and it is what makes the fix
cheap: `:wat::core::ast-name` returns **verbatim token text**, not a normalized name. Measured on the
faithful-Clojure surface, where a normalization would have shown up:

```
(:wat::core::ast-name <head of (wat.core/if true 1 2)>)  ->  "wat.core/if"
```

So for any single-line named node, `end-col − col == length(name)` **iff** the name is written at that
span. Empirically 172/172 genuine matches satisfied it and 50/50 phantoms violated it; now it is
grounded in the mechanism as well as the sample.

## ⛔ FINDING 3 — wat-grep has ZERO tests, and that is why 1 and 2 shipped

Measured:

```
tests naming :wat::grep::*                        0
tests exercising `--grep` mode                     0
assertions in wat-scripts/scratch-pad/probe-grep-*  0   (they RUN; they check nothing)
runs of the file's OWN declared control             0   ("★ NON-VACUITY: Span == Node")
```

`wat/grep.wat` is a shipped stdlib file behind a shipped CLI mode, and the only thing standing over
it is the loader gate — which proves it **type-checks**, not that it **works**. Its header declares a
non-vacuity control in prose and nothing has ever run it.

R59 `NISI FRANGAS, NIHIL PROBAS`. wat-grep has no numbers at all, so there was nothing to break.
`[[feedback_impose_the_check_and_read_the_screams]]` — the check was never imposed.

---

## ✅ WHAT I CHECKED THAT IS **NOT** DEFECTIVE

Recorded so this note is a measurement and not a mood:

- **Coverage is COMPLETE.** `eval_ast_kind` (`src/edn_shim.rs:1011`) is exhaustive over all 14
  `WatAST` variants with **no catch-all arm**; `structural?` names exactly the 4 kinds that have
  children (list/vector/map/set) and `nameable?` exactly the 3 that have names
  (symbol/keyword/string). Verified by enumerating both sides, not by reading the comment. **No node
  is invisible to the walk and no subtree is skipped.**
- **A missing file or a directory raises LOUDLY** — `:wat::io::read-file` surfaces a
  `MalformedForm`. Only the malformed *parse* is silent. Finding 1 is narrow, not general.
- **The pre-order walk's parent numbering is sound** — a child's parent is always already assigned,
  as the header claims.
- **`Span` is emitted unconditionally beside `Node`**, as the header claims; the guard is on `Named`
  only, and the absence genuinely is the guard.

## ⚠ AND ONE SELF-DECLARED WEAKNESS THE FILE ADMITS

`wat/grep.wat:47` — *"this field list (line/col/end-line/end-col) must stay in lockstep with
`:wat::grep::Extent`'s four fields below — **nothing pins them together**, so a rename of one must be
made in both by hand."* That is the convention rung, written down as a known weakness rather than
closed. Not a defect today; a defect the day someone renames a field.

---

## THE SHAPE OF THE FIX (for the builder's ruling — not yet drawn as a stone)

Findings 1 and 2 are the same defect wearing two coats: **the fact base cannot express something the
consumer needs, so silence gets read as an answer.**

- **F1 →** a `:wat::grep::Unreadable {file, reason, line, col}` fact, so a rule can SEE it — *and* the
  driver surfaces it unconditionally, because a user who does not join the fact would otherwise get
  the same silence back. wat-grep owns the print; this is its own job, not the rule's.
- **F2 →** a `:wat::grep::Written {id}` fact, emitted only when `end-col − col == length(name)` on a
  single-line named node. `Named` keeps its true meaning (*what this node is called*); `Written` means
  *and it is spelled here*. A rewriting rule joins `Written`; a querying rule does not.
- **F3 →** the gates that make both falsifiable, including the file's own `Span == Node` control,
  which has never run.

⚠ **F1 and F2 share a failure mode and it must be designed against:** an opt-in fact does nothing for
the consumer who does not know to opt in. That is why F1's answer is *fact **and** unconditional
print*, and it is the open question on F2 — whether `Written` is enough, or whether the codemod path
must be unable to join `Span` without it.

**Scope is the builder's call.** F3 is the root and F1/F2 are unverifiable without it, so the honest
minimum is all three together — but the three are separable if that is too much at once.
