# BRIEF — angle brackets are ILLEGAL for types. Chase it to green.

Builder: *"just make angle brackets illegal and chase down what to change… if we just take a hard
stance of 'you shall not express angle brackets for types' we know exactly what to work on."*

Right. **This is arc 109 ③.** The wall goes in and STAYS. You then ride the substrate-as-teacher
waterfall — read the error, fix that site, rebuild, read the next — until the floor is green.

## The wall — two doors, both permanent

```
src/types.rs:4608   parse_declared_name's  Some(lt_index)   → a declaration's own NAME
src/types.rs        the parse_type_expr* family              → every type REFERENCE / annotation
```

Refuse the angle form at both. A `TypeError` naming the raw keyword and pointing at `:- [T …]`.

⚠ Find the second door yourself — `parse_type_expr_with_span` at `:4802` is the entry, and the
`'<'` reads at `:5194` / `:4607` are the candidates. A previous stone put the wall at
`parse_declared_name` ONLY and caught nothing but declarations, because annotations resolve through a
different family. **Both, or the sweep is blind to half its work.**

## ★ The wall does the classification for you — do not build a census

`canonical_callable_name` (`runtime.rs:4191`) strips `<…>` from a CALLABLE name **without touching
the type parser**. So `:wat::spawn::Locus/launch<D,I,O,W>` — a method name with a type suffix — never
reaches the wall and must NOT be converted. It is not a type.

**If it screams, it is a type. If it is silent, it is a name.** That is the whole discrimination, and
it is free.

## The fix rule, per site

```
a REFERENCE / annotation   :Head<A,B>   →  (:Head :- [:A :B])       in parens
a DECLARATION's own name   :Head<A,B>   →  :Head :- [:A :B]         siblings, NO parens
a name built by concat     "…Head<" + args + ">"  →  emit the FORM the same way
```

The corpus already speaks this: 947 `:-` reference forms landed in `2a0d7fa2e`. Copy what is there.

## How to work it

1. Wall in. Build. `target/release/wat --check` a one-line program.
2. Read the error. Fix **that** site. Rebuild. Read the next.
3. The stdlib boot is fail-fast — you get ONE site per round. **That is fine when you are fixing**;
   each round reveals the next. It was only fatal to censusing.
4. When `--check` is clean, run `scripts/floor.sh` — **allowed for this stone**, it is the progress
   meter — and keep going. The fail-count is the meter, not a crisis. Watch it waterfall.
5. Green floor, wall still in, angle form dead.

Expect the first sites at `wat/cache.wat:195` (via `service.wat:626`'s
`string::interpolate "{b}::State{p}"`) and its sibling at `service.wat:634`. The known minting
population is ~18 across `wat/service.wat` and `wat/bracket.wat`, plus whatever the wall finds that
no grep could see.

## STOP triggers

- **STOP-1 — a site that CANNOT be expressed in the new form.** Not "is awkward" — cannot. That is a
  substrate gap and a real finding; report it with the exact form you tried.
- **STOP-2 — the wall changes what a NON-angle keyword accepts or rejects.** Additive refusal only.
- **STOP-3 — you find yourself converting something that did not scream.** It is not a type. Stop and
  report it.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★ | the wall is IN and stays in | both doors refuse `:Head<A,B>`; `git diff src/types.rs` shows both |
| 2★★ | the floor | `scripts/floor.sh` **green** |
| 3★ | callable names untouched | `Locus/launch<…>` still resolves; you converted nothing that did not scream |
| 4 | no angle type left in `wat/` | a probe for `"…Name<"` string literals and `:Head<` keywords returns only PROSE |
| 5 | clippy | 0 under `-D warnings` |

**Row 3 is the one that decides it.** The wall makes conversion obvious; the risk is converting a
NAME because it looked like a type.

## Boundaries

- `src/types.rs` (both doors), and whatever the errors point you at — `wat/*.wat` macros included.
- ⚠ `.wat` edits here are **single-site, error-directed fixes**, not a structural sweep, so R21's
  codemod rule does not bind. If you find yourself doing the same rewrite in 10+ places by hand,
  STOP — that one IS a codemod.
- Do NOT commit, push, stash or amend. Keep the index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

The floor's Summary line verbatim. The waterfall — the fail-count at each round. Every site you
changed and its role. Anything that screamed which you did NOT convert, and why. Whether any site hit
STOP-1. What surprised you.
