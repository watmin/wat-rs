# STONE B-i — the corpus moves: `:wat::core::{i64,f64}::` → `:wat::{i64,f64}::`

DRAWN + BRIEFED 2026-08-26 against `1333e90d0`.
DESIGN: `DESIGN-STONE-the-numerics-get-their-homes.md`. Homes built by **A-i** (`b2d10158f`) and
**A-ii** (`1333e90d0`) — all 36 new names are registered and live.

## Why B is split, and what B-i is

`:wat::rete::core::{i64,f64}::` — the rete DSL clone — is **B-ii**, not this stone, because it is a
different kind of work: `src/rete/vocabulary.rs` holds **paired** names, 70 numeric entries of

```rust
rete_name: ":wat::rete::core::i64::+",   // the spelling .wat rete rules USE — moves with the corpus
core_name: ":wat::core::i64::+",         // points at the core op — must NOT move until Stone C
```

so the rete corpus cannot move without a paired Rust edit. **B-i needs ZERO Rust changes** — the new
core spellings are already registered, so a renamed `.wat` file resolves through the registry the
moment it is written. That difference is the whole reason to do core first.

## The work

```
:wat::core::i64::   ->  :wat::i64::      1441 occurrences
:wat::core::f64::   ->  :wat::f64::       172 occurrences
                                         ────
                                         1613 across 429 .wat files
```

⛔ **`.wat` ONLY.** Every `.rs` occurrence is the OLD spelling's own machinery — dispatch arms, type
schemes, purity rulings, the rete vocabulary — and it must survive until Stone C retires the old
name. If you find a `.rs` site that is a *caller* rather than an implementer/classifier/test-of-the-
old-name, **that is a finding**: report it, do not migrate it.

## ⛔ THE TOOL — a wat-fix RULES codemod, and nothing else

R21: a structural rewrite across many `.wat` files is a **wat-fix codemod — wat rewriting wat**.
Not hand-edits. Not sed. Not python. Write
`wat-scripts/fixes/rename-core-numerics-to-their-homes.wat`.

**COPY `wat-scripts/fixes/rename-core-string-to-string.wat`** — it is this exact migration one
namespace over, and its header records the traps. Two of them are load-bearing:

1. **`rename-keyword-prefix` is a SILENT NO-OP for a `::`-terminated open prefix.** The parked
   `wat-scripts/scratch-pad/BLOCKED-rename-core-string-to-string.wat` is the evidence. **The rules
   form is the one that works.**
2. **KEYWORD ONLY.** `(:wat::rete::where (:wat::rete::string::= ?k "keyword"))` is mandatory.
   `wat/grep.wat`'s `Named` fact also fires for the `"string"` kind, and a string literal's span
   covers its surrounding quotes while its `name` does not — splice the replacement into that span
   and you corrupt the literal into unquoted keyword syntax.

Two entry points, one rule set: `wat --grep <file>` is the finder (counts Matches, writes nothing);
`wat <file>` is the applier.

## ★ THE SAFETY PROPERTY — structural, not a census

`:wat::core::i64` **the TYPE** has **6,670** `.wat` occurrences that head for arc **251**'s
`wat.type/` and MUST NOT MOVE. They are safe **by construction**: the rule matches
`starts-with? ?n ":wat::core::i64::"`, and the bare type keyword is SHORTER than that prefix, so it
cannot match. **The trailing `::` is the entire discrimination.** The string codemod's header makes
the same argument for `:wat::core::String` vs `:wat::core::string::`; do not weaken it.

Still prove it: the type count must be **identical before and after**.

## ⚠ EXCLUDE TWO FILES, AND THE REASON IS THE POINT

```
wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat
wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat
```

These are A-i's and A-ii's probes, and they **deliberately assert BOTH spellings** — that is their
entire job, and it is the only artifact proving the old names still work. Rewriting their old halves
would silently convert them into probes that test one spelling twice. The applier takes an explicit
path list, so simply do not list them. **Say in the codemod's header that they are excluded and
why**, so the next reader does not "fix" the omission.

(They also carry the only `.wat` occurrences of `max-of`/`min-of` anywhere in the corpus — which is
why B-i needs no shape-rewriting rule at all. See STOP-2.)

## Your role

cwd `/home/john/work/holon/wat-rs`; run `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND, blocking. **You may not spawn sub-agents.** Do not commit, push, stash, revert, or
`git checkout`; `git stash@{0}` must never be touched.

You may run `cargo build --release`, `./target/release/wat --grep <f>`, `./target/release/wat <f>`,
`./target/release/wat --check <f>`, and single named tests. **Not** the floor, **not** clippy.

## The method — dry-run BEFORE you touch the corpus

1. Write the codemod. `--check` it.
2. `--grep` it over the real corpus. The Match count is your predicted edit count — **compare it to
   1613 and explain any difference before proceeding.**
3. **Copy the corpus to `/tmp`, apply there, and `diff -r` against the original.** Read the diff.
   Confirm every hunk is a prefix rewrite and nothing else moved.
4. Only then apply in place.
5. **Re-run the applier. It must report ZERO matches** — idempotence is the property, and the second
   run is the proof.

## STOP triggers — each rejects

1. **STOP-1 — the `--grep` count and the census disagree** and you cannot explain why. Report both
   numbers and the command that produced each. Do not apply.
2. **STOP-2 — a `.wat` site needs a SHAPE change, not a prefix rewrite.** `max-of`/`min-of` are
   variadic under the new spelling and take one Vector under the old (builder's ruling 2026-08-26:
   *"keep variadic - clojure is the destination"*). Outside the two excluded probes the corpus has
   **zero** such sites — so if you find one, the census was wrong and I want it before any edit.
3. **STOP-3 — the TYPE count moves.** Any change to `:wat::core::i64`-the-type is arc 251's
   territory and a bug in your rule.
4. **STOP-4 — a room's line number does not hold.** Written against `1333e90d0`.

## Acceptance — every row derives its bar

```bash
# 1. the ops are gone from .wat — except the two probes, which keep BOTH. BAR: 0.
git grep -oE ':wat::core::(i64|f64)::' -- '*.wat' ':!docs' \
  ':!wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat' \
  ':!wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat' | wc -l

# 2. the new spellings are there. BAR: 1613 (or your own explained number).
git grep -oE ':wat::(i64|f64)::' -- '*.wat' ':!docs' | wc -l

# 3. THE TYPE DID NOT MOVE — measure BEFORE and AFTER with this one command, prove equality.
git grep -oF ':wat::core::i64' -- '*.wat' ':!docs' | wc -l   # both ends; 8111 before, and the
                                                             # ops leaving means it must DROP by
                                                             # exactly 1441 to 6670 — derive it

# 4. idempotence — the second apply changes nothing. BAR: zero matches.
./target/release/wat --grep wat-scripts/fixes/rename-core-numerics-to-their-homes.wat

# 5. the corpus still loads and type-checks (the gate that walks wat-scripts/).
cargo test --release --test lint every_wat_scripts_file_loads_on_the_current_runtime
cargo build --release
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **The `--grep` count vs the 1613 census**, and any difference explained.
- **The `/tmp` dry-run diff**: how many files, how many hunks, and confirmation you read it and every
  hunk is a prefix rewrite.
- The idempotence proof — the second run's zero.
- The codemod's full text.
- Anything the brief got wrong. What you did NOT do, and why.
