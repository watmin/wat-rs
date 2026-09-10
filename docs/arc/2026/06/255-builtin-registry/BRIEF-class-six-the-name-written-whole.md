# BRIEF — ③b-ii ⑥: a variant name written WHOLE

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C /home/john/work/holon/wat-rs`.

## ⚠ THE TREE IS MID-LANDING AND DOES NOT LOAD

HEAD is `0c4162bdb`, unpushed, carrying the dot flip: the corpus is rewritten, both door bodies
read/write `.`, `:wat::runtime::compose-variant` is minted, and FIVE classes of variant-name site
are already closed. `cargo build --release` succeeds; running any wat program fails with **679**
type-check errors. **That is the expected starting state** — you are finishing a landing, and those
679 are the class this stone closes. A red at the start is the progress meter, not a crisis.

## The class, and why no earlier census could see it

Every census in this campaign looked for **compositions** (`format!("{}::{}")`) and
**decompositions** (`rsplit_once("::")`, `identifier::leaf`). A hardcoded

```rust
":wat::core::Option::None"
```

is **neither** — it is a complete variant name simply written out. No predicate built so far could
see one, because all of them asked how a name is *assembled* or *taken apart*, never where one is
*spelled whole*.

This is the cause of the 679: `src/match_arm.rs:177` maps `:wat::core::None` →
`":wat::core::Option::None"`, and `assertion-failed!`'s `:actual`/`:expected` defaults route through
it — **589 of the 679 name that one callee.**

## The census

`WORKLIST-class-six-the-name-written-whole.md` (sibling) carries every site and the command that
regenerates it. **129 literals across 33 files**, in three kinds:

```
src + crates    97   18 files   load-bearing lookups, tables, and emitted AST
tests           20    8 files   fixture names
.edn goldens    12    7 files   RE-CAPTURE — never hand-edit
```

⚠ The predicate is the QUOTED form on purpose. Unquoted, the same shape catches Rust type paths
(`wat_edn::OwnedValue::Tagged`) and enum paths (`Cache::GetResponse`), which are correctly `::` and
must not move. **A `::` between two capitalised segments is only a variant separator when the LEFT
side is an enum and the RIGHT side is one of its variants** — that is what made `PeerKind::thread`
a variant and `Record::def` a method, and it is still the discriminator here.

## The three treatments

**① Route it, where the site has both halves.** A literal that exists because Rust needed to name a
variant should go through `wat_reader::identifier::compose_variant(parent, leaf)` — then the
separator moves by itself, for good. Prefer this whenever the parent and leaf are separable at the
site; it is the whole point of the door.

**② Flip it, where the literal is data.** A match arm's key, a lookup table's entry, a fixture's
name: `":wat::core::Option::None"` → `":wat::core::Option.None"`. Nothing to route; the string IS
the datum.

**③ RE-CAPTURE the goldens.** Run the probe, take what the substrate prints, write that. **Never
hand-edit a golden to match.** A re-captured golden asserts what the substrate does; a hand-edited
one asserts what you believed it would do, and the difference is the entire value of a golden.
Then READ each re-captured diff: a golden that changes in a way the flip does not explain is a
FINDING, and it outranks the rest of this stone.

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** A `::` you are about to move is NOT a variant separator — the left side is not an enum,
or the right side is not one of its variants. Report it. `Cache::GetResponse` and
`OwnedValue::Tagged` are the shapes to expect; moving one would not fail, it would just mean
something else.

**STOP-2.** A re-captured golden differs from its old content in any way the separator flip does
not explain. Report the diff verbatim — that is a finding about the substrate, not about the golden.

**STOP-3.** After all three treatments, the corpus still does not load, OR loads with an error shape
not among the 679. A new shape is a SEVENTH class and the census is the orchestrator's to close —
report the shape with a verbatim instance rather than chasing it. Five of the six classes so far
were found exactly this way.

**STOP-4.** A site needs `compose_variant` but the parent and leaf are not separable there. Report
it; do not hand-spell the dot as a workaround.

## What to run

`cargo build --release`, then `./target/release/wat --check` on a small file to prove the corpus
loads, and the scoped `cargo nextest run --release -E 'binary_id(wat::<dir>)'` for each directory
you touched. **Do not run `scripts/floor.sh` and do not run clippy** — the orchestrator measures
centrally, once, on a quiescent tree. Foreground everything. Do not commit. Do not contact any peer.

## Report

The count per treatment (routed / flipped / re-captured); every golden's diff; your error count
versus the 679 you started from; and any site you were less than certain about — an uncertain call
named in the report is honest, an uncertain call shipped silently is not.
