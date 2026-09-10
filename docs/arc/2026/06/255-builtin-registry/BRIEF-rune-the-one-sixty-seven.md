# BRIEF — ③a-iii: rune the 167, one closed category at a time

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; a path containing `.claude/worktrees/`
is harness state — use `git -C /home/john/work/holon/wat-rs` for git.

You own **ONE AREA** of `WORKLIST-the-167-screams.md` (this brief's sibling). Your area is named in
your spawn message. Every line in it is listed there with its file, line number, hit kind, and the
verbatim source — locate by CONTENT, not by line number, since a `+1`/`-1` drift is normal.

## The work, in one paragraph

`tests/lint/one_variant_separator.rs` is a new wall: the `::` between an enum and its variant is
spelled in exactly one file, `crates/wat-reader/src/identifier.rs`. It fires on 167 lines that
spell a separator inside a file that talks about variants.

⛔ **DO NOT ASSUME THEY ARE ALL NON-VARIANT.** An earlier version of this brief said so, deduced
from "③a-ii routed all twenty-one." **It was false** — four riders fired STOP-1 and five more
variant sites were in the list (`wat-doc/src/lib.rs:1024`, two in `tests/reflection/`,
`rete/expr_ir.rs:751`, `runtime.rs:3265`). **Read each line and decide.** Your job is to give each
line in your area a co-located rune naming which non-variant thing its separator is — and to fire
STOP-1 on any line that is not one.

## The rune

On the offending line, or the line immediately above it:

```rust
// rune:lint(one-variant-separator, <category>) — <why this separator is not a variant separator>
```

The wall **validates the category against a closed list** and refuses anything else:

```
namespace     the "::" separates namespace segments             :wat::core::foldl
type-path     it composes/decomposes an ENUM's OWN type path    format!("{}::Op", surface.name)
display       it renders a name into prose a PERSON reads       "variant {t}::{v} declares …"
edn           it translates "::" <-> "." for EDN rendering      ns.replace("::", ".")
not-a-name    the string is not a wat name at all               DirEntry::path(), a doc string
```

⛔ **`variant` is not a category** — the wall rejects a rune claiming it.

★ **`display` is the one that will be read again.** It marks every site that must switch to `.`
when the separator flips, and those sites share no call shape — this category is the only thing
that will ever enumerate them. When a `format!` puts an enum and its variant into a message a human
reads, it is `display`, even though it looks exactly like a composition.

⚠ **`type-path` vs `variant`.** `format!("{}::Op", surface.name)` composes the ENUM's own name —
the enum lives at `S::Op` and its variant is `Bump`, so the post-flip spelling is `S::Op.Bump` and
that `::` stays. That is `type-path`. A `::` with a VARIANT on its right is not runeable at all.

## The reason must earn the rune

One clause, specific to the site. *"not a variant"* is not a reason — it restates the category.
Name what the separator actually joins: *"namespace and leaf of a fully-qualified verb name"*,
*"the surface's Op enum's own path; its variants hang off this"*, *"user-facing arity error; the
reader sees the wat spelling"*.

## Blast radius

The lines in your area and nothing else. **Add comments only.** No code changes, no reordering, no
reformatting, no touching another area's files. If a line is already long, put the rune on the line
ABOVE it rather than wrapping the code.

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** A line in your area **is** a variant separator — its `::` has an enum on the left and
one of that enum's variants on the right. Do NOT rune it and do NOT route it. **Report it.** It
refutes the census that stone ③a-ii closed, and that refutation is worth more than the rest of your
area combined.

**STOP-2.** A line needs a category that is not on the closed list. Report the line and what it
actually joins — the list is short on purpose, and a sixth category is a ruling, not a rider's call.

**STOP-3.** You cannot write a reason without guessing what the code does. Report the line. A rune
with an invented reason is worse than a red.

**STOP-4.** Your area's worklist does not match the file — the content named is not there. Report
it verbatim; do not hunt for something similar.

## What to run

Nothing. **Do not run cargo, `scripts/floor.sh`, clippy, or the wall itself** — the orchestrator
measures centrally, once, on a quiescent tree, and a build from here contends with every sibling
rider on one `target/` lock. Run every command you do use in the FOREGROUND and block on it. Do not
commit. Do not contact any peer.

## Report

Your area, line by line: the category you gave it and your one-clause reason. Then anything that
surprised you, and any line you were less than certain about — an uncertain rune named in the
report is honest; an uncertain rune shipped silently is not.
