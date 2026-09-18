# SCORE — a defclause outranks a defn

**Struck 2026-09-18**, branch `sns-sqs`, from HEAD `c67974cb2`. Nothing committed; the tree is left
dirty for the orchestrator.

**Rule chosen: (a) REFUSE.** A `defclause` may not name something that is already declared. Armed at
a measured **zero** corpus offenders.

**And the scarier variant is settled, not re-named: it was REAL, it was SILENT, and it was worse
than the type error.** A consumer clause on a library record's generated accessor type-checked
green, ran, and turned `7` into `1004` with consumer code printing from inside the library's own
body. `exit 0`, both before and after the clause was added. That is the second door, it is closed
here, and the DESIGN did not know it existed.

---

## ⭑⭑ ROW 1 — PHASE 1, THE COUNTS, AND THE INSTRUMENT FIRST

**Instrument: `wat-scripts/grep/defclause-over-defn.wat`** — a wat-grep program over the form tree
(`Node` / `Named` / `Span` / `Source`), **not text**. It emits one `Match` per
`(:wat::core::defclause :N …)` and per `(:wat::core::defn :N …)` / `(:wat::rete::core::defn :N …)`,
capturing `N` as the child at index 1 of a list whose child at index 0 is that keyword. Run over
every tracked `.wat`:

```
$ git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
    | ./target/release/wat --grep ./wat-scripts/grep/defclause-over-defn.wat
```

| | count |
|---|---:|
| tracked `.wat` files walked | **1873** |
| `defclause` declarations | **71** (66 distinct names, across 34 files) |
| `defn` declarations | **6704** (4380 distinct names) |
| ⭐ **`defclause` + `defn` on one name, SAME FILE** | **0** |
| ⭐ **`defclause` names that ANY file declares as a `defn`** | **2** — and both are accidents, see below |
| duplicate `defclause` name within one file | **0** |

### The two cross-file names are not reliance — they are two unrelated probes

```
:my::bad            defclause  tests/diagnostics/probe_arc296_s7_ensure_reason_enum.wat
                    defn       tests/wat_lang/probe_def_not_special.wat
:probe::describe    defclause  tests/rete/probe_arc278_open_surface_dispatch.wat
                               wat-scripts/probes/arc-170/probe-defclause-{discriminate,open-arg,real-shape}.wat
                    defn       tests/diagnostics/probe_arc296_error_surface.wat
```

`grep -c 'load-file!'` over all five files: **0 each**. They are separate programs that never meet,
reusing a probe namespace. ⭑ **Nothing in this corpus relies on cross-origin `defclause`** — so the
STOP condition in the brief did not fire, and Phase 2 was permitted.

### The instrument was driven BOTH ways

**Non-vacuity:** the same-file-collision rule is a rete self-join, and the fact base is reset per
file, so the join IS the same-file predicate. Driven on a two-line file written to collide:

```
(:wat::core::defn :probe::f [n <- :wat::core::i64] -> :wat::core::i64 n)
(:wat::core::defclause :probe::f ([s <- :wat::core::String] -> :wat::core::String s))
→ defclause-declaration · defn-declaration · defclause-and-defn-in-the-same-file
```

**Against text:** `grep -nP '\(:wat::core::defclause\b'` over the same 1873 files finds **74**.
The three extra are *prose inside comments* —
`wat-scripts/fixes/spawn-program-to-test-spawn-peer.wat:35`,
`wat-scripts/fixes/vend-only-wat-repl-names.wat:21`, `wat/repl.wat:153` — every one of them a
sentence ABOUT a defclause, not a defclause. 74 − 3 = 71, and the gap is accounted for by reading
all three, not by arithmetic.

⚠ **Only the `defclause` half was reconciled against text, and that is the half the ruling rests
on.** The 6704 `defn` figure is the form census's own number and was NOT diffed against a textual
count; `defined-twice.wat`'s standing blind spot applies to both halves — the fact base carries no
quote marker, so a declaration inside a quasiquoted macro template counts as one, and a `defn` whose
index-1 child is not a nameable node emits no `Named` fact and so no row.

