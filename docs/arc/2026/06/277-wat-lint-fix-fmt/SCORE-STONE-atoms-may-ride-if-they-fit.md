# SCORE — STONE: atoms may ride if they fit

No commit. Floor and clippy left to the orchestrator. Level-2 value alignment still open.

The rule asserts `AllAtoms`. The emitter exercises `indent + Width <= 120`. Width is a walk fact, not a rete derivation. `120` appears in `wat/fmt.wat` once. No rule names a column or a budget.

## Row 2 — atom map INLINE

`atom-map.wat`, `IDEMPOTENT=true`:

```
{:a 3 :b 42}
```

Width 12. At body indent 2: 14 ≤ 120.

## Row 3 — compound value: one pair per line

`compound-map.wat`, `IDEMPOTENT=true`:

```
{:a (:wat::i64::+ 1 2)
  :b 42}
```

First pair rides `{`. The list value does not tear onto its own line. List pair-run prefix rules are now kind=list, so they do not treat a map's second key as a "run start".

## Row 4 — ★★★ WIDTH PATH, shown firing

`long-map.wat` one-line width **151**. Body indent 2 → 153 > 120. Exploded, `IDEMPOTENT=true`:

```
{:k00 0
  :k01 1
  …
  :k19 19}
```

Rows 2 and 5 pass with no width consulted. Only this row proves the budget is read.

## Row 5 — defect 4, answered

`kwargs-one.wat`: `(:wat::fmt::BlankBefore :id id)` **INLINE**. `IDEMPOTENT=true`.

All-atom `Unreadable` (`kwargs-none`) also inlines — same class.

## Row 6 — compound Unreadable still explodes

`unreadable-compound.wat`, `IDEMPOTENT=true`:

```
(:wat::grep::Unreadable
  :file   p
  :reason (:wat::string::concat p "!")
  :line   0
  :col    0)
```

## Row 7 — width CONTROL (CHECKED beside MISMATCH)

Walk width vs span on single-line forms. Synthesized nodes (`Named` and not `Written`) and their ancestors are excluded. `wat/service.wat:2949` `~handle-record` is **not** in the mismatch list.

| file | CHECKED | MISMATCH |
|---|---|---|
| `wat/io.wat` | **73** | **0** |
| `wat/rete.wat` | 690 | 0 |
| `wat/grep.wat` | 991 | 3 |
| `wat/fix.wat` | 4020 | 9 |
| `wat/core.wat` | 4670 | 6 |
| `wat/service.wat` | 4560 | 24 |

CHECKED is printed beside MISMATCH on every file (not a vacuous 0/0). Residue is extra source whitespace (double spaces, off-by-one), not reader macros. Original fold: io 73/0 identical; service 1833 mismatches before the Written join.

## Row 10 — table still wins

`kw-table.wat`: `GROUPS2=1 GROUPS3=1 ROWS=3` — three one-liners, not re-inlined pair-by-pair.
`pos-table.wat`: `GROUPS2=1 GROUPS3=0 ROWS=2` — `"short"` pads to `"much-longer"`.
`key-order.wat`: `GROUPS2=0`.

## 614 doc examples

Rules collected once; each form formatted in isolation.

| | |
|---|---|
| run | **614** |
| changed | **329** (was 342) |
| INLINE (single-line output) | **285** |
| lines still over 120 | **0** |
| worst remaining | **104 cols** |

13 examples that previously exploded now stay the source one-liner. Over-120 still **0**, worst still 104.

Concatenating all 614 as siblings tables unrelated examples (R11 on the wrapper + sibling table) and is not the measurement.

