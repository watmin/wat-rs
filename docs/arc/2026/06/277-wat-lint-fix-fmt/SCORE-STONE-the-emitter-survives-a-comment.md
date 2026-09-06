# SCORE — STONE: the emitter survives a comment

No commit. Floor and clippy left to the orchestrator. Defect E unfixed and reported (STOP-2). No Rust. No new record. No rule-file change.

A comment is a line. Emitting it must leave the emitter where a sibling break would have: the next node is still owed its indent. A blank line is `"\n"`, never indent + `"\n"`.

## Row 2 — ★★★ the node after a comment KEEPS its indent

`comment-indent.wat`, `IDEMPOTENT=true`, `--check` clean. Formatted:

```
;; a leading comment above the form
(:wat::core::defn :fix::cgo
  [x <- :wat::core::i64]
  -> :wat::core::i64
  ;; about the body
  (:wat::i64::+ x 1))

;; C trailing on the body
(:wat::core::defn :fix::cgo2
  []
  -> :wat::core::nil
  nil)
```

`(:wat::i64::+ x 1)` at indent **2**, not 0. Leading comment at column 0. Body comment at indent 2 (the form it sits above). `cgo2` at column 0.

## Row 3 — ★★★ `wat/spawn.wat` variants stay at indent 2, tags still padded

Real file, not a fixture. `FORMS=35 COMMENTS=429 IDEMPOTENT=true`. ServiceEvent:

```
(:wat::core::defenum :wat::spawn::ServiceEvent :- [I O A] :wat::enum::Impure
  :Shutdown   []
  ;; owner dropped the handle (self-peer drained) — exit; deadlock-free termination
  :Admin      [msg <- :A]
  ;; owner sent an admin op over the lineage peer (Ok path); A = self-peer's recv type
  :Connection [peer <- (:wat::kernel::Peer :- [I O])]
  :Message    [idx <- :wat::core::i64
               msg <- :O]
  :Closed     [idx <- :wat::core::i64]
  :Lost       [idx   <- :wat::core::i64
               cause <- :wat::kernel::Failure]
  :Malformed  [idx   <- :wat::core::i64
               cause <- :wat::kernel::Failure]
  ;; arc 278: peer ALIVE, message undecodable — reply cause + keep serving
  :Rejected   [idx   <- :wat::core::i64
               cause <- :wat::kernel::Failure])
```

Every variant at indent 2. Tags padded (`:Shutdown` / `:Admin` / `:Connection`). `:Shutdown []` still inserted. Trailing comments lifted — see row 10.

## Row 4 — ★★ blank lines carry NO trailing whitespace

`grep -c ' $'` on formatted output = **0** for comment-indent, spawn, io, deporder, let-complex, defn-uneven, and every fixture.

Spawn had 16 lines that ended in a space: the inter-token `" "` (or an AlignPairs pad) sitting on the line a trailing comment had been lifted off. `rstrip-ws` inside `ensure-nl` drops those spaces when a newline is actually emitted. Pad that a riding value still consumes is left alone.

## Row 5 — ★ no blank between ret-spec and body

