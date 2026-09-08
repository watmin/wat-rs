# BRIEF — STONE M2: the corpus takes the map ctor

> Read `DESIGN-STONE-M2-the-corpus-takes-the-map-ctor.md` first, then `wat/fix.wat`'s header
> (line 23, the BOOTSTRAP block). The worklist is measured and on disk:
> `WORKLIST-296-M2-positional-ctor-sites.txt` (288 files, 4,933 sites, with counts) and
> `WORKLIST-296-M2-paths.edn` (the driver's path vector, ready to pipe).

## THE WORK, IN ONE PARAGRAPH

Write `wat-scripts/fixes/positional-ctor-to-map.wat` and run it over every path in the worklist, so
that every positional variant construction becomes a map naming its declared fields in declaration
order — `(:ns::E::V a b)` → `(:ns::E::V {:f1 a :f2 b})`, and `(:ns::E::V)` → `(:ns::E::V {})` for a
unit variant. **The codemod ASKS `type-of` for field names; it does not read them out of source
text.** Because the codemod is a `.wat` program and the current binary refuses the corpus it must
read, the run happens through the dance below. When it finishes, `./target/release/wat <any file>`
runs again and the floor returns.

## READ IN ORDER — the rooms

```
wat/fix.wat:23                  THE BOOTSTRAP BLOCK. Read before anything. Its header records that
                                a prior self abandoned this tool because the dance was not written
                                down. The dance in this brief SUPERSEDES its step 1 (see below).
wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat
                                ★ THE SHAPE TO COPY. The codemod RELAND 4 taught to ASK `type-of`.
                                Read how it extracts top-level declarations, evaluates them with
                                `:wat::eval-with-defs!`, and reads field names back — that is the
                                exact capability this stone needs.
docs/arc/2026/04/109-kill-std/SCORE-STONE-the-match-arm-RELAND-4-ask-reflection.md
                                the measurement proving decls-only `eval-with-defs!` + `type-of`
                                answers a defservice-GENERATED response type's fields, in order.
wat-scripts/fixes/positional-to-kwargs.wat
                                ⚠ READ IT TO SEE WHAT NOT TO DO. It is reflection-free and observes
                                def-forms as bytes; its `def-head?` (line 17) lists defrecord /
                                holon::defrecord / defstruct / defholon and NEVER defenum. Its
                                strategy is blind to the 32 generated enums.
wat/fix.wat  (fix-source · ast-span · ast-end-span · fix-text-apply)
                                the span-faithful edit machinery. Edits are token-span inserts;
                                surrounding whitespace survives.
```

## ⛔ THE DANCE — THIS SUPERSEDES `wat/fix.wat:23` STEP 1

Stone M is **committed** (`c8ef970fb`), not dirty, so there is nothing to stash. Restore from the
commit instead — idempotent, re-runnable, and nothing to drop:

```bash
1.  git checkout HEAD~1 -- src/check.rs src/runtime.rs src/types.rs src/record/construct.rs
2.  cargo build --release
3.  # DRY RUN FIRST — copy 3-4 worklist files to /tmp, run the codemod on the copies, `diff` them.
    #   Verify: field names correct and IN DECLARATION ORDER; unit variants get `{}`; accessors
    #   (`:ns::E/field`), type references (`<- :ns::E`) and ALREADY-MAP forms are untouched.
    # THEN the real run over EVERY path:
    cat docs/arc/2026/06/296-diagnostics-fully-edn/WORKLIST-296-M2-paths.edn \
      | ./target/release/wat ./wat-scripts/fixes/positional-ctor-to-map.wat
4.  git checkout HEAD -- src/check.rs src/runtime.rs src/types.rs src/record/construct.rs
5.  cargo build --release
```

Run every build and every codemod invocation in the **foreground** and wait for it; your turn ends
when the numbers are in your hands.

## THE DONE-WHEN — FIXTURE-LOCAL ERRORS AT ZERO

For any corpus file `f`, the count of errors whose `:file` is `f`:

```bash
./target/release/wat --check "$f" 2>&1 | grep -o "\"$f\"" | wc -l      # must be 0
./target/release/wat wat-scripts/scratch-pad/<any>.wat; echo "EXIT=$?"  # must not be 3
```

**Do not use a bare exit code as the bar.** While the world is red, every `--check` exits 1 for
reasons that have nothing to do with the file — that is exactly how M's probe rows went green on
`1 == 1`. Count what is LOCAL to the file.

Also in this stone, one change serving both jobs: **`tests/types/probe_arc296_enum_map_ctor.rs`'s
`check()` becomes a fixture-local error count** instead of an exit code, and its five rows must then
be honestly green.

## STOP TRIGGERS

- **STOP-1 — the codemod reads field names from SOURCE TEXT.** 32 of the 60 affected enums have no
  `defenum` anywhere; they are `defservice`/`defsurface` output. A text-derived map migrates the 28
  it can see and leaves the build broken. ASK `type-of`.
- **STOP-2 — a hand-edited `.wat` site.** R21. If a site resists the codemod, that is a finding about
  the codemod; report it with the file and form.
- **STOP-3 — `src/` is edited to make the corpus pass.** M's refusal is correct and is not in scope
  here. Touch only the four files named in the dance, and only by `git checkout`.
- **STOP-4 — a bare exit code is used as an acceptance bar.** See THE DONE-WHEN.
- **STOP-5 — a worklist path is skipped.** `wat/fix.wat:23` warns a missed file breaks the build. If
  a path cannot be migrated, STOP and report it; do not proceed with a partial corpus.
- **STOP-6 — variant SPELLINGS are changed.** This stone migrates the construction GRAMMAR only.
  `:wat::core::Some` stays `:wat::core::Some`; its retirement is the next stone.
- **STOP-7 — the floor is run.** It is 4+ minutes and it is the orchestrator's. Your acceptance is
  the fixture-local count above plus the probe's five rows.
