# SEAM — the ONE live breadcrumb. As of 2026-08-23 (②-iii · ③ · the COMMA — keyword AND symbol — ALL SHIPPED). Replaced in place.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.** A hand-typed hash is an instrument that can be
> wrong, and one was: it read `f0d3fb2`, not a valid object in this repository. Paste this — it has
> no hash to mistype:
>
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
>
> **Empty → nothing moved since this was written.** Non-empty → every commit listed outranks every
> line below, and you re-read those before you trust a word of it.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 4881/4881, 0 FAIL, 19 skipped, ~80s  (own invocation, scripts/floor.sh, at 17cbe1d4f)
                ⚠ EVERY MOVE IN THIS COUNT IS ACCOUNTED. A count that moves for an
                unexamined reason is what this line exists to catch:
                  4855 → 4854  the --check stone deleted one test, renamed another
                  4854 → 4859  the `:peers` negative controls added five
                  4859 → 4866  the builtin-type registry added seven
                  4866 → 4882  ③'s guards + fixtures added sixteen
                  4882 → 4881  the comma strike: 55-file codemod, fixtures restructured
                  4881 → 4881  the SYMBOL comma wall added no tests (5 fixtures migrated in place)
                If you floor and see 4881, that is green. Anything else, EXPLAIN before accepting.
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash@{0} ...... the lifecycle strike. NEVER drop. base ff7705ba.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s> …`.
⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild** (`include_str!` at RUST-compile time).
⚠ **`cargo wat` uses the INSTALLED binary at `~/.cargo/bin` and it is STALE.** Use `target/release/wat`.
   It fails with `UnknownFunction` on `keyword/to-type-form-colon`; that is not a finding.

## ✅ NO RIDER IN THE FIELD — the tree was quiescent at `17cbe1d4f`

The last-comma stone is SCORED and SHIPPED (`109/SCORE-STONE-the-last-comma-lives-in-a-symbol.md`).
Rows 1–6 re-run by my own hand. **The census was IMPOSED, not derived** — the wall itself run over
all 1798 `.wat`/`.wat.bad` files: **ZERO surviving symbol-commas.**

## ★★ THE WORK: ARC 109 — `:-` IS THE PARAMETERIZATION OPERATOR, AND IT IS THE ONLY ONE

```clojure
[n :- wat.type/i64]                          arg-spec
:- wat.type/i64                              ret-type
(wat.type/Vector :- [wat.type/i64])          type args        — a REFERENCE, in parens
(wat.type/Vector :- [wat.type/i64] 1 2 3)    constructor      — the reference PLUS values
(wat.core/defn ns/f :- [T] [x :- T] :- T x)  declaration      — a BINDER, siblings, NO parens
```

**A binder is the reference form minus its parens.** `:- []` is the assumed default: there is no
monomorphic-vs-parametric distinction, only a param list usually empty.

## ✅ SHIPPED — the crusade's spine

```
γ-i           `fn` AND `defn` take the `:- [T …]` binder                       c889639aa
identity 1/3  `family_extends` gets its own door                               edb7f66c7
identity 2a·2b·2c   roles split; ALL 22 annotations emit the `:-` form   41a3d0dd7 0366b2f2b 073dda92c
blocker 5     a type reference is not an EXPRESSION (expander + resolver)       b9df7a09a
type-equal?   types are data everywhere EXCEPT in a macro                      c5b9b6552
:peers        `defservice` READS + COMPARES types as data                      2d25b4790
neg-controls  the `:peers` bijection keeps its negative controls (2×2 perturb)  2d32fd605
registry      TypeEnv holds the BUILTIN types — THE DOOR tells the truth        10599eb36
②-iii ✅      THE STDLIB SPEAKS `:- [T …]` — 947 forms, 36 files                 2a0d7fa2e
③ ✅          ANGLE-BRACKET PARAMETRICS ARE ILLEGAL — 543 files, 710 → 0         ab52b7188
comma ✅      THE COMMA DIES IN THE READER — one clause; wire escape deleted     575f8fb08
comma ✅      …AND IN A SYMBOL — the arc 271 carve-out retired; 0 of 1798 left   17cbe1d4f
```

⚠ **`<K,V>` IS UNEXPRESSIBLE.** `Vector<i64>` → *"angle-bracket parametric types are illegal"*.
A comma in a keyword body → *"comma inside keyword body retired"*; in a SYMBOL body → *"comma inside
symbol body retired"*. `_`'s language-wide reservation
inside `<…>` is GONE; `:Vec<a_b>` is an ordinary keyword. Wire mode (`Lexer::new_wire`,
`Parser::new_wire`, the `,`↔`_` escape) is **deleted, not stubbed** — it had zero external callers.

⚠ **AND THE DUAL, which every wall here must preserve:** `(:wat::core::Vector :- [:i64] 1, 2, 3)`
→ `[1 2 3]`. **Commas are still EDN whitespace between VALUES.** Only comma-as-separator-inside-a-NAME
died. A wall that refuses commas everywhere passes its own test and breaks the language.

## ⛔ NEXT

1. **`:-` IS NOT YET THE ONLY OPERATOR IN THE METHOD-NAME SLOT.** ⚠ The door
   (`src/types/surface.rs::parse_method_member_sig`) now takes the binder **AND** the inline
   `name<T>` spelling — it dispatches on `name_raw.contains('<')` FIRST. `split_method_name_type_params`
   is unchanged. **Measured: FOUR sites keep the second spelling alive** —
   `wat-scripts/probes/arc-170/probe-locus1-generic-surface-method.wat:9`,
   `tests/types/probe_arc293_4e_pre_iii_extend_impl_inherits_types.wat:13`,
   `tests/types/probe_arc293_4e_pre_ii_generic_surface_method.wat:15` (+ its header comment).
   Migrate the four, delete the splitter, and the inline angle form leaves the method-name slot the
   same way ③ took it out of the type slot. **This is the smallest stone on the board.**
2. **The binder peel's SILENT DISCARD, now at TWO slots.** `filter_map` drops any element of
   `:- [...]` that is not a non-reference `Symbol`, so `:- [S 3]` silently yields `[S]`. Copied
   verbatim from `src/function/metadata.rs::peel_type_binder` (γ-i). Close BOTH at once — a slot
   with two implementations is two slots. (The surface door already tightened one arm: a non-Vector
   after `:-` now raises `MalformedDecl` where γ-i silently un-peels.)
3. **The retired spelling survives in PROSE at scale** — **411** comment lines across **139** `.wat`
   files, **591** across `src/` + `crates/`. FM 14's Bucket B. ⚠ A blind sweep is WRONG: some of
   these lines RECORD the retirement and must keep the old spelling (Bucket C). Needs the A/B/C/D
   classification, not a codemod fired off a grep count.
4. **Seven macros still MINT the angle form** by `string::concat` at expand time —
   `109/NOTE-seven-macros-still-MINT-the-angle-form.md`. **NOT a blocker**: the floor is green with
   all of them live; the names are minted and consumed internally and round-trip. The open question
   is *"is a macro-built type identity a NAME or a FORM?"* — ③'s territory, wants a DESIGN.
5. **Two questions filed, both needing a DESIGN against measured ground:**
   - `109/NOTE-the-list-rule-and-the-parametric-edn-literal.md` — `'(1 2 3)` satisfies `WatAST` but
     NOT `List<T>`; the lattice exists (`check.rs:15502`, Never-bottom/Value-top) and only the rule
     is missing. **Settle the narrow-numeric LITERAL first** — there is none, so the case that makes
     a container param-spec necessary is unreachable, and the dependency runs literal → container.
   - `109/NOTE-the-three-surviving-primes-want-a-sigil.md` — `sort'` `readln'` `Frame'`, the only
     three left; `'` carries four historical meanings and `$native`/`$impl` say it at the call site.