**Spellings, scoped before counting:** `(:…defclause` has exactly ONE spelling in this corpus
(`:wat::core::defclause`, which is also the literal `check.rs` matches at `:3166` / `:9252` / `:9326`);
`defn` has TWO declaring spellings (`:wat::core::defn` ×6746 textual, `:wat::rete::core::defn` ×47)
and the finder keys on both. A finder that knew only the first would have under-reported.

### The three load-bearing users the brief named — measured, and one of them was wrong

| brief said | measured |
|---|---|
| `wat/rete/` relies on `defclause` | **ONE** declaration in the whole tree: `wat/rete/oracle/insert.wat:56`, `:wat::rete::insert`. Nothing declares that name as a `defn`. |
| `wat/Record.wat` relies on `defclause` | ⛔ **`grep -c defclause wat/Record.wat` → 0.** The file contains none. The claim does not survive contact. |
| "generic heads" rely on it | 22 stdlib declarations: `wat/core.wat` (10 — `+ - * /` `quot rem mod` `sort sort-by nth-spec`), `wat/seq.wat` (5), `wat/spawn.wat` (3), `wat/bracket.wat` (2), `wat/sqlite.wat` (1), `wat/test.wat` (1). **None is also a `defn`**, so the wall cannot reach them. |

---

## ⭑⭑ ROW 2 — THE RULE, AND WHY NOT THE OTHER TWO

### (a) REFUSE — chosen

A `defclause` is refused when its name is already declared. Two doors, because the collision
surfaces in two different registries (row 3).

### (b) EXTEND — rejected, and the reason is the silent variant

Extending does kill the *type-error* witness: the library's own `(:mylib::greet "hi")` would find
the `defn`'s String clause and work. ⛔ **It does not touch the silent one.** A consumer clause that
*matches* the existing signature joins the table and still changes which body runs — that is
precisely the `1004` measured in row 4, and extending is the mechanism by which it would become
*legal*. It is also the largest change available: the clause table and the registered scheme are
different registries with different dispatch rules, and merging them is a dispatch-semantics
rewrite, not a wall.

### (c) ORIGIN-SCOPED — rejected as strictly wider than the corpus needs

Origin-scoping preserves "an author may add clauses to their own `defn`". The corpus contains
**zero** of those (same-file collisions: 0). So (c) buys a capability no caller uses, while
requiring this language to mint a notion of *origin* it does not have — file? namespace? load
unit? — and each answer is a new rule with its own edge cases. And it leaves the silent
substitution open inside one origin. (a) is narrower **and** kills strictly more.

⭑ **The precedent is `UnreachableClause` (Stone 118.B2c), which is the sibling wall in the same
registry**: *"ARMED AT ZERO OFFENDERS (the house pattern)"*, minted after a census found exactly one
offender and it was the fixture written to be refused. This is that shape again, with 0.

---

## ⭑⭑ ROW 3 — WHERE THE WALL SITS, AND WHY IT IS TWO DOORS AND NOT ONE

The clause table is written at `check.rs`'s pre-pass (`:800`) and again by
`collect_splice_defs_ctx`'s `:wat::core::defclause` arm in the sequential loop. The pre-pass is the
only writer that can see `SymbolTable`, so it is the one that judges and reports;
`CheckEnv::refused_defclause_names` carries that verdict to the second writer, which would
otherwise register the very table the first refused. **One error, one verdict, two writers.**

### Door 1 — `ClauseOverExistingDeclaration` (`defclause_displaces_declaration`)

`sym.get(name)` holds a function that is not this form's own stub.

⚠ **The subtlety that makes a naive fix wrong.** `runtime::register_defclause`'s `Stub` phase
registers a 0-arg, nil-bodied stub `Function` for EVERY `defclause` (so the resolver can validate
recursive clause bodies before the real `ClauseSet` lands at step 9). By check time, **every one of
the 71 corpus defclauses is in `sym.functions` under its own name** — a bare `has_function` test
refuses all of them.

