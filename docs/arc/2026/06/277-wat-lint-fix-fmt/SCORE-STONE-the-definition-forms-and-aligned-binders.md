# SCORE — STONE: the definition forms, and the aligned `<-`

No commit. Floor and clippy left to the orchestrator. `:examples` fat-arrow untouched (deferred). R8 trailing comments still open.

`AlignStride {form, stride}` is a sibling of `AlignPairs`. Stride 2 stays AlignPairs; stride 3 is name / `<-` / type. The emitter pads; no rule names a column.

## Row 2 — defrecord: name rides, `<-` aligned

`defrecord-fields.wat`, `IDEMPOTENT=true`:

```
(:wat::core::defrecord :fix::Rec
  [celsius <- :wat::core::i64
   some    <- :wat::core::f64
   other   <- :wat::core::String])
```

## Row 3 — defstruct / defenum

defstruct, same treatment, `IDEMPOTENT=true`:

```
(:wat::core::defstruct :fix::St
  [message <- :wat::core::String
   n       <- :wat::core::i64])
```

defenum mixed (`:Shutdown` bare, `:Admin` / `:Pair` with fields, `:Nil []`), `IDEMPOTENT=true`:

```
(:wat::core::defenum :fix::Ev :wat::enum::Pure :Shutdown
  :Admin
  [msg <- :wat::core::String]
  :Pair
  [idx  <- :wat::core::i64
   name <- :wat::core::String]
  :Nil [])
```

Bare `:Shutdown` rides the head. Field-carrying tags start a line; empty `:Nil []` stays one token. Two-field `<-` aligned. No fixed child-index assumption.

## Row 4 — ★★★ defn arg-spec `<-` aligned (uneven names)

`defn-uneven.wat`, `IDEMPOTENT=true`:

```
(:wat::core::defn :fix::uneven
  [self    <- :wat::core::i64
   work-fn <- :wat::core::String
   c       <- :wat::core::f64]
  -> :wat::core::nil
  nil)
```

`self` / `work-fn` / `c` — arrows in one column. This is the long-standing gap.

## Row 14 — step-payload defrecords

`doc-example.wat`: both names ride the head line:

```
(:wat::core::defrecord :probe::StepPayloadExampleTemp
  [celsius <- :wat::core::i64])
(:wat::core::defrecord :probe::StepPayloadExampleResult
  [celsius <- :wat::core::i64])
```

## Prior rules still win

kw-table `GROUPS2=1 GROUPS3=1 ROWS=3`. pos-table padded. atom-map INLINE. kwargs-pos keys aligned. `kwargs-one` INLINE. Every existing fixture `IDEMPOTENT=true`.

## 614 doc examples

| | |
|---|---|
| run | **614** |
| changed | **329** |
| INLINE | **285** |
| over 120 | **0** |
| worst | **104** |

## Walls / comments / load

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Deleted after. `ClaimedUnder` 0. `grep -c 'col' rules/*.wat` **0**. `grep -c '120' rules/*.wat` **0**. `io.wat` **COMMENTS=28**. `--check` clean on new fixtures before the floor. `every_wat_scripts_file_loads` **1 passed**.

No Rust. `Break` / `Claim` / `BlankBefore` / `TableRow` / `Width` / `AllAtoms` / `AlignPairs` unchanged as records. New: `AlignStride`.

kwargs no longer treats a defrecord name as a pair-run key (that was why the name fell off the head line).

## Mechanism

- `AlignStride` + emitter pad at indices `0, stride, 2*stride, …` (includes the first unbroken name).
- `defn-args.wat` asserts stride 3 on the arg-spec vector.
- `rules/defrecord.wat` + `defrecord-fields.wat`: claim the form; break field vectors (not the `:-` type-args vector); one field per line after the first; stride 3.
- defenum: break a variant tag only when the next sibling is a vector (`:-` excluded).

---

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| defrecord / defstruct / defenum-mixed / defn-uneven | ruled, **IDEMPOTENT=true** |
| every previous fixture | ruled + idempotent |
| `run.wat` on `wat/io.wat` | **COMMENTS=28** |
| `grep -c 'col'` / `'120'` over rules | **0** / **0** |
| kind-conflict sabotage | **raises**, then deleted |
| 614 doc examples | **0** over 120, worst 104 |
| `every_wat_scripts_file_loads` | **1 passed** |
