# BRIEF — the prose stops teaching a dead syntax

**Shared brief. Six riders, disjoint file sets, no coordination needed between you.** Your slice is
named in your own prompt; touch nothing outside it.

`<K,V>` cannot be written, minted, rendered, parsed, or documented in an `@arg`/`@ret`. It is still
**taught** — 1,373 lines of prose across the guides, the stdlib, the corpus and the Rust comments. A
reader learning wat from these would learn the retired spelling exclusively, write it, and be refused by
the reader with no idea what to write instead.

Read `DESIGN-STONE-rip-the-heresy-from-the-prose.md` for the campaign context.

## ⛔ TEXT EDITS ONLY — do NOT run cargo

Do not run `cargo build`, `cargo test`, `cargo nextest`, or `scripts/floor.sh`. Six riders share one
`target/` and one build lock; a per-rider build is N-way contention and a gate none of you controls.
**The orchestrator builds, floors and clippies centrally, once, after all six are done.**

You may run `./target/release/wat --check <file>` on a `.wat` you edited — it uses the existing binary
and takes ~0.2s. That is the only verification you need: a comment edit must not change how the file
parses.

## The surviving spellings — what prose must say instead

```clojure
;; a TYPE REFERENCE
:wat::core::Vector<wat::core::i64>      →  (:wat::core::Vector :- [:wat::core::i64])
:wat::kernel::Peer<S,R>                 →  (:wat::kernel::Peer :- [S R])
:wat::core::Option<wat::core::Vector<T>> →  (:wat::core::Option :- [(:wat::core::Vector :- [T])])

;; a DECLARATION's own name (defn / defrecord / defenum / defsurface / typealias …)
defsurface :Holds<T>                    →  defsurface :Holds :- [T]
defn :my::pick<T>                       →  defn :my::pick :- [T]

;; a FUNCTION type
:wat::core::Fn(A)->B                    →  [A :-> B]
```

⚠ **Reference vs declaration is a judgement.** A type being *used* takes the parenthesised reference
form; a name being *declared* takes the bare binder as siblings. If a line is ambiguous, prefer the
reference form and say so in your report.

## The classification — FM 14's A/B/C/D. **C is what a careless pass destroys.**

```
A  live prose naming a wat type in the dead spelling   →  REWRITE
B  stale text describing retired machinery             →  UPDATE, or delete if the machinery is gone
C  prose RECORDING the retirement                      →  KEEP, VERBATIM
D  RUST generics in a Rust comment                     →  KEEP, UNTOUCHED  (rs slice only)
```

**Class C example — do not touch lines like these:**

> *"`Head<K,V>` was the only construct in the language that gave a comma meaning"*
> *"arc 109 ③ — angle brackets are ILLEGAL for types; `<T>` used to be sniffed and split here"*

They are the record of *why* the language has its shape. Erasing them leaves the law looking arbitrary.
~45 such lines were measured across the `.wat` corpus. **When in doubt, KEEP and report it.**

**Class D — the `.rs` slice only.** `Vec<T>`, `Option<String>`, `Arc<Function>`, `HashMap<K,V>`,
`Receiver<T>` in a Rust comment are **Rust's own generics** and are none of this stone's business. The
discriminator is whether the comment is describing a **wat** type or a **Rust** type — a `:wat::`
prefix or a wat `::` path is the strong signal, but read the sentence, do not pattern-match.

## ★ The two `.md` slices have a SECOND job

Stopping the guides from saying `<K,V>` is not enough — **not one of the five user-facing documents
mentions `:- [` a single time.** A reader who never sees the wrong spelling still cannot write the
right one.

Where the old text *taught the rules* of the angle form, it must teach the rules of `:-`. The
cheatsheet's nesting section is the clearest case — it currently explains why `:Vec<:String>` is
illegal, a rule about a syntax that no longer lexes. Replace the explanation, do not merely delete it.

⚠ **Do not invent the rules.** The canonical statement is in
`docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md` under *"`:-` IS THE PARAMETERIZATION OPERATOR"*,
and every claim you write must be one you can verify with `./target/release/wat --check`. **A guide
that teaches a spelling the reader refuses is the exact defect this stone exists to end** — do not
create a second one pointing the other way.

## STOP triggers

- **STOP-1 — a line is class C and class A at once**: it records history AND is used as live guidance.
  Report it; splitting the sentence is a judgement the orchestrator should see.
- **STOP-2 — a `.wat` file stops parsing** after your edit (`--check` fails). You changed more than a
  comment. Revert that file and report.
- **STOP-3 — you cannot verify a rule you are about to write** into a guide. Report it rather than
  writing an unverified claim into the teaching material.

## Boundaries

- **Your slice only.** The other five riders are working concurrently; touching their files loses work.
- ⛔ **`docs/arc/**` is OUT OF SCOPE for every rider.** The arc record is history by construction.
- ⛔ **`*.wat.bad` is OUT OF SCOPE.** Those are negative fixtures; their illegal text is the test.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- Do NOT change code. Comments and prose only. If a fix requires touching code, that is a finding —
  report it.

## Your report

The count you changed, and the count you deliberately KEPT as class C or D — the second number is the
one that shows you classified rather than swept. Every STOP that fired. Any line you were unsure about,
quoted. For the `.md` riders: which rules you rewrote, and how you verified each claim you wrote.