The discriminator is an **identity test, not a shape test**: the stub's body is
`WatAST::NilLit(form.span().clone())` — literally this defclause form's span. A real
`(defn :my::f [] -> :wat::core::nil nil)` has the *identical shape* (empty params, unit return,
`NilLit` body — `:wat::core::nil` canonicalizes to `TypeExpr::Tuple(vec![])`), so shape alone would
have had a silent false-negative class. Only the span tells them apart.

⚠ **Bound, stated:** the identity is `(file, line, col)`. One file loaded twice under two different
path spellings would produce spans differing only in `file`, and the second would read as a
displacement — a located refusal rather than a silent hijack, which is the conservative direction.
No corpus program does it.

### Door 2 — `ClauseOverGeneratedCompanion` (`defclause_squats_type_companion`)

⛔ **Door 1 cannot see this one, and that is the whole finding of row 4.** The squatted companion is
never in `SymbolTable` at all:

```
freeze step 5    register_defines → register_defclause(Stub) mints a Function under :mylib::Point/x
freeze step 6.8a register_aggregate_methods wants to mint the REAL accessor, computes
                 `acc_existing = if sym.has_function(path) { Existing::Equivalent }` — a LIE, the
                 occupant is a defclause stub — and `resolve::gate` returns NoOp. It declines.
freeze step 8    check_program: there is nothing to displace. The companion was never born.
```

So door 2 asks the `TypeEnv` instead: is this name the per-field accessor (`:T/field`) or the
positional constructor (`:T'`) of a declared `TypeDef::Aggregate`? Name-splitting routes through
`wat_reader::identifier`'s `receiver` / `method` / `prime` / `deprimed` — never a hand-rolled
`rsplit_once` (STONE-one-name-grammar; the lint caught me doing exactly that, row 8).

⚠ `is-T?` is deliberately **not** in door 2: `register_type_predicates` already refuses a squatter
outright (`DuplicateDefine`), measured. Adding it would double-report. Its diagnostic is poor (it
locates at `src/runtime.rs:2494`) — named in row 9, not touched.

`:wat::core::/` — division, the one corpus `defclause` name containing a `/` — is why the accessor
arm requires the receiver to be a declared aggregate: `receiver(":wat::core::/")` is `":wat::core:"`,
which is not a type.

---

## ⭑⭑ ROW 4 — ⛔ THE SCARIER VARIANT: DRIVEN, REAL, AND SILENT

The DESIGN's trap-door 3 asked whether a consumer clause that *matches* the signature re-points
dispatch into consumer code at runtime while type-checking green. **It does — through the generated
companions, which nobody had looked at.**

```wat
;; lib3.wat — the LIBRARY
(:wat::core::defrecord :mylib::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :mylib::sum [p <- :mylib::Point] -> :wat::core::i64
  (:wat::core::+ (:mylib::Point/x p) (:mylib::Point/y p)))
```

```wat
;; the CONSUMER
(:wat::load-file! "lib3.wat")
(:wat::core::defclause :mylib::Point/x
  ([p <- :mylib::Point] -> :wat::core::i64
    (:wat::core::do (:wat::kernel::println "CONSUMER accessor ran") 1000)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:mylib::sum (:mylib::Point :x 3 :y 4))))
```

### BEFORE (pristine `c67974cb2` release binary)

```
acc_base.wat    (no clause)              →  7                          exit 0
acc_hijack.wat  (the clause, after load) →  "CONSUMER accessor ran"    exit 0
                                            1004
acc_hijack_before.wat (clause BEFORE the load-file!)
                                         →  "CONSUMER accessor ran…"   exit 0
                                            1004
```

⭐ **No error. No warning. Exit 0 in every arrangement.** The library's own
`(:mylib::Point/x p)` executed consumer code; `3` became `1000` and the sum `7` became `1004`.
Order-independent, because the squat happens at step 5 and the codegen skip at step 6.8a — source
order never enters it.

### AFTER

```
acc_base.wat    →  7                                                   exit 0
acc_hijack.wat  →  #wat.check/ClauseOverGeneratedCompanion  ×1         exit 3
                   :location {:file "acc_hijack.wat" :line 3 :col 1}
```

