# BRIEF — STONE 255.1c-taxonomy · amend the subject line, append five `Category` variants

Prerequisite to the kernel carve. `:wat::runtime::Category`'s ten variants were derived from
pure-data and stdio families; the `:wat::kernel::` tier is 44 verbs of a different kind and the
taxonomy runs out. This stone adds names. **It re-categorizes nothing.**

## ⛔ READ THE STONE — AND KNOW THAT IT ARGUES WITH ITSELF ON PURPOSE

`DESIGN-STONE-255.1c-taxonomy-the-kernel-tier-exhausts-Category.md`.

★ **It ends in an `⊘ AMENDED` block, and the amendment OVERRULES the `:Message` section above it.**
The original section justifies `:Message` from transport ("two of four tiers have no stream"); a
second `intueri` cast **refuted that argument** as a HOW, not a WHAT — the same test the stone itself
used to refuse a `:Mutate` variant. The conclusion survived; the reasoning did not.

**So: the `:Message` prose you ship comes from the AMENDMENT, not from the earlier section.** If you
find yourself writing "two of four tiers are raw Value pass-through" into the enum, you have used the
refuted argument. Read the file to the end before you write a line.

## The target — one file

**`wat/runtime-meta.wat`.** Two edits, both inside it.

⚠ **This file is read at COMPILE TIME by `wat_enum_derive::wat_enum_from!`** — variants, order, and
the `;;` prose on each become the generated Rust enum and its `///` docs. **The prose IS the
deliverable.** A malformed `defenum` fails the build, not a test.

### Edit 1 — the subject line (`:58`). Do this FIRST.

Today:
```
;; Category — what kind of computation an intrinsic or special form performs.
```
It is already false: `:Declaration` isn't a computation (verified — `derive`, `declare-acronyms`,
`use!` are all `Ok(Value::Unit)` runtime no-ops, each saying so in its own comment). Replace with the
stone's amended wording — *what a verb DOES in the language; sometimes a runtime computation,
sometimes a program-level registration, sometimes a contract discharged at check time.*

⛔ **This edit is load-bearing, not cosmetic.** `:CheckGate` is dishonest under the old line — that
was the ward's whole objection, and shipping the variant without the amendment leaves the header's
own rule false on its face.

### Edit 2 — append five variants at the END of the `defenum` (`:84–120`)

**APPEND ONLY.** The last variant today is `:Declaration)` — the closing paren moves to the new last
variant. Inserting mid-list renumbers the generated enum.

Order: `:Resource` · `:Message` · `:Ambient` · `:Project` · `:CheckGate`.

Each gets a `;;` block above it, in the file's existing voice. The stone gives the substance for all
five; **`:Message`'s exact prose is in the amendment block** and includes a deliberate closing clause
naming the transport argument as refuted — keep it, it is what stops this being re-litigated.

Match the house style you can see in the file: state the DOING, give examples, and name at least one
thing the variant is NOT (every existing variant does this — `:Probe` says "NOT 'returns a bool'",
`:Arithmetic` says "NOT string concatenation", `:Io` says an encoding step does not make it
`:Transform`).

### Edit 3 — one clause onto `:Io`'s existing prose

`:Io` is **correct and stays**. Add only the contrast clause from the amendment:
*"Contrast `:Message`: a peer is a typed value the caller holds a handle to, not an OS stream."*
Change nothing else about it.

## Prove it

- `cargo build --release` — the derive macro consumes the file at compile time, so a green build is
  the structural proof that all fifteen variants and their prose landed.
- Confirm the generated enum carries the new variants: `grep -n 'Resource\|Message\|Ambient\|Project\|CheckGate' crates/wat-doc/src/lib.rs` will NOT show them (they are generated, not literal) — instead show them from a build artifact or a `--check` of any file that names `:wat::runtime::Category`. **If you cannot find an honest instrument for this, say so rather than assert it.**
- A SCOPED `cargo nextest run --release -E 'test(category) + test(runtime_meta) + test(intrinsic)'`.

## ⛔ You do NOT run the floor

Foreground only: `cargo build --release`, `./target/release/wat --check <file>`, a scoped
`nextest -E`. **Not** `scripts/floor.sh`, not an unscoped `nextest`. The orchestrator measures
centrally, once, on a quiescent tree.

Cap long runs: `systemd-run --user --scope -q -p MemoryMax=6G -p MemorySwapMax=0 timeout 900 …`
Read exit codes directly, never through a pipe.

## STOP triggers — ship nothing further, report, stop

**STOP-1** — the build fails on the `defenum` and the fix is not obvious from the error. This file
feeds a proc macro; a subtle syntax error surfaces as a confusing derive failure, not a parse error.
Report the payload verbatim.

**STOP-2** — a `MissingCategory` → `compile_error!` fires anywhere. That means an existing registry
row lost its category, which this stone must not cause.

**STOP-3** — anything outside `wat/runtime-meta.wat` needs to change.

**STOP-4** — a scoped `nextest` goes red.

## On a RED

No such thing as a known flake. (a) Do **NOT** re-run. (b) Copy the whole stdout+stderr block
**verbatim** — never a `| head`/`| tail` window. (c) Name the exact assertion. (d) Surface it.

## Your report

1. The subject line, before and after.
2. Each of the five new `;;` blocks, quoted as shipped.
3. `:Io`'s prose before and after.
4. Confirmation that no variant was inserted mid-list and no existing variant's prose changed except
   `:Io`'s added clause.
5. Everything you ran, with results. State plainly that you did not run the floor.
6. Honest deltas — what surprised you, what this brief got wrong. Wall-clock against a **25–40
   minute** prediction.

Slow is smooth, smooth is fast.
