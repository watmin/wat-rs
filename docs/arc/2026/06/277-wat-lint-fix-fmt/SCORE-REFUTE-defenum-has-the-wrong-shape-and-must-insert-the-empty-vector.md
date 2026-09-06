# SCORE — REFUTE: defenum has the wrong shape, and it must INSERT the empty vector

No commit. Floor and clippy left to the orchestrator. Rows 2 / 3-defstruct / 4 / 14 of the parent stone are unchanged.

This is a **category change**: wat-fmt inserts a token. Licensed only for `[]` on a bare enum variant — both spellings are legal and equivalent. A second insertion still needs its own argument.

## E1 / E2 / E3 / E4 / E5 / E6 / E7 — `defenum-mixed.wat`

`IDEMPOTENT=true`. Formatted (and `--check` clean):

```
(:wat::core::defenum :fix::Ev :wat::enum::Pure
  :Shutdown []
  :Admin    [msg <- :wat::core::String]
  :Pair     [idx  <- :wat::core::i64
             name <- :wat::core::String]
  :Nil      [])
```

- Tag and vector are one unit.
- Head line is name + purity; first variant is not special.
- Tags padded so vectors share a column (`:Admin` / `:Pair` / `:Nil` pad to `:Shutdown`).
- Multi-field: second field aligns under the first, `<-` in one column.
- `:Shutdown` was bare → `[]` inserted.
- `:Nil []` already empty → not duplicated.
- Second pass does not insert again.

## E8 — spawn.wat's shape

Same parametric mix as `wat/spawn.wat:195-203`, as a loadable fixture (`defenum-spawn-shape.wat`). `IDEMPOTENT=true`, formatted output `--check` clean:

```
(:wat::core::defenum :fix::SE :- [I O A] :wat::enum::Impure
  :Shutdown   []
  :Admin      [msg <- :A]
  :Connection [peer <- (:wat::kernel::Peer :- [I O])]
  :Message    [idx <- :wat::core::i64
               msg <- :O]
  :Closed     [idx <- :wat::core::i64]
  :Lost       [idx   <- :wat::core::i64
               cause <- :wat::kernel::Failure]
  :Malformed  [idx   <- :wat::core::i64
               cause <- :wat::kernel::Failure]
  :Rejected   [idx   <- :wat::core::i64
               cause <- :wat::kernel::Failure])
```

`:- [I O A]` stays on the head line. Formatting `wat/spawn.wat` itself inserts `:Shutdown []` and pads tags the same way. `--check` of a formatted copy of `spawn.wat` is not a valid gate (the stdlib already declares `ServiceEvent`). Comment-indent on the live file is the corpus-only emitter defect, unchanged.

## E9 — no regression

defrecord / defstruct / defn uneven `<-` / step-payload names on the head line / atom-map INLINE / kw-table GROUPS2=1 / every existing fixture `IDEMPOTENT=true`.

614 examples: **N=614 CHANGED=330 INLINE=284 OVER120=0 WORST=104**.

## Mechanism

- `EmptyVecAfter {id}` — the emitter writes ` []` after that node. Asserted only when a variant tag's next sibling is not a vector.
- Variant tags Break `"block"`; field vectors do not (they ride the tag). AlignPairs on the defenum form pads tags (broken child whose next rides, including EmptyVecAfter).
- Inside a field vector: existing AlignStride 3.
- kwargs claim/keys/prefix exclude defrecord/defstruct/defenum heads (AlignPairs on defenum would otherwise look like a pair-run).
- AllAtoms does not fire on defenum.

## Walls / comments / load

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Deleted after. `ClaimedUnder` 0. `grep -c 'col'` / `'120'` over rules **0**. `io.wat` **COMMENTS=28**. `every_wat_scripts_file_loads` **1 passed**.

No Rust.

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED. No edit.** Both corrections landed; the parent stone's accepted rows did not regress.

| what | result |
|---|---|
| ★ E1-E4 · the ruled enum shape | tag+vector one unit · head line is name+purity · tags padded · multi-field aligned |
| ★★★ E5 · `[]` INSERTED | `:Shutdown []` — the category change, working |
| ★★ E6 · already-empty `:Nil []` | **not duplicated** |
| ★★★ E7 · idempotence | **`IDEMPOTENT=true`** — no `[] []` on pass 2 |
| E8 · the parametric mixed enum | `:- [I O A]` rides; every variant paired; `--check` clean |
| E9 · no regression | defrecord · defstruct · defn's `<-` · step-payload names · table · atom-map |
| the 614 doc examples | **0 over 120**, worst 104 |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** · clippy **0** |

E7 was the row where an insertion rule characteristically dies, and it holds: pass 2 recognises the
`[]` pass 1 wrote.

★ **The mechanism is honest about what it is.** `EmptyVecAfter {id}` — the emitter writes ` []`
after a tag whose next sibling is not a vector. **A separate fact for a separate axis**, the same
discipline that put `BlankBefore` and `AlignPairs` in their own records rather than overloading
`Break.kind`. The `[]` licence stays exactly as narrow as the refute drew it.

## ⛔ AND THE REAL FILE SHOWS THE CORPUS DEFECT IS WORSE THAN "INDENT DRIFT"

`wat/spawn.wat:195-203` formatted — **the one file E8's fixture was modelled on**:

```
  (:wat::core::defenum :wat::spawn::ServiceEvent :- [I O A] :wat::enum::Impure
      :Shutdown   []
      
      ;; owner dropped the handle (self-peer drained) — exit; deadlock-free termination
  :Admin      [msg <- :A]                    ⛔ DE-INDENTED to column 2
      
      ;; owner sent an admin op over the lineage peer (Ok path); A = self-peer's recv type
  :Connection [peer <- (:wat::kernel::Peer :- [I O])]
      :Message    [idx <- :wat::core::i64
```

**Three faults, one cause.** A variant carrying a trailing comment leaves the emitter's column state
wrong, so the NEXT variant lands at column 2 instead of 6; the tag padding is lost across that
boundary; and the trailing comment is promoted to its own line ABOVE the following variant, moving
it away from the variant it documents.

⚠ **This is NOT a regression from this stone.** It is defects 1 and 2 of
`[[NOTE-the-formatter-had-never-met-a-real-file]]`, and it is invisible to every fixture because
**every fixture in this arc is comment-free**.

⛔ **But it now has teeth it did not have before.** `wat/sqlite.wat:52-55` and
`wat/spawn.wat:195-203` are the two files whose hand-alignment the builder pointed at as good — and
they are **commented enums**, so they come out WORSE than they went in. The corpus-only defect and
the enum work have collided in exactly one place.

★ **It does not block the priority path**: a doc `@example` lives on one `///` line and cannot
contain a `;;`, so nothing on the arc-255 route touches this. **It does block ever pointing
`wat fmt` at the corpus**, which the builder has already scoped to a test bed.

## The remaining board

```
⛔ the emitter's column state across a comment      the ONLY thing between here and corpus use
   R8 · aligned trailing comments                  needs the comment fact; same neighbourhood
   level-2 alignment inside a value                open since the kwargs stone
   the :examples fat arrow                         deferred, NOTE carries the vertical shape
   arc 109 · make a bare variant ILLEGAL           the formatter is now its migration
✅ the arc-255 doc migration                        UNBLOCKED — 0 over 120, idempotent
```