The same substitution over a plain `defn` (`sub_hijack.wat`: a consumer clause with the library's
EXACT signature and a different body) is refused by door 1.

⚠ **What the DESIGN's own spike said, and why it was not the whole answer.** The FINDING recorded
*"a clause that does match → green (verdict unchanged)"* and scoped the runtime question out. That
was right about the `defn` path — a matching clause on a plain `defn` is caught by the redef gate,
loudly if misdirectedly (it fires `DefRedefForbidden` by default and `DefRedefTypeChange` under
`set-redef! true`, both **locating in the library and citing the consumer as `prior-loc`** — the
blame exactly inverted). It was the **generated** names, which have no `defined_values` binding and
therefore no redef gate, where green meant green.

---

## ⭑⭑ ROW 5 — THE WITNESS KILLED, BEFORE AND AFTER

The DESIGN's witness, verbatim from its own text.

```wat
;; lib.wat
(:wat::core::defn :mylib::greet  [s <- :wat::core::String] -> :wat::core::String s)
(:wat::core::defn :mylib::caller [] -> :wat::core::String (:mylib::greet "hi"))
```
```wat
;; witness.wat
(:wat::load-file! "lib.wat")
(:wat::core::defclause :mylib::greet ([n <- :wat::core::i64] -> :wat::core::i64 n))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [_ (:mylib::caller)] nil))
```

### BEFORE — two errors, BOTH located in a file the consumer never wrote

```
exit=3  "2 type-check errors"
  #wat.check/DefRedefForbidden      :location {:file "lib.wat" :line 2 :col 1}
                                    :prior-loc {:file "witness.wat" …}     ← blame inverted
  #wat.check/NoMatchingClauseAtCallSite
        :message "no clause of `:mylib::greet` matches arity 1 with types [:wat::core::String];
                  clauses attempted: (1: [:wat::core::i64])"
                                    :location {:file "lib.wat" :line 3 :col 83}
```

### AFTER — one error, located at the consumer's own declaration

```
exit=3  "1 type-check error"
  #wat.check/ClauseOverExistingDeclaration
    :location        {:file "witness.wat" :line 2 :col 1 :end {:line 2 :col 84}}
    :name            ":mylib::greet"
    :prior-kind      "function"
    :prior-decl-loc  {:file "lib.wat" :line 2 :col 82}
```

### ⭑ THE NEW DIAGNOSTIC'S TEXT, verbatim

> this `:wat::core::defclause` declares `:mylib::greet`, which is already declared as a function at
> lib.wat:2:82. A defclause's clause table OUTRANKS an existing declaration at every call site in
> the program — including call sites inside the file that made the first declaration — so this form
> would re-point `:mylib::greet` for its declarer too, not just for you. Give these clauses a name
> of your own, or change the declaration at lib.wat:2:82.

and door 2's:

> this `:wat::core::defclause` declares `:mylib::Point/x`, which is the per-field accessor the
> declaration of `:mylib::Point` generates. Declaring it here does not add a clause to that
> companion — it takes the name INSTEAD of it, so `:mylib::Point`'s own uses of `:mylib::Point/x`
> would call this clause table with no type error at all. Give these clauses a name of your own.

The outer span is the **consumer's form** — the site the author edits — in both. `prior-decl-loc`
still cites the displaced declaration, so the victim is named without being blamed.

⚠ **Honest about what the message does NOT say, and why.** Door 2 carries no prior location:
`AggregateDef` has no declaration span in `TypeEnv`, and citing the defclause's own span as the
"prior" location would name the wrong file. It carries `:type ":mylib::Point"` instead, which is
what a reader needs to find it.

---

## ⭑ ROW 6 — THE LINES THE WALL DOES NOT CROSS, EACH DRIVEN

| program | before | after |
|---|---|---|
| library loaded, no clause | exit 0 | **exit 0** |
| `defclause` on a fresh name, then called | 42 | **42** |
| accessor baseline (record + library sum) | 7 | **7** |
| ⭐ **one file holding a `defclause`, loaded TWICE** | 42, exit 0 | **42, exit 0** |
| stdlib freeze (71 corpus defclauses incl. 23 stdlib) | green | **green** — the whole floor |

