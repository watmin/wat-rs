# BRIEF — make the angle form ILLEGAL, and let the screams be the census

Builder, 2026-08-23: *"how about we just make parametrics via angle brackets illegal and just make
every heretic scream — set them ablaze … that's your census."*

**This stone's DELIVERABLE IS THE LIST, not a green floor.** You are lighting a fire and writing down
what burns. A red floor is the expected outcome and the correct one.

## Why a wall and not a census

I tried to count these by hand **five times today and was wrong five times**, each time because the
instrument selected a narrower population than the claim:

```
grep … | head -2            2 of 6 sites   (and it set a rider's scope)
`<…>` contiguous            2 of 7         (names are built by string::concat, `<` and `>` in
                                            SEPARATE literals)
`"…Name<"`                  7 of ~18       (missed string::interpolate "…Peer<{o},{r}>" — no
                                            leading colon, `<…>` filled by `{}`)
"the corpus" = `wat/`       3.4% of 1527 files
"the stdlib loads"          the LOAD waterfall, not the behaviour one
```

★ **And the decisive reason a grep can never finish this job: most of these names do not exist in any
file.** They are assembled at expand time by `string::concat` / `string::interpolate` and handed to
`keyword-node`. A wall at the parser sees the minted name exactly as it sees a written one.
`[[feedback_impose_the_check_and_read_the_screams]]`

## The one door

`src/types.rs:4607-4608` — the single place a keyword becomes a parametric type:

```rust
// Split at first '<' if present.
match stripped.find('<') {
    None => Ok((raw, Vec::new())),
    Some(lt_index) => { …parse the angle params… }
}
```

**Turn the `Some` arm into a refusal.** A `TypeError` naming the offending raw keyword and pointing at
the `:- [T …]` form. It fires for written AND minted names because both arrive here as a keyword
string — that is the whole reason this is the right wall.

⚠ `runtime.rs:14418`'s `split_type_params` is a SIBLING that also splits on `'<'`, with 9 callers via
`split_type_params_pub`, plus `check.rs:12048`'s `is_type_bracket_candidate` (9 callers), and
`types.rs:3092`/`3154`/`5194`. **Do NOT change those in this stone.** They are consumers of an
already-split name; the wall goes on the PARSE door only. If leaving them makes the wall vacuous,
that is STOP-1.

## What to do with the fire

1. Put the wall in. Build.
2. `scripts/floor.sh` — **you may run the floor for this one stone**, because the floor IS the
   instrument. Capture it; do not fix anything yet.
3. **Write the census from the ARM**, one row per distinct site:

   | site (`file:line`) | the raw keyword refused | ROLE |
   |---|---|---|

   Role is one of — and this classification is the actual deliverable:
   - **ANNOTATION** — the name lands in a param/return/field type slot. Converts to the form.
     (`bracket.wat:448-449` `[self <- ~runner-self-kw ctx <- ~ctx-ty-kw]` is the worked example.)
   - **DECL-NAME** — the name is a declaration's own name. RULED to emit the form; not yet shipped.
   - **IDENTITY / CALLABLE** — the name is a lookup key or a callable. **A form is not a name.**
     `"wat::spawn::Locus/launch<"` is one of these — a METHOD name with a type suffix, not a type
     reference at all.
   - **PROSE** — inside an error message. Two exist; leave them.

4. Then **revert the wall** and hand me the census. Do not attempt the conversions.

## STOP triggers

- **STOP-1 — if the wall is VACUOUS** (floor stays green, or only a handful scream). That means the
  angle form is being consumed somewhere that never reaches this door, and the wall is in the wrong
  place. STOP and report where you think it actually lands. A wall that cannot fail is not a wall.
- **STOP-2 — if a scream is not classifiable** into the four roles above, STOP and report it. A fifth
  role is a finding worth more than the census.
- **STOP-3 — do NOT convert anything.** Not one site. The classification is the deliverable and a
  conversion made mid-census corrupts the count.

## Boundaries

- `src/types.rs`, the one `match` arm. Nothing else — and it is REVERTED before you finish.
- Do NOT touch `split_type_params`, `is_type_bracket_candidate`, or `types.rs:3092/3154/5194`.
- Do NOT hand-edit any `.wat`. R21, and this stone edits no corpus at all.
- Do NOT commit, push, stash, revert-other-work or amend. Keep the index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

**The census table, complete**, one row per distinct site with its role. The floor's Summary line
verbatim and the failure count. Any site you could not classify. Whether the wall fired on minted
names as well as written ones — name one of each. Confirmation that `git status` is clean when you
finish (wall reverted, nothing staged). What surprised you.