`defn-uneven.wat` and `comment-indent.wat`: `->` then the next line is the body (or the body's leading comment), never a blank.

## Row 6 — ★ exactly one blank after each top-level form

Inserted if absent, left if present, extras trimmed. comment-indent: one blank between `cgo` and `cgo2`, one blank after the last form. Does not accumulate (row 8).

## Row 7 — ★★★ a blank INSIDE a body is PRESERVED

`let-complex.wat`, `IDEMPOTENT=true`:

```
    [mapped (:wat::core::foldl
              …
              xs)

     j (:wat::i64::+ 40 2)]
```

The `BlankBefore` gap between `mapped` and `j` is still there. Rule 2 forbade one position (ret-spec → body), not the 869 in-form blanks.

## Row 8 — ★★ idempotent

Every fixture `IDEMPOTENT=true`. comment-indent, spawn, io, deporder included. The required blank does not become two on pass 2.

## Row 9 — ★★★ no comment is LOST

Reader counts on **source and formatted output**:

| file | source | formatted |
|---|---|---|
| `wat/io.wat` | **28** | **28** |
| `wat/deporder.wat` | **85** | **85** |
| `wat/spawn.wat` | 429 | 429 |
| `comment-indent.wat` | 3 | 3 |

## Row 10 — ★★ REPORTED, not fixed — every TRAILING comment

All still emitted as a **leading** comment of the next node. None stayed trailing on a surviving code line. Not chosen; the pre-existing lift.

**Fixture** `comment-indent.wat`:
- `;; C trailing on the body` was after `))` on the body line → own line at column 0 above `:fix::cgo2`.

**`wat/spawn.wat` — 6 trailing-on-line comments:**

| was | became |
|---|---|
| `:Shutdown … ;; owner dropped the handle…` | own line, indent 2, immediately above `:Admin` |
| `:Admin … ;; owner sent an admin op…` | own line, indent 2, immediately above `:Connection` |
| `:Malformed … ;; arc 278: peer ALIVE…` | own line, indent 2, immediately above `:Rejected` |
| `:Rejected … ;; arc 278 Stone 1a: over-FOO…` | own line, column 0, now a leading comment of the next top-level form (`PoolMsg`) |
| `SendOutcome::Closed nil)   ;; parent's recv' already faces this` | own line at the match indent, above the following arm |
| `SendOutcome::Closed nil)   ;; the recv' below already faces this` | same |

`:Connection` / `:Message` / `:Closed` / `:Lost` had no trailing comment and stayed on their lines. Comments that were already their own line take the following form's indent (rule 1).

## Row 11 — ruled shapes hold

defenum-mixed: tag+vector one unit, tags padded, `[]` inserted, not duplicated. defrecord / defstruct `<-` aligned. defn-uneven `<-` aligned. atom-map `{ :a 3 :b 42}` INLINE. kwargs-pos keys aligned. kw-table still three one-line `Node`s. Every existing fixture `IDEMPOTENT=true`. `--check` clean on all fixtures including `comment-indent.wat`.

## Row 12 — the 614 doc examples

Isolation (`run-examples.wat`). **N=614 CHANGED=614 INLINE=0 OVER120=0 WORST=104**.

Over-120 still **0**, worst still **104**. CHANGED=614 / INLINE=0 is rule 3: every isolated form now ends with the required trailing blank, so a source one-liner is never byte-identical, and the INLINE detector (`nlines==1` or `nlines==2` with an empty second) treats `"…\n\n"` as three split parts. The code line still rides: `(probe-form 1 2)` stays one line plus the blank.

## Row 13 — three walls stand

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Deleted after. `ClaimedUnder` **0**. `grep -c 'col'` / `'120'` over `rules/*.wat` **0** (grep exits 1). `120` still once in `wat/fmt.wat` (emitter width budget).

## Row 14 — wat-scripts load

`every_wat_scripts_file_loads_on_the_current_runtime` **1 passed**.

## Mechanism

- `pending-indent?` / `drop-pending-indent` / `open-comment-line` — a comment opens a fresh line without leaving spaces on the previous one (defect D).
- `flush-comments` takes `restore`. Pre-node (`true`): after the comment, rewrite `spaces(indent)` so the node is owed its column (defects F and G). Post-node (`false`): do not choose E.
- `parent-id == 0` forces `this-indent` 0 and drops leftover pending indent (a lifted trailing comment must not indent the next top-level form).
- `ensure-blank` / `trim-extra-nl` — exactly one blank (`\n\n`) after each top-level form, including the last. Insert / leave / trim. Never spaces.
- `rstrip-ws` inside `ensure-nl` — a newline after pad or the inter-token space does not leave trailing whitespace.

No rule named a column. `apply-blank` still writes a structural blank inside a body (row 7).

---

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| comment-indent | body at indent 2, **IDEMPOTENT=true**, `--check` clean |
| `wat/spawn.wat` | variants indent 2, tags padded, **COMMENTS=429** both sides |
| `grep -c ' $'` on formatted outputs | **0** |
| let-complex | in-body blank preserved, **IDEMPOTENT=true** |
| defn-uneven | no blank between `->` and `nil` |
| `run.wat` / formatted `wat/io.wat` | **COMMENTS=28** source and output |
| `wat/deporder.wat` | **COMMENTS=85** source and output |
| every fixture | ruled + **IDEMPOTENT=true** + `--check` clean |
| 614 doc examples | **0** over 120, worst 104 |
| kind-conflict sabotage | **raises** `block vs align`, then deleted |
| `grep -c 'col'` / `'120'` over rules | **0** / **0** |
| `ClaimedUnder` | **0** |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED. No edit.** Every row holds under an independent re-run, and the three faults the previous
SCORE showed on `wat/spawn.wat` — the de-indent to column 2, the lost tag padding, the promoted
comment — are gone.

| what | my own re-run |
|---|---|
| ★★★ row 2 · node after a comment keeps its indent | `(:wat::i64::+ x 1))` at **2** · `IDEMPOTENT=true` |
| ★★★ row 3 · `wat/spawn.wat`, the REAL FILE | every variant at indent **2** · tags padded · `:Shutdown []` still inserted |
| ★★ row 4 · no trailing whitespace | `grep -c ' $'` on the output = **0** |
| ★★★ row 9 · no comment LOST | spawn **429→429** · deporder **85→85** · io **28→28** |
| row 7 · an in-body blank survives | validated predicate (pos + neg control): spawn interior blanks are `let-blank`'s, not a purge |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** |
| clippy | **0** under `-D warnings --all-targets` |

## ⛔ AND THE THING NO ROW ASKED — the corpus's WIDTH has collapsed

`[[NOTE-the-formatter-had-never-met-a-real-file]]` recorded, earlier in this same arc:

```
deporder max line 306 -> 163        over 120:  3 -> 2
```

Measured at HEAD, by an in-wat census (`wat-scripts/scratch-pad/277-file-width-census.wat`) that
weighs the emitter's own output string rather than a shell decode of it:

```
             SOURCE                    FORMATTED
deporder     over120=3   worst=306     over120=259  worst=420
spawn        over120=19  worst=227     over120=175  worst=265
grep         over120=1   worst=121     over120=66   worst=226
io           over120=1   worst=122     over120=0    worst=104   ← the shallow file still IMPROVES
```

**The formatter now makes three of four real files far worse than it found them**, against the ruled
120. `wat/io.wat` — the one real file this arc had measured before the NOTE — is 45 lines of
one-liners, which is exactly why it could not show this.

### Attribution — measured, not assumed

⚠ **`wat/*.wat` is FROZEN into the release binary at build time.** Editing the emitter and re-running
changes nothing; the swap must be followed by `cargo build --release`. A first attribution attempt
here swapped `wat/fmt.wat` without rebuilding and reported "identical" — a green from a probe that
never fired. Two earlier sabotages were also self-consistent (a rename applied to definition *and*
call sites; a `"block"`→`"align"` flip applied globally, aimed at a file whose one-liners cannot show
it). The test that settled it doubles the indent unit — a change every indented line must show.

Rebuilt against the pre-stone emitter (`e4f1cd817:wat/fmt.wat`):

| file | pre-stone | HEAD | this stone's delta |
|---|---|---|---|
| deporder | over120=**239** worst=420 | over120=259 worst=420 | **+20**, worst unchanged |
| spawn | over120=**131** worst=265 | over120=175 worst=265 | **+44**, worst unchanged |
| grep | over120=**56** worst=219 | over120=66 worst=226 | **+10**, worst +7 |

**The collapse predates this stone.** It did not create a single one of the worst lines. Its own
delta is comment-shaped — over-120 lines carrying a `;;` number **19 / 66 / 5** in the three files —
which is defect E doing it: a lifted trailing comment becomes its own full-width line at the node's
indent. **The brief forbade choosing E (STOP-2), so this delta is the brief's, not the strike's.**

★ **Defect E is therefore not cosmetic.** It was carried as an attachment-policy question; it is also
a width cost, and it is the one open item that pays for itself twice.

## ⚠ AND ONE SHAPE CHANGE THAT LANDS ON THE 255 PATH

An isolated one-line example now returns with a **trailing blank line**:

```
(:wat::core::mapv …)   →   "(:wat::core::mapv\n  …\n  (:wat::core::Vector 1 2 3))\n\n"
```

Rule 3 applies "one blank after each top-level form" to the **last** form too. That is the whole of
the `CHANGED 330→614 · INLINE 284→0` shift this SCORE reports as arithmetic. The 614 examples are
still **0 over 120**, so the priority path is not blocked — but `crates/wat-doc/src/print.rs` would
embed that blank in every `:examples` entry, and **whether the ruling reaches the last top-level form
is the builder's**, not the strike's.

Corpus evidence, for blast radius only and not as authority: **130** tracked `.wat` files end on a
blank line, **1689** on a single newline (predicate validated against a positive and a negative
control).

## The board this leaves

```
⛔ WIDTH on real files                the formatter makes the corpus WORSE; nothing backs off
   E · where a TRAILING comment goes  now ALSO a width cost — 19/66/5 over-120 comment lines
   rule 3 at the LAST top-level form  a trailing blank on every isolated example — 255's consumer
   level-2 alignment inside a value   open since the kwargs stone
   arc 109 · bare variant ILLEGAL     the formatter is its migration
✅ the emitter survives a comment      spawn.wat's three faults are GONE
✅ the arc-255 doc migration           0 over 120, worst 104
```