⭐ The double-load row is the one that matters: it is a LIVE green path today, and it is exactly
what a `has_function`-only wall would have broken. It is a standing test
(`loading_one_file_twice_is_still_green`), not a one-off run.

### One deliberate behaviour change beyond the witness, with its census

Two `defclause` **forms** for the same name in one file are now refused. Today they silently
last-wins: `(defclause :two::f ([i64]…)) (defclause :two::f ([String]…))` then `(:two::f 7)` fails
with `NoMatchingClauseAtCallSite` because only the SECOND table exists — the first form's clauses
are dropped with no diagnostic. Census over 1873 files: **zero** files carry a duplicate
`defclause` name. It is the same "express something's def once" rule `UnreachableClause` cites.

---

## ⭑⭑ ROW 7 — ⛔ THE FLOOR WENT RED FIRST, AND THE RED WAS MINE

**Do not bury this.** The first floor run on this tree was RED, and all three arms were caused by
the test file I had just written. Captured whole at `.floor/2026-09-18T21-19-05Z/ARM.txt` (kept).

```
     Summary [ 256.892s] 5299 tests run: 5296 passed, 3 failed, 22 skipped
        FAIL [   0.060s] (  71/5299) wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat
        FAIL [   0.091s] (  77/5299) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
        FAIL [   0.403s] (  93/5299) wat::lint one_name_grammar::only_identifier_rs_parses_a_name
```

The exact arms, each a different mechanism:

1. **`no_inlined_wat_in_tests.rs:427`** — *"INLINED-WAT IN TESTS — 1 file(s) still carry a string
   literal that wat's own reader parses as a form… Offenders: `tests/function/probe_defclause_over_defn_refused.rs`"*.
   I had put the library and all six consumer programs in Rust `r#"…"#` literals.
   **Fixed** by moving every one to a co-located fixture (`_lib.wat`, `_lib_record.wat`,
   `_clause_lib.wat`, three `.wat.bad` negatives, three `.wat` positives) read with
   `std::fs::read_to_string` and handed to the `InMemoryLoader`.
2. **`no_loose_string_assert.rs:112`** — *"LOOSE STRING ASSERTIONS — 5 site(s)"*, at
   `probe_defclause_over_defn_refused.rs:122,126,130,193,194`. **Fixed** by moving the presence
   checks into the `for needle in […]` idiom (the lint's predicate requires a string *literal*
   argument) and earning one `// rune:lint(loose-assert)` for the single legitimately-loose site:
   a targeted ABSENCE (`!contains("NoMatchingClauseAtCallSite")`) over the whole rendered error
   tree, which the lint's own rubric names as exemptible.
3. **`one_name_grammar.rs:114`** — *"A SECOND NAME PARSER — 2 site(s)… Offenders:
   `src/check.rs:9447 rsplit_once('/')` · `src/check.rs:9456 strip_suffix('\'')`"*. That is
   **source**, not test: my `defclause_squats_type_companion` hand-rolled the name grammar.
   **Fixed** by routing through `wat_reader::identifier::{receiver, method, prime, deprimed}` — the
   one door.

★ Arm 3 is the one worth keeping: the lint caught a *real* defect of the kind it exists to prevent
(two parsers for one name grammar), in code written ten minutes earlier, and the fix is strictly
better than what I wrote.

### THE FLOOR, GREEN, ON THE TREE LEFT BEHIND — Summary verbatim

```
     Summary [ 258.434s] 5299 tests run: 5299 passed, 22 skipped
[floor] exit=0. Log kept at .floor/2026-09-18T21-31-26Z/ regardless — a green run is evidence too.
```

Read from the Summary line, never a piped exit code. Log at `.floor/2026-09-18T21-31-26Z/`
(`raw.log` + `clean.log`). **NO `ARM.txt`.** Zero `FAIL` / `TIMEOUT` / `SIGSEGV` / `ABORT` tokens in
the log (`grep -c 'FAIL' clean.log` → 0).