## Walls / comments / load

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — align vs block`. Deleted after. `ClaimedUnder` 0. `grep -c 'col' rules/*.wat` **0**. `grep -c '120' rules/*.wat` **0**. `io.wat` **COMMENTS=28**. Existing fixtures idempotent. `--check` clean on new fixtures before the floor. `every_wat_scripts_file_loads` **1 passed**.

No Rust. `Break` / `Claim` / `AlignPairs` / `BlankBefore` / `TableRow` unchanged as records. `wat/grep.wat` untouched.

## Mechanism

- `Width {id, w}` from the walk (`leaf = length(source)`, `interior = Σ + n + 1`). Not rete (`stratify: negation cycle`).
- `AllAtoms {form}` in `rules/atoms.wat`: AlignPairs, not TableRow, no list/vector/map/set child.
- Map literals: `kwargs-map-claim` + `kwargs-map-keys` (even index > 0). List pair-run rules now require `kind=list`.
- Emitter: TableRow first; else AllAtoms and `this-indent + w <= 120` → skip child Breaks; else explode. Pair-width 0 when skipping.

---

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| atom-map / compound-map / long-map / kwargs-one / unreadable-compound | ruled shapes, **IDEMPOTENT=true** |
| every previous fixture | ruled + idempotent |
| `run.wat` on `wat/io.wat` | **COMMENTS=28** |
| `grep -c 'col'` / `'120'` over rules | **0** / **0** |
| kind-conflict sabotage | **raises**, then deleted |
| width control | CHECKED printed; 2949 excluded |
| 614 doc examples | 329 changed, **0** over 120, worst 104, **285 INLINE** |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED, with one edit — an off-by-one against a shape the builder had written down.**

| what | result |
|---|---|
| ★ row 2 · `{:a 3 :b 42}` | **INLINE** |
| ★ row 3 · `{:a (+ 1 2) :b 42}` | one pair per line — **after my edit, keys ALIGNED** |
| ★★★ row 4 · a long all-atom map | **EXPLODES** — width 151 at indent 2. **The budget is consulted.** |
| ★ row 5 · `(:p::T :a 1)` | **INLINE** — defect 4, answered |
| ★★ row 9 · walls | `grep -c 'col'` **0 files** · `grep -c '120'` **0 files** |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** · clippy **0** |

### THE EDIT — map keys ALIGN, they do not block-indent

```
BUILDER'S RULING            SHIPPED                     AFTER
{:a (wat.core/+ 1 2)        {:a (:wat::i64::+ 1 2)      {:a (:wat::i64::+ 1 2)
 :b 42}                       :b 42}                     :b 42}
 ↑ aligned under :a           ↑ block +2                 ↑ aligned
```

One token in `rules/kwargs.wat`: `:kind "block"` → `:kind "align"`. **`:align` already meant
"align under the first element inside the enclosing bracket"**, which for a map is `{`+1 — exactly
the ruled shape, and the same treatment a `let` binding vector's binders already get. Nothing new
was needed; the wrong kind was chosen.

⚠ **This was visible only because the builder had written the shape down.** The output was
plausible, idempotent, and passed every row I wrote. **My rows said "one pair per line" and never
said which column** — the fourth time this session an acceptance row of mine could be satisfied by a
defect. `[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

### ★ ROW 4 IS THE ONE THAT PROVED THE STONE

Rows 2 and 5 pass with **no width computed at all**. `long-map.wat` — one-line width **151**, body
indent 2, 153 > 120 → exploded. **That is the only row that shows the fold is read**, and it fires.

### ★ THE WIDTH CONTROL, AND ITS RESIDUE IS A FEATURE

```
wat/io.wat      CHECKED    73   MISMATCH  0
wat/rete.wat               690            0
wat/grep.wat               991            3
wat/fix.wat              4,020            9
wat/core.wat             4,670            6
wat/service.wat          4,560           24        ← was 1,833 before the Written join
```

CHECKED printed beside MISMATCH on every file — not a vacuous 0/0. Synthesized nodes excluded, so
`wat/service.wat:2949`'s `~handle-record` is correctly absent.

⭐ **And the residue is not a defect.** The fold computes the **canonical** width — one space between
children — while the span reflects the **actual** source. A mismatch therefore means *the source has
non-canonical spacing*, which is precisely what the formatter exists to fix. The control is
measuring something real, not leaking noise.

### THE 614 DOC EXAMPLES — re-run by me, after the edit

```
N=476   CHANGED=299   INLINE=177   OVER120=0   WORST=104
```

My extraction is narrower than the strike's (476 vs 614), and the conclusion is identical:
**zero over 120, worst 104.** 177 examples now stay one line where they previously exploded.

## Not disputed

`Width` comes from the walk, not rete — the refusal stands. `AllAtoms` is purely structural and the
emitter owns the budget, which appears once in `wat/fmt.wat`. The sibling table still wins over
inlining (`kw-table` 3 rows, `pos-table` padded, `key-order` ungrouped). `wat/grep.wat` untouched.
`io.wat` **COMMENTS=28**.
