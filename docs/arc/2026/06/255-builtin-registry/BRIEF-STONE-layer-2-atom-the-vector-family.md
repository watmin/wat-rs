# BRIEF — STONE layer-2: split `atom.rs`'s six vector/bytes verbs at the conversion boundary

Read `DESIGN-STONE-layer-2-atom-the-vector-family.md` first — especially **THE RULE**, which is
this brief's whole method.

## The work, one paragraph

Six `#[wat_intrinsic]` verbs in `src/intrinsic/holon/atom.rs` mix door work with implementation.
**For each, find the line where wat values become Rust domain values, and move everything past it
into `src/holon/`.** The codec logic (encode/decode of the vector wire format) goes to a new
`src/holon/codec.rs`. The attribute, doc block and signature stay where they are; the delegate that
remains does the conversion, calls the impl, and adapts the result back.

## THE RULE — apply it per verb, not per wave

> **The door converts wat values to Rust domain values and adapts errors back.
> Everything past the conversion is implementation.**

⛔ **"No change needed" is a legal, expected answer.** `holon_vector_bind` is 12 lines and is
probably already pure door. Do not manufacture a split to make the count six. Report which verbs
needed nothing and why.

## Read in order

```
src/intrinsic/holon/atom.rs      the six (line numbers below)
  eval_holon_bytes_vector   93 ln   :wat::holon::bytes-vector    ← START HERE, the exemplar.
                                    Its seam is at the `// Header.` comment: everything above
                                    is conversion, everything below is codec.
  holon_vector_bytes        51 ln   :wat::holon::vector-bytes     its encode twin
  eval_holon_vector_bundle  50 ln   :wat::holon::vector-bundle
  eval_holon_vector_permute 29 ln   :wat::holon::vector-permute
  holon_vector_blend        20 ln   :wat::holon::vector-blend
  holon_vector_bind         12 ln   :wat::holon::vector-bind      likely already pure door

src/holon/outcome.rs:206-255     the five `vector_decode_outcome_*` constructors — ALREADY in
                                 the impl layer. The extracted codec returns through these.
src/intrinsic/i64.rs:~171        the delegate shape (a body that is one call)
src/collection/eval.rs           layer-1's result: what a clean impl-layer landing looks like
```

## The test that the seam was cut correctly

**`src/holon/codec.rs`'s signatures must mention no `WatAST`, no `Value`, no `RuntimeError`,
no `Span`, no `Environment`, no `SymbolTable`.** Domain types only — `Vec<u8>`, `holon::Vector`,
and a plain Rust `Result`/enum for the failure cases.

If a function cannot be extracted without dragging a wat type along, that is the finding: report it
with the specific type and the reason. It means the seam is somewhere other than where the rule
predicts, and knowing that is worth more than the extraction.

★ The five `vector_decode_outcome_*` constructors DO return `Value`. They are the ADAPTATION of
the codec's result and belong to the door side — so the codec should return a domain-typed outcome
and the delegate should map it through those constructors. If that inverts awkwardly, say so.

## Blast radius

`src/intrinsic/holon/atom.rs`, the new `src/holon/codec.rs`, `src/holon/mod.rs` (to declare it),
and `src/holon/outcome.rs` only if a signature genuinely must change. Nothing else.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **An extraction requires a wat type in the impl's signature.** STOP and report the type — that
   is the stone's most valuable possible finding.
2. **You are about to change the wire format**, or any observable behaviour. Bytes in, bytes out,
   identical. STOP.
3. **You are about to move a `#[wat_intrinsic]` attribute or doc block out of `src/intrinsic/`.**
   The verb would vanish from the completeness gate's population. STOP.
4. **You are about to touch a verb outside the six** — `from-holon` above all. STOP.
5. **You are about to add a gate, lint, or ledger** for the door/impl rule. A later stone. STOP.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: for each of the six, name the exact line where wat values become domain
      values — the seam — and how many lines sit past it. Report BEFORE editing. A verb with zero
      lines past the seam needs no change; say so.
 1. ★ THE CODEC EXTRACTED to src/holon/codec.rs, and its signatures mention NO wat type. Quote
      every public signature in the new file.
 2. ★ EACH OF THE SIX: changed or unchanged, with the reason. An unchanged verb must be justified
      by the rule, not by its size.
 3. ★ THE DOC BLOCKS UNTOUCHED — `git diff` on atom.rs shows no removed `///` line and no removed
      `@` directive.
 4. ★ BYTE-IDENTICAL ROUND TRIP: encode a vector, decode it back, before and after the change.
      Show the same bytes. Also exercise at least two FAILURE paths (truncated header, length
      mismatch) and show identical outcomes.
 5. ★ BREAK THE DOOR: sabotage one delegate so it does not reach its impl; show a test go red;
      quote it; restore. A split that compiles proves nothing about which half runs.
 6. ★ THE GATE STILL SEES ALL SIX:
      cargo nextest run --release -E 'test(every_dispatched_verb_is_classified_or_disposed)'
      Report its UNREVIEWED line — must read 217. And the registry must read 429.
 7. ★ LINE ACCOUNTING: atom.rs before/after, codec.rs, and the registration layer's net change.
 8. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 9. cargo nextest run --release -E 'test(holon) + test(vsa) + test(intrinsic)'
```

★ **Row 0 is the one that makes this stone worth doing.** Naming the seam per verb, in writing, is
the data the eventual gate predicate gets built from. Six honest seam locations are worth more than
six extractions.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

Your row-0 seam table — the line, and the count past it, for all six. The new file's signatures.
Which verbs changed and which did not, each with its reason. The round-trip evidence including the
two failure paths. Row 5's red, verbatim, and confirmation you restored it. The gate's UNREVIEWED
line and the registry count. Line accounting. Then the honest deltas — above all **any place the
rule was ambiguous or wrong**, because that is what the next stone inherits.