**5299, up from the prior stone's 5292 — the +7 is this stone's gate**, all seven present in the log
by name under `wat::function probe_defclause_over_defn_refused::*`. Tier A's band holds: 258.434 s
against the prior four runs' 259.390 / 259.799 / 259.813 / 254.555 s.

⚠ **And a green floor proves little here, exactly as trap-door 4 predicted** — the corpus contains
**zero** collisions, so nothing in it could have gone red either way. The argument is from the PATH
(row 3) and from the driven witnesses (rows 4–6), not from the number. What the green *does* prove
is the negative: 71 defclause declarations, 23 of them stdlib, and the wall refuses none of them.

---

## ⭑ ROW 8 — CLIPPY + `--no-run`, READ NOT ASSUMED

```
$ cargo clippy --release --workspace --all-targets   →  exit 0
   grep -c '^error'   on the captured log  →  0
   grep -c '^warning' on the captured log  →  0
$ cargo nextest run --release --no-run              →  exit 0
$ cargo build --release                             →  exit 0
```

Counted from a `tee`'d log, not inferred from the tail. Run twice — once before the lint fixes and
once after — because the tree that is graded must be the tree that was measured.

### The circuit happy path (`~/work/BREADCRUMB.md` § FRESHNESS PROBE), one line, driven

```
distinct=8000 dup=0 workers=12  ·  pub-exh-last=none  ·  bp-delay=0
total=13723  ·  exit=0
```

---

## ⭑ ROW 9 — WHAT I DID NOT DRIVE, NAMED

1. ⛔ **The companion codegen still declines silently, and I did not fix it.**
   `register_aggregate_methods` computes `acc_existing = if sym.has_function(path)
   { Existing::Equivalent } else { Absent }` (`runtime.rs:2112`, and a second copy of the same shape
   at `:3394`) — calling a defclause stub "equivalent" to the accessor
   it is about to mint. Its own comment (`runtime.rs:1787`) says *"DuplicateDefine is an error —
   after the macro's accessor emission was removed, no other path registers these accessor paths"*,
   and that sentence is **false today**: the defclause Stub path does. The honest repair is at that
   site, but `Existing::Equivalent` there is also what keeps the boot-cache replay green (step 6.8a
   re-runs over cached symbols that already hold the accessors), and the brief forbids touching the
   boot cache. **I walled the consequence at check time instead and am naming the cause here.**
   Its visible residue: after door 2 fires, the run also reports knock-on errors inside the library
   (`ArityMismatch: :mylib::Point/x: expected 0 argument(s); got 1`) because the accessor was never
   minted and the 0-ary stub is what the call site sees. **The located, correct error is first**;
   the trailing ones are the symptom of the never-minted companion, not of the wall.
2. **`is-T?`'s diagnostic locates in Rust.** `register_type_predicates` refuses a squatter with
   `#wat.runtime/DuplicateDefine {:location {:file "src/runtime.rs" :line 2494}}` — the right
   verdict at the wrong address. Same family as this stone's diagnostic work; a separate site.
3. **Eval-time / REPL `defclause`.** The wall is in `check_program`. A `defclause` registered purely
   at eval time (`register_runtime_defs_form`'s Runtime phase, the REPL door) does not pass through
   it. Not driven; the corpus has no such program.
4. **`:wat::core::def` value redef vs a clause table.** `register_defclause` also writes a sentinel
   `Var(u64::MAX)` into `defined_values`, which is why the `defn` collision produced a
   `DefRedefForbidden` at all. I left that path exactly as it was — the wall fires before it and the
   program never reaches it, but the inverted blame in that message is still there for anyone who
   reaches it another way.
5. **Whether `defclause` should be refused over a `def` VALUE or a macro.** Out of the stone's
   scope; the census measured only `defn`.

---

## ⛔ ROW 10 — SCOPE, IN FULL

```
 M src/check.rs        159 +++   the two doors + the pre-pass signature; NO dispatch change
 M src/check/env.rs     12 +     `refused_defclause_names`, so the second clause-table writer
                                 obeys the first's verdict
 M src/check/error.rs   94 +     two CheckErrorKind variants + their Display
?? tests/function/probe_defclause_over_defn_refused.rs          7 tests
?? tests/function/probe_defclause_over_defn_refused_*.wat       6 fixtures
?? tests/function/probe_defclause_over_defn_refused_*.wat.bad   3 must-fail fixtures
?? wat-scripts/grep/defclause-over-defn.wat                     the census instrument
 M wat-scripts/grep/README.md                                   one table row for it
```

**UNTOUCHED, by name:** `.config/nextest.toml` · the boot cache (`src/freeze/boot_cache.rs`) ·
Tier B · `check.rs:6068`'s defclause **dispatch precedence** (the wall refuses the registration; it
does not change what wins once a table exists) · `CheckEnv::register_defclause`'s unconditional
insert (`check/env.rs:456`) · `runtime::register_defclause` · `register_aggregate_methods`.
No `.wat` corpus file was edited, by hand or otherwise — the one new `.wat` is a new file.