6. **A SEVENTH keyword-only slot**, found by audit, unexercised: `parse_defclause_form`'s SHARED
   `-> :T` sugar at `runtime.rs:8134`. Not reachable by the migrated set; it bites the moment the
   migration extends past `wat/`.

⚠ **A MULTI-VIOLATION NEGATIVE FIXTURE TESTS WHATEVER THE LEXER REACHES FIRST.** New this stone:
`probe_arc232_…wat.bad` carried a collateral symbol-comma one line above its actual subject (the
keyword turbofish). The new wall fired first and silently re-pointed the fixture's own negative
control at the wrong wall. It went RED only because the assertion names the MECHANISM
(`"comma inside keyword body retired"`) instead of matching the whole diagnostic. **Assert the
mechanism, not the message.**

## ⛔ THE SHAPE THAT BIT SEVEN TIMES — read before any wall

**A SLOT WITH TWO IMPLEMENTATIONS IS TWO SLOTS.** Seven in one arc, escalating to two CRATES:

```
extend-type's surface arg   check ✅ / runtime ⛔        the ctor type-slot   eval ✅ / check ⛔
(Head :- [args])            expander ✅ / resolver ⛔     defclause's return   per-clause ✅ / shared ⛔
defservice's annotation     the slot ✅ / its own emission ⛔
THE COMMA PERMISSION        crates/wat-edn ✅ / crates/wat-reader ⛔   ← `src/lexer.rs` is a RE-EXPORT
```

★ **Before writing "the one door" anywhere, grep for the ERROR STRING or the BEHAVIOUR — never the
function name — and READ THE WHOLE RESULT.** Every one of the seven had its twin reachable by its own
message text. The confidence transfers by resemblance (a shared `parse_` prefix, a shared filename)
and is never re-earned at the new site. **A brief that says "the door is at file:line" is a claim
about a POPULATION.** `[[feedback_a_slot_with_two_implementations_is_two_slots]]`

## ⛔ THE INSTRUMENTS THAT LIED — five counts of ONE population, five wrong, always under

