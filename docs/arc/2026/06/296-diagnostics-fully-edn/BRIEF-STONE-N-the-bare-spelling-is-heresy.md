# BRIEF — STONE N: the bare variant spelling is heresy

> Read `DESIGN-STONE-N-the-bare-spelling-is-heresy.md`, then `wat/fix.wat`'s BOOTSTRAP block
> (line 23). This SUPERSEDES the M2 reland — **do not run `unwrap-alias-edits`**; the rename repairs
> those 118 sites and an unwrap would undo work that is about to become correct.

## THE WORK, IN ONE PARAGRAPH

Make the five illegal spellings REFUSE with a message naming their replacement. Build. **The
refusal's own errors are the worklist** — every site screams with its file and line, which is a
form-tree census no grep can match. Then, through the dance, rename the corpus to the qualified
variant FQDNs and finish M2's remaining wrap. When it ends, one spelling and one construction
grammar exist, the 118 damaged sites are correct, and the toolchain runs.

## READ IN ORDER — the rooms

```
src/match_arm.rs:159-165        builtin_variant — the four arms, each `BARE | QUALIFIED`. The bare
                                alternative is the heresy. Make it REFUSE, naming the replacement.
src/runtime.rs:1700 · 8668      `:None` || bare || qualified — the legacy arm rides here too
src/runtime.rs:8761 · 8792 · 8820 · 13427-13429     Some / Ok / Err doors
src/declare/register.rs:1155 · 1167                 registration
src/check.rs  (M's infer_enum_map_ctor + its positional refusal)
                                the SHAPE to copy for the refusal message — it names the map form,
                                which is why it teaches instead of merely rejecting.
wat-scripts/fixes/positional-ctor-to-map.wat
                                M2's codemod, uncommitted on this tree. It ASKS type-of (11 hits).
                                EXTEND it; do not rewrite it. Its gensym->FQDN table is what left
                                23 `service.wat` template heads behind — DERIVE that table, do not
                                hand-list it.
wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat
                                the reflection precedent (RELAND 4's "the codemod can ask now")
```

## THE ORDER INSIDE THE STONE

```
1  the wall          make the five spellings refuse, message names the replacement
2  build             cargo build --release
3  READ THE SCREAMS  --check the corpus; the errors ARE the worklist. Record the true counts —
                     they will not equal the DESIGN's grep estimates, and the screams are right.
4  the dance         git checkout <pre-M commit> -- src/check.rs src/runtime.rs src/types.rs \
                                                     src/record/construct.rs src/match_arm.rs \
                                                     src/declare/register.rs
                     cargo build --release          # old checker: accepts everything
                     <dry-run on /tmp copies, diff>
                     rename codemod over the screams' worklist
                     THEN the wrap codemod (positional-ctor-to-map.wat, extended)
                     git checkout HEAD -- <the same files>
                     cargo build --release
5  verify            fixture-local errors 0; a corpus program RUNS (exit != 3)
```

★ **Rename BEFORE wrap.** After the rename every head is a three-segment variant FQDN, so the wrap
applies with no exception — which is the whole reason this stone precedes the wrap's completion.

## THE DONE-WHEN

```bash
./target/release/wat <any corpus .wat>; echo "EXIT=$?"          # NOT 3
./target/release/wat --check "$f" 2>&1 | grep -o "\"$f\"" | wc -l   # 0, for every worklist path
```

Fixture-local counts only. **A bare exit code is not a bar** — while the world is red every
`--check` exits 1 for reasons unrelated to the file, which is exactly how M's probe rows went green
on `1 == 1`.

## STOP TRIGGERS

- **STOP-1 — the worklist is a GREP.** The DESIGN's numbers are estimates and are labelled as such.
  Build the wall, read its errors, and migrate what SCREAMS. A site a grep cannot see is precisely
  the site this method exists to catch.
- **STOP-2 — `unwrap-alias-edits` is run.** Measured: `(:wat::core::Option::Some {:value 1})` has
  **0** fixture-local errors. The rename repairs those 118; unwrapping undoes correct work.
- **STOP-3 — the refusal is a silent fall-through.** It must name its replacement. A bare spelling
  that merely stops resolving is how `:wat::core::None` survived four arcs.
- **STOP-4 — the gensym→FQDN table is hand-listed.** That is what left 23 `service.wat` template
  heads behind. Derive it.
- **STOP-5 — a worklist path is skipped.** `wat/fix.wat:23`: a missed file breaks the build. If a
  site resists, STOP and report it with the file and the form.
- **STOP-6 — the Rust arms are left admitting the bare spelling.** A rename that leaves the door
  open is not an annihilation. The bare form must be UNREPRESENTABLE at the end of this stone.
- **STOP-7 — the floor is run.** Orchestrator's, and it is 4+ minutes. Your acceptance is the
  done-when above plus the five probe rows.