---

## ⭑ ROW 11 — WHAT SURPRISED ME

1. ⭐ **The runtime already has the instinct the checker lost.** `register_defclause`'s Stub arm is
   written `if reserved_ok && !sym.has_function(&name)` — it *already refuses to clobber* an
   existing function. The check-time clause-table registration, doing the same job in a different
   registry, has no such guard. The asymmetry the DESIGN called "the flaw" is one missing
   conjunction away from its own sibling.
2. ⭐⭐ **The silent variant lives in the names nobody writes down.** Every program in this arc's
   history attacked names that appear as a declaration in source. The reachable-and-silent surface
   turned out to be the names that appear in **no** source at all — `:T/field`, `:T'` — because the
   only wall protecting them was a codegen pass that treats an occupied name as a benign
   re-declaration. A census of *written* names could never have found it; the witness did.
3. **The DESIGN named `Record.wat` as a load-bearing `defclause` user and it contains none.** Third
   time on this branch that a brief's file list disagreed with the census, and the census won every
   time (`the-stdlib-vends-only-wat/SCORE.md` row 5 is the previous one).
4. **The lint suite caught a real defect in my source, not just style.** `one_name_grammar` flagged
   `rsplit_once('/')` + `strip_suffix('\'')` in code I had written minutes before — a second parser
   for a name grammar, which is exactly the thing that stone exists to prevent.
5. **The stub and a legitimate `defn` are structurally indistinguishable.** `:wat::core::nil`
   canonicalizes to `TypeExpr::Tuple(vec![])`, so `(defn :my::f [] -> :wat::core::nil nil)` has the
   stub's exact shape. I wrote the shape test first; it would have had a silent false-negative
   class, and only the span identity closes it.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-18

```
floor    Summary [ 255.830s] 5299 tests run: 5299 passed, 22 skipped
         .floor/2026-09-18T21-42-33Z/ · exit=0 · NO ARM.txt · Tier A's band holds
clippy   0  ← read from the command's own output, not assumed
```

