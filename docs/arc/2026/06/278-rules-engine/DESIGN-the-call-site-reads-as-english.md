# DESIGN — the call site reads as English

**Stone C.** Two renames. Closes no defect — and that is stated up front, not discovered later.

## WHY

Two `intueri` findings survived Stones D and D2, both Level 2, both at sites every service author
types:

**1. `Alarm`'s field `after` is a preposition wearing a noun's clothes.**

```wat
(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)
```

reads *"alarm, after a millisecond, 1."* It parses only because the record's docstring twenty lines
up speaks the sentence for you (`service.wat:52`). And it **stutters against the intrinsic it feeds**
— `service.wat:1618-1622` has two `after`s in one form as different parts of speech:

```wat
(:wat::kernel::after <peer-kind> (:wat::service::Alarm/after ~sym) (:wat::service::Alarm/op ~sym))
```

**2. The seven unit constructors are named for a unit and return a quantity.** `(Millisecond 50)`
reads *"a millisecond, 50."* And the corpus has renamed primitive constructors **twice, deliberately,
with the precedent recorded in the same file as the type**: `value.rs:304` — *"formerly
`:wat::core::Char`; Stone 242.1 rename"*; `value.rs:313` — *"Stone C1 lowercased the surface; see the
`char` precedent."* The time constructors were never swept.

## ⛔ THE ONE CONTRACT DECISION

**The call site reads as English without a gloss.**

```wat
(:wat::service::Alarm :delay (:wat::time::Milliseconds 50) :op :-tick)
```

*"alarm, delay 50 milliseconds, op `-tick`."* One reading, no docstring, no stutter.

If the rename lands and the call site still needs the record's prose to parse, the stone moved
tokens and changed nothing. **That is checkable by reading one line.**

## ⚠ THIS CLOSES NO DEFECT

Nothing is broken. Both are Level 2 — mumbles, not lies. It is scheduled **last** for that reason,
and it is worth doing only because the `NonZeroDuration` stone already touched all seven constructor
registrations, so this is the cheapest moment there will ever be.

## FILES

| what | scope |
|---|---|
| `wat/service.wat:67` | `Alarm [after <- …]` → `[delay <- …]` — **STDLIB** |
| `src/…` | one `Alarm/after` accessor |
| corpus | the `:after` kwarg sites — **wat-fix codemod** |
| `src/intrinsic/time.rs`, `src/check.rs`, `src/rete/purity.rs` | the seven constructor names |
| corpus | 99 constructor call sites — **wat-fix codemod** |

**Counts, mine, and the finder's are the fact:** 74 `:after` keyword occurrences across 25 files —
**but `:after` is a bare keyword and some of those are not `Alarm`'s.** Constructor call sites:
`Millisecond` 87, `Nanosecond` 4, `Minute` 3, `Hour` 3, `Second` 1, `Day` 1, **`Microsecond` 0**.

★ My censuses have been wrong **eight** times this campaign, the last three all form-vs-token — and
one of them, in the stone before this, mis-shaped four of ten template sites. **Treat every number
here as a hypothesis.**

⛔ `wat/service.wat` is stdlib, frozen into the binary. `fix.wat`'s **BOOTSTRAP / STASH-DANCE**
governs.

## OUT OF SCOPE = REJECTED

- **`Microsecond` → `Microseconds`.** Zero call sites. Rename it anyway **for symmetry** — seven
  siblings that do not agree is worse than one unused plural — but say that is why.
- **`visible`/`unacked`.** Already landed in Stone D.
- **Anything in `wat/service.wat` beyond the one field.** R1's seam is a separate, re-drawn stone.
- **`Alarm/after`'s call sites in the generated template.** They follow the field; they are not a
  second decision.

## THE PROOF

1. **★ One call site, read aloud.** `(Alarm :delay (Milliseconds 50) :op :-tick)`. If it needs the
   docstring, the stone failed.
2. **★ The stutter is gone.** `service.wat:1618-1622` no longer has two `after`s as different parts
   of speech.
3. **The finder's census**, reported before applying, against my hypothesis.
4. **Both codemods idempotent** — re-run reports 0.
5. **The floor**, `5214/5214`, and the circuit `distinct=8000; dup=0` five runs.
