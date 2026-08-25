# BRIEF — STONE: an edit carries what it CLAIMS to replace

DESIGN: `docs/arc/2026/06/282-wat-fix-over-rust/DESIGN-STONE-an-edit-carries-what-it-claims-to-replace.md`
— read it whole, and read **THE ONE PINNED CONTRACT** twice.

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first and stay there.

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Every command in the
FOREGROUND, blocking. Your turn ends when the numbers are in your hands.

**You may not spawn sub-agents.** Do not commit, push, stash, revert, or `git checkout`. There is a
`git stash@{0}` that must never be touched. One untracked file,
`wat-scripts/fixes/rename-four-families-to-their-homes.wat`, belongs to a blocked stone — **leave it
exactly as it is** (it is not in your 31).

You may run `cargo build --release` and single named tests. **Not** the full floor, **not** clippy.

⚠ `wat/fix.wat` and `wat/lint.wat` are stdlib — `include_str!` at Rust-compile time, so each change
needs a ~19s rebuild before it is live. The other 29 files are under `wat-scripts/` and
`./target/release/wat --check <file>` answers in ~0.15s. That asymmetry is your loop.
⚠ A file under `wat/` cannot pass a standalone `--check`. Not your red.

---

## The work in one paragraph

`fix-text-apply` takes `(offset, old-len, new-text)`. It knows how many characters to overwrite and
never learns what it believes is there, so it cannot tell a correct edit from a catastrophic one.
Change the edit to `(offset, old-text, new-text)` and make apply **verify before it splices**.
`old-len` stops being carried — it is `(length old-text)`, and two copies of one fact is how they
come to disagree.

---

## ⛔ THE THING TO GET RIGHT, and it is the one way to fail while everything goes green

**`old-text` is the rule's CLAIM. It must NEVER be sliced out of the source at the edit's own offset.**

Work the corruption this stone exists to stop. A reader-synthesized `:wat::core::char/of` keyword had
a span covering `\a` — two columns:

```
off      = the span's start offset
old-len  = fix-text-span-len(start, end, lines)  =  2     ← CORRECT. It IS the span width.
new-text = ":wat::core::char"                             ← 16 chars
```

`old-len` was never wrong. The splice replaced exactly the two characters the span named. **The bug
was that the rule believed it was replacing `:wat::core::char/of` and was actually replacing `\a`.**

So an `old-text` derived from the span would be `"\a"`, would match itself, and the splice would
proceed. **The check would guard nothing.** The disagreement between *what the rule claims* and *what
the source holds* is the entire signal.

### The claim is already written down and thrown away

`wat-scripts/fixes/rename-core-string-to-string.wat` asserts two captures and reads one:

```wat
(:wat::grep::Capture :name "old" :value ?n)      ← the belief. Captured. NEVER READ.
(:wat::grep::Capture :name "new" :value …)       ← read via :rn::second-capture
```

For every rules-based codemod, `old-text` is the **first** capture. It is already there.

---

## The cascade, measured

```
341  occurrences of the edit-tuple type annotation
 31  files:  wat/fix.wat (52) · wat/lint.wat · 28 × wat-scripts/fixes/ · 1 × wat-scripts/lib/
 43  call sites of fix-text-apply
```

The middle value's binding, classified — **this is a derivation, not a judgement per site**, because
*the length expression names its own subject, and the subject is the old text*:

| how `old-len` is bound | count | becomes | vacuous? |
|---|---|---|---|
| `(:wat::string::length X)` | 29 | `X` | **no** — `X` is the rule's belief |
| `(:wat::fix::fix-text-span-len …)` | 7 | ⚠ the rule's CLAIM, not the span text | **would be** if you use span text |

★ **The 7 are where this stone is won or lost.** For a rules-based codemod the claim is the `"old"`
capture. If a site has no recoverable claim, that is STOP-2 — report it, do not paper it with span
text.

---

## The form

```wat
(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
```

`fix-text-apply`: before splicing, compare `(subs src off (+ off (length old-text)))` against
`old-text`. On disagreement, **raise** — naming the offset, what was claimed, and what is actually
there. A codemod author must be able to read that message and know which rule lied.

Add the sibling door beside `fix-text-span-len`:

```wat
(:wat::core::defn :wat::fix::fix-text-span-text
  [start-span <- … end-span <- … lines <- … src <- :wat::core::String] -> :wat::core::String)
```

⚠ **It is a loaded gun.** It is for edits whose subject genuinely IS the span — deleting a region,
reflowing whitespace — never for a rename. Filling `old-text` from it in a rename is STOP-1.

---

## ORDER — this is a STASH-DANCE, and doing it out of order costs you the run

The codemod that migrates the codemods is itself a codemod calling `fix-text-apply`.

1. Write `wat-scripts/fixes/edits-carry-the-old-text.wat` against the **OLD** API — it still exists
   and still works. Read `wat/fix.wat`'s header BOOTSTRAP / STASH-DANCE note first.
2. **Dry-run on a `/tmp` copy and diff byte-level.** Every hunk must be the type annotation or the
   binding, and nothing else.
3. Apply to the 31 files, **including itself** — a recorded migration only has to type-check
   afterwards, it never runs again.
4. **THEN** edit `wat/fix.wat` (the `fix-text-apply` definition, its own internal tuple sites, and the
   new `fix-text-span-text`) and `wat/lint.wat`.
5. Rebuild once. `--check` the 29 in a loop; fix what it names.

Between 3 and 5 the tree does not type-check. **That is expected**, it is why this lands as one
atomic commit, and it is not a reason to stop.

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — you are about to fill `old-text` from the source at the edit's own offset.** That makes
   the check compare a slice against itself. If a site seems to need it, report the site; do not do it.
2. **STOP-2 — a site has no recoverable claim.** The rule computes a length from a span and nothing
   in scope says what it believes is there. Report the file and the binding; do not invent a claim
   and do not fall back to span text.
3. **STOP-3 — the dry-run diff touches anything other than a type annotation or a middle binding.**
   Report the hunk verbatim.
4. **STOP-4 — a recorded migration stops being idempotent.** `rename-core-string-to-string.wat` must
   still re-run to zero changes over the corpus. If it now raises, that raise is either a REAL find
   (a latent bad edit the old apply would have made) or a bug in your migration — report which, with
   the message.
5. **STOP-5 — a room's line number does not hold what this brief says.** Written against `c80aa5860`.

---

## Acceptance rows you must produce yourself

**★ ROW 1 IS THE CONTROL AND IT IS THE POINT OF THE STONE.** Build a deliberate liar: a rule that
matches a `~` node by `Span` (NOT `Written`), claims `":wat::core::unquote"` as `old-text`, and tries
to rewrite it. `fix-text-apply` must **raise**, naming the offset, the claim, and what is really
there. Before this stone that edit silently replaces `~` with a nineteen-character name. **Run it and
paste the message.** Without this row the stone is unfalsifiable and a vacuous check passes every
other row.

Then: all 31 `--check` clean · `every_wat_scripts_file_loads` green ·
`rename-core-string-to-string.wat` re-runs to **0** changes with a byte-identical tree ·
`fix-text-span-len`'s remaining caller count.

## Report back with

- **Row 1's raise message, verbatim.** First thing in your report.
- The dry-run diff shape: files, hunks, and your confirmation that every hunk is annotation-or-binding.
- The 7 `fix-text-span-len` sites, one line each: what claim you gave it and where that claim came
  from. This is the part I will read most carefully.
- `fix-text-span-len`'s caller count after the migration.
- Anything the brief got wrong. It was written by someone who has been wrong about this corpus
  repeatedly today.
- What you did NOT do, and why.