| check | my own result |
|---|---|
| **door 1** — plain `defn` witness | ✅ `ClauseOverExistingDeclaration` |
| **door 2** — the SILENT accessor case | ✅ `ClauseOverGeneratedCompanion` |
| **the diagnostic blames the consumer** | ✅ `:location hijack.wat:2:1` (the consumer's form) with `:prior-decl-loc lib.wat:1:81` as context. **Before: two errors, BOTH located in `lib.wat`, one with blame inverted.** |

The message, verbatim from my run — it names the mechanism *and* both remedies:

> *"this `:wat::core::defclause` declares `:mylib::greet`, which is already declared as a function at
> lib.wat:1:81. A defclause's clause table OUTRANKS an existing declaration at every call site in the
> program — including call sites inside the file that made the first declaration — so this form would
> re-point `:mylib::greet` for its declarer too, not just for you. Give these clauses a name of your
> own, or change the declaration at lib.wat:1:81."*

## ⭐⭐ THE SCARIER VARIANT WAS REAL, AND WORSE THAN EITHER OF US DESCRIBED

The spike scoped it out; the DESIGN carried it forward as *"name it again if you cannot settle it"*. It
is settled, and it is not a type error at all:

> A consumer `defclause` on a library record's **generated accessor** `:mylib::Point/x` **type-checked
> green, ran to exit 0, and turned 7 into 1004** — consumer code executing from inside the library's
> own body, in both source orders.

⛔ **And the mechanism is a SUPPRESSION, not an override.** The defclause stub claims the name at freeze
step 5; `register_aggregate_methods` (6.8a) finds the slot occupied, classifies it
`Existing::Equivalent`, and **declines to mint the real accessor**. There was never anything to displace.
That is why the fix needs **two doors**: the collision surfaces in two registries, and door 2 must ask
`TypeEnv` rather than `sym.functions`.

★ The discriminator against a defclause's *own* stub is the stub body's **span identity** — a shape test
would have a false-negative class, because `(defn :f [] -> :wat::core::nil nil)` has the stub's exact
shape. That is the kind of detail that decides whether a guard works or merely looks like it does.

## ⭑ IT ARGUED ME OUT OF ORIGIN-SCOPING, AND WAS RIGHT

I had recommended (c) origin-scoped — *"you may extend what you own"* — in conversation with the builder.
Phase 1 killed it on measurement:

```
defclause declarations                          71  (66 distinct names, 34 files)
defclause + defn on ONE name, SAME FILE          0
cross-file name overlaps                         2  — probe files with load-file! = 0 each; they never meet
duplicate defclause name within one file         0
```

Origin-scoping would preserve a same-origin use the corpus has **zero** of, needs a notion of "origin"
the language does not have, **and leaves silent substitution open inside one origin.** Refuse is
narrower **and** kills strictly more. Armed at zero offenders — the `UnreachableClause` house pattern.

## ⛔ AND MY DESIGN WAS WRONG A THIRD TIME TODAY

It named `wat/Record.wat` as one of three load-bearing `defclause` users. **`wat/Record.wat` contains
ZERO `defclause`** — verified by me. `wat/rete/` has one declaration (`:wat::rete::insert`, no `defn`
anywhere); "generic heads" are 22 stdlib declarations, none also a `defn`. **Nothing relies on
cross-origin `defclause`,** so the STOP never fired.

★★ Three briefs today carried a fabricated or stale fact: **stale line numbers** (+21,
`a-send-that-should-be-bounded`), an **over-broad file list** that would have moved two user demos into
a reserved namespace (`the-stdlib-vends-only-wat`), and now an **invented dependency**. Every one was
caught because the executor measured instead of trusting the brief. ⛔ **A brief's confident fact is a
claim with someone else's time attached** — `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`.

## Its floor went RED first, all three arms its own

`Summary [ 256.892s] … 5296 passed, 3 failed` — `.floor/2026-09-18T21-19-05Z/ARM.txt` kept, not re-run:
inlined wat in tests (→ co-located fixtures), five loose string asserts (→ loop idiom plus **one earned**
`rune:lint(loose-assert)` for a targeted absence), and ⭐ **`one_name_grammar` catching a second name
parser in its own new `check.rs` code** — `rsplit_once('/')` + `strip_suffix('\'')`, routed through
`wat_reader::identifier`. The tree's own lints caught all three.

## Named, not fixed — and correctly so

1. ⛔ **The companion codegen still declines silently.** `runtime.rs:2112`'s `Existing::Equivalent` is
   the *cause*; the stone walls the *consequence* at check time because repairing the cause is what keeps
   the boot-cache replay green — which the brief forbade touching. **Residue disclosed:** after door 2
   fires, knock-on `ArityMismatch` errors still appear inside the library, but the correctly-located
   error is **first**.
2. `is-T?`'s refusal locates at `src/runtime.rs:2494`.
3. **Eval-time / REPL `defclause` does not pass through `check_program`** — the wall does not cover it.

Untouched by name, as scoped: `check.rs:6068`'s dispatch precedence, `env.rs:456`'s unconditional insert,
`.config/nextest.toml`, the boot cache, Tier B. No `.wat` corpus file edited.