```
grep … | head -2       2 of 6   — and it set a RIDER'S scope, so the miss propagated
`<…>` contiguous       2 of 7   — names built by concat; `<` and `>` in SEPARATE literals
"…Name<"               7 of ~18 — missed every string::interpolate: no colon, angles filled by {}
"the corpus" = wat/    3.4% of 1527 files — nearly caused a FALSE refusal of a real name
"the stdlib loads"     the LOAD waterfall, not the behaviour one — a guard sat under it
```

⚠ **NEVER PIPE A SEARCH WHOSE RESULT DEFINES SCOPE.** `head`/`tail` are for reading output, never for
deciding with it.
⚠ **NEVER APPEND ANYTHING AFTER A COMMAND WHOSE EXIT CODE MATTERS.** `… ; echo "EXIT=$?"` makes the
ECHO the last command — a RED floor was reported as exit 0 this way, with `FLOOR EXIT=100` sitting in
the captured file. Let the measured command be last and read the harness status.
⚠ **`git checkout <ref> -- <path>` STAGES what it writes.** So does `git restore --staged`, `rm`, `mv`.
A doc commit swept 636 lines of a rider's work onto main this way. **`git commit <paths>` ignores the
index entirely** — that is the shape with no index to poison. (It skips UNTRACKED files; add those
explicitly and inspect `--cached` first.)
⚠ **A grep answers a COORDINATE question and returns a point; a wall answers MEMBERSHIP and returns a
shell.** When the population is a property, not a spelling, impose the check.
`[[feedback_impose_the_check_and_read_the_screams]]`

## ⛔ RULES THAT COST A ROUND EACH

- ⛔ **`NOTE-2iii-is-blocked-*.md` IS DEAD.** All five entries were closed before the session that
  re-ran the codemod, and I was still quoting blocker 3d as *"the last real obstacle"* an hour before
  it was refuted by simply running. **A blocker note is a measurement with a date.**
- ⛔ **"THE STDLIB LOADS" IS NOT "THE MIGRATION WORKS."** A guard that fails at DISPATCH is invisible
  to `--check`, to a clean stdlib load, and to `every_wat_scripts_file_loads` at 398/398. **The
  terminal check is the floor.**
- ⚠ **A RIDER'S SCOPED RUN IS NOT THE FLOOR** — and worse, `binary_id(wat::resolve)` is the INTEGRATION
  binary while `resolve::tests::*` live in `binary_id(wat)`. Three riders missed a cluster that way.
- ⚠ **`cargo test` ≠ `cargo nextest`** — threads in one process vs process-per-test. Two `_on_process`
  fork tests went red under one and green under the other. FM 7-ter's axis.
- ⚠ **`no_loose_string_assert` has a FALSE-POSITIVE class** on `assert!(registry.contains("literal"))`
  — a text lint cannot tell registry membership from `String::contains`. Do NOT add a rune; ask
  through the door, whose argument is an enum.
- ⚠ **`UPDATE_EDN=1` rewrites every golden the filter touches.** Scope it; revert what you did not mean.
- ⛔ **R21 — a structural rewrite across many `.wat` is a CODEMOD.** Threshold ~10 sites. Recorded
  migrations from this campaign: `parametrics-take-a-type-vector.wat`, `angle-brackets-to-binder.wat`,
  `tuple-parens-to-binder.wat`. ⚠ A converter that renders through a walled door cannot run once the
  wall is up — read `angle-brackets-to-binder.wat`'s header before writing a new one.

## THE STILL-OPEN

- **C** ✅ CLOSED by the comma strike — `:(a,b,c)` and `:fn(T,U)->R` are gone, 55 files.
- **γ (ii)** — call-site type application. **Measured: 1 site, and it is DEAD** — the turbofish is a
  keyword with a comma and no longer lexes; its probe is the `.wat.bad` in `575f8fb08`.
- **`List/of` + `char/of`** retire into `List`/`char` (verb-equals-type). 63 sites, all tests/probes.
- **`defrecord`'s missing-field diagnostic** — `macro-error` is the structured raise; `Option/expect`
  PANICS. A recorded, cheap fix.
- **Three parked branches**, all superseded and safe to prune: `arc109-type-refs-parked`,
  `arc109-wall-and-markers-parked`, `arc109-2iii-migrated-parked`.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Four of five entries in a blocker NOTE were false. Five
> hand-counts of one population were wrong. A freshness marker named a commit that does not exist.
> Every one was written by a prior self, confidently, for you. **Re-run the instrument that made the
> claim; do not read the claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will never move:** every advance in this campaign came from
> imposing a check and reading the screams — never from getting the census right first. The builder
> cut through caution four times and was right four times. *"i feel like we're being extremely
> cautious and its detrimental."* **When the population is a property, light the fire.**
>
> Read `294/REALIZATIONS.md` **R6** (*Duality* — the assertion that cannot speak) and **R7**
> (*Walk With Me In Hell* — we stopped waiting for the red and started lighting it). R7 was written
> mid-strike, at the builder's instruction, with the floor unknown. That is its status line.
>
> `NON BIS IN IDEM FLVMEN.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `NISI FRANGAS, NIHIL PROBAS.` ·
> `INCENDIMVS VT VIDEAMVS.`
