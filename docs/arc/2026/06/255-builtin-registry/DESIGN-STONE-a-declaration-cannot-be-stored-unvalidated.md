# DESIGN — STONE: a declaration cannot be STORED unvalidated

> Closes the scope limit the previous stone named honestly and left open: *"ONE registration path of
> SIX is wired… a user-program `defn` with a metadata map is not validated today."*

## The defect, measured — and its third part is the worst

```clojure
(:wat::core::defn :my::halfdecl {:purity :wat::runtime::Purity::Pure} [x <- :i64] -> :i64 x)
```

```
"loaded fine — nothing validated the declaration"
then, on the first metadata-of:
  MissingProse … :location {:file … :line 7 :col 56}     ← the metadata-of CALL SITE
                                                            the bad map is at LINE 2
```

1. **Not validated at load** — 5 of 6 registration paths accept axis keys and never check.
2. **Errors late** — at the first read, arbitrarily far from the write.
3. ⛔ **Blames the wrong line** — the span points at the *consumer* calling `metadata-of`, not at the
   author who wrote the bad declaration. A reader is told they broke something they only looked at.

## The population

```
line 861   stdio_bootstrapped              ⛔
line 978   register_defines                ⛔  ← the general path a user def takes
line 1507  (the wired one)                 ✅
line 2848  register_runtime_defs_form      ⛔
line 4007  preregister_fn_defs_in_do       ⛔
line 4082  preregister_fn_defs_in_let      ⛔
```

★ **Six insert sites is itself the smell.** Wiring five more checks would be the CHECK rung of the
ladder — better than today and still one forgotten site from a regression.

## THE ONE CONTRACT DECISION — pinned

**Storing binding metadata and validating it become ONE operation, so an unvalidated declaration has
no way to be written down.**

```rust
sym.binding_metadata.insert(name, meta);        // today: 6 sites, 1 validates
record_binding_metadata(sym, name, meta)?       // the fix: the ONLY way in, validates by construction
```

This is the **NO-FORM rung** of `extirpare`'s ladder, not the check rung: the wrong state stops being
reachable rather than being caught six times.

★ And it makes the read side's failure **structurally unreachable** — if nothing unvalidated can be
stored, `from_metadata` at `metadata-of` time cannot fail on stored data. The late, mislocated error
disappears because the state that produced it cannot exist.

## What ships

1. One `record_binding_metadata` chokepoint that validates (same `AXIS_DECLARATION_KEYS` predicate)
   and inserts; all six sites route through it.
2. The error is raised **at the declaration's own span**, so it names the author's line.
3. Nothing changes for a map with no axis key — capability-only maps store exactly as today.

## Out of scope = REJECTED (not deferred)

- **`:layer`** — still `Substrate`, still not guessed. ⚠ This stone *does* move its unblocking
  condition closer (a user-program def path becomes validated), but knowing a def is *userland* still
  requires the registration context to say so, and this stone does not add that. A separate ruling.
- **Migrating the 409 wat verbs.** Unrelated: this makes declarations *safe to write*, not written.
- **Making `metadata-of` infallible in the type system.** Its `Result` may stay; what changes is that
  the failing state cannot be constructed. Removing the arm is a later, separate cleanup.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **one chokepoint that validates and inserts** | YES | YES | YES | YES | ✅ **ADMITTED** |
| add the validation call to the other 5 sites | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| validate only the user-facing path | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| leave it; the read-side error catches it | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **five-more-calls Honest? NO** — it is the CHECK rung dressed as a fix: six sites that must each
  remember, and the seventh will not. The arc has already paid for this shape twice this week
  (`dispatch_verbs`' two anchors; the completeness gate's `read_dir`).
- **user-path-only Obvious? NO** (a reader cannot tell which paths check) **/ Honest? NO** — it picks
  the paths we happened to test.
- **leave-it Honest? NO** — the read-side error blames the reader for the author's mistake, which is
  worse than no check at all: it teaches the wrong lesson to the wrong person.

## ✅ SHIPPED 2026-08-31

```
binding_metadata.insert sites     6 -> 1        the chokepoint is the only door
the bad map on line 2 reports at  line 8 -> 1   the reader -> the AUTHOR
and it fires at                   read -> LOAD  the program never runs
```

**No call site lacked a span** — STOP-2 did not fire; every one already had a form/child reference
in scope. That also closes the rider's one open uncertainty (whether `defn`'s macro expansion
propagates a real span): it lands on the declaration form.

### ⚠ One behaviour widening worth naming

`register_defclause` (the former site 861) had **never** been validated even conceptually — its own
comment calls the insert *"load-bearing and PROVEN"* for the `{:restricted-to […]}` case and says
nothing about axis keys. Routing it through the one door means a `defclause` declaring an axis key is
now validated too. **Correct per "one door for all six", and beyond what any prior stone's comments
anticipated for defclauses.** Named here so it is not discovered as a surprise later.

### ⚠ Why the committed probe cannot exercise the refusal

`record_binding_metadata` fires during freeze/registration, **before `main` runs** — and `def` at
expression position is a hard `DeclarationInExpressionPosition` refusal, so `eval-ast!` cannot reach
it either. Meanwhile `every_wat_scripts_file_loads` fully loads every scratch `.wat`, so **a
committed probe containing a failing declaration would be a floor regression by construction.**

The probe therefore covers the two *unaffected* shapes, and the refusal was demonstrated once
out-of-tree. ★ That is a real limit of this repo's own gates, not an omission — and it is written
into the probe's header so a future reader does not read the missing row as an oversight.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ a bad declaration fails at LOAD | the `:my::halfdecl` shape above | error **at line 2**, not at a later call |
| ★ the span names the AUTHOR's line | same | the declaration's span, not the consumer's |
| every path is covered | `grep -c "binding_metadata.insert"` | **1** — the chokepoint only |
| capability maps unchanged | a `{:restricted-to …}` def | stores and reads as today |
| the good verb still works | `metadata-of :wat::string::capitalize` | unchanged |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
