# DESIGN + BRIEF — ③b-ii ⑦: the fixtures under `tests/`

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C /home/john/work/holon/wat-rs`.

## ⚠ STATE: the corpus LOADS; the floor is the arbiter

HEAD is the held landing (unpushed). The dot flip is complete on the substrate side — the stdlib
freezes, `wat --check` reports **zero** errors, and six classes of variant-name site are closed.

```
floor   5334 tests run (the SAME roster as pre-flip)   4539 passed   795 FAILED
```

The 795 are this class and nothing else. **A red start is the progress meter.** You are driving
795 → 0.

## The class, and how it happened

My codemod path list was `find wat wat-tests wat-scripts -name '*.wat'` — **846 files**. There are
**1400 more `.wat`/`.wat.bad` files under `tests/`**, of which **378 carry a variant `::`**. I never
asked whether `.wat` files lived anywhere else. That is the third time the `tests/` boundary has
hidden a class in this campaign, and the first time my own hand drew it.

## ★ Why this class has an instrument the stdlib never had

The census splits exactly as phase ① did — 449 distinct variant-shaped tokens in `tests/`:

```
112   already in the 382-pair census      (stdlib / corpus variants)
289   fixture-local WITH a defenum        (text-parseable from source)
 48   macro-generated                     (a defservice's Response/Op/Reply — defenum'd nowhere)
```

Neither instrument is complete alone. **But these are TEST FIXTURES.** A missed stdlib variant
breaks the world silently; a missed fixture fails **its own test, by name**. So the floor is the
completeness check that was structurally unavailable when this same problem appeared in the stdlib.

That inverts the discipline for this class only: you do not need a census provably complete before
you start. You need a good one, then the floor enumerates the remainder, and every survivor names
its own file.

⛔ **This licence does NOT extend to guessing.** Every rename must still be a variant by the
substrate's own answer, never by its shape:

```
wat::cache::Cache::GetRequest      a live defrecord   (wat/cache.wat:171)      NOT a variant
wat::cache::HolographicLru::put    a live defn        (wat/cache.wat:294)      NOT a variant
wat_edn::OwnedValue::Tagged        a Rust type path                            NOT a wat name
wat::program::PeerKind.thread      VARIANT of wat::program::PeerKind
```

A regex that renames one of the first three does not fail; it means something else.

⛔⛔ **NEVER BRANCH ON CHARACTER CASE.** `(wat.core/defenum user/some-enum … first [] second [])`
legally produces `user/some-enum.first`. `Enum.Variant` over `enum.variant` is a BIAS, and a bias is
not a rule — see `docs/arc/2026/04/109-kill-std/NOTE-character-case-carries-no-meaning.md`. Any
candidate predicate you write must be case-free; over-generate freely and let the ASK decide, which
is what makes over-generation safe. Two live variants, `PeerKind.thread` and `PeerKind.process`,
have lowercase leaves, and a capitalised-leaf grep walks straight past them.

## The three phases

**A — extend the pair list.** Text-parse every `defenum` under `tests/` and compose each declared
variant's old (`::`) and new (`.`) spelling through `wat_reader::identifier` semantics — the parent
is the enum's declared name, the leaf is the declared variant. Merge with
`docs/arc/2026/06/255-builtin-registry/dot-flip-phase1-pairs.txt`. Record the extension as its own
artifact beside that file; the instrument must outlive the number.

**B — run the PROVEN codemod on the paths I omitted.**
`wat-scripts/fixes/variant-separator-to-dot.wat` is unchanged and already verified: on the 846-file
run every one of 4221 changed lines reconstructed exactly from a census pair, 0 unexplained. Feed it
`tests/**/*.wat` and `tests/**/*.wat.bad` with the merged pair list. **Dry-run on a `/tmp` copy and
`diff` first**, exactly as that codemod's own stone did.

**C — read the floor's reds, and ASK for each residue.** For every still-failing fixture, the error
names the offending token. Construct the **dot form you propose to write** and ask
`:wat::runtime::compose-variant`'s sibling `:wat::runtime::variant-parent-of` about THAT — the
inverted ask, measured working on this tree:

```
wat::core::Option.Some  -> VARIANT of wat::core::Option
wat::core::Option::Some -> no            the old spelling is dead
wat::core::Record.def   -> no            a method is still not a variant
```

Asking about the old `::` name answers `None` for everything now, which is why the question has to
be turned around. Iterate B→C until the floor stops naming fixtures.

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** The dry-run diff changes anything other than a `::` → `.` at a variant boundary. Report
the hunk verbatim.

**STOP-2.** A floor failure is NOT a variant-spelling failure — a fixture whose red the flip does
not explain. That is an eighth class, and the census is the orchestrator's to close. Report it with
the verbatim failing block; do not chase it.

**STOP-3.** A `.wat.bad` fixture STOPS failing, or starts failing for a different reason. Those
files exist to be refused; a change in *why* they are refused is a finding about the checker, not a
migration detail.

**STOP-4.** You cannot confirm a token is a variant — the ask says `None` for both spellings and no
`defenum` declares it. Report it. Do not rename on shape.

## Acceptance

```
floor 5334/5334 via scripts/floor.sh   ·   clippy --release --all-targets -D warnings = 0
the one-variant-separator wall stays GREEN — the flip must reintroduce no hand-rolled split
the pair-list extension committed beside phase ①'s, with the command that regenerates it
```

## What to run

Your dry-run and diff; `cargo build --release` if needed. **The orchestrator runs `scripts/floor.sh`
and clippy centrally** — but you MAY read `.floor/latest/raw.log` from the run already on disk to
enumerate the 795. Foreground everything. Do not commit. Do not contact any peer.
