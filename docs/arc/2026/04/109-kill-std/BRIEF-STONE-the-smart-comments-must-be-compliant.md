# BRIEF — the smart comments must be compliant

`@arg` / `@ret` annotations are the user-facing API reference for wat's intrinsics, and they are
**validated** — `crates/wat-doc` checks the directive grammar, tag order, and rejects separators in the
type position. Then it adjudicates the TYPE like this, at five sites:

```rust
// Type token must start with `:` (all wat types are keywords).
if !ty_token.starts_with(':') { … }
```

**The first character is the entire type check.** You will replace it with the language's own reader.

Read `DESIGN-STONE-rip-the-heresy-from-the-prose.md` first. The tree is CLEAN and the floor is green at
4911/4911. Copy the report shape of `SCORE-STONE-the-last-comma-lives-in-a-symbol.md`.

## STEP 1 — `wat-doc` asks the reader

The dependency is available and adds nothing to the graph: `wat-doc` → `wat-source-derive` →
`wat-reader`, and `wat-source-derive`'s manifest states the reason it exists — *"Depends on nothing of
wat's but wat-reader, so both wat-doc and the main crate can use it without a cycle."* `wat-reader`
depends only on `wat-edn`. Add the direct dependency.

Then the five shape tests (`crates/wat-doc/src/lib.rs:369, 407, 525, 671, 705`) become **one** call —
one helper, five call sites, not five copies of a lex.

⚠ **Do NOT write an angle-specific check.** The point is that the READER decides what a type may be
spelled. Measured, running the real lexer over doc type tokens:

```
:wat::kernel::Listener<S,R>                  REFUSED
:wat::core::Listener<S>                      REFUSED     ← single-param too
:wat::core::Bytes                            LEXES
:wat::core::<                                LEXES       ← the operator survives
(:wat::core::Vector :- [:wat::core::i64])    LEXES       ← the surviving spelling
```

A reader-based check also catches the retired `fn(…)->…` vocabulary for free — see the offender list.

⚠ **Keep the `starts_with(':')` requirement as well.** A bare `Bytes` lexes fine as a symbol; the
annotation grammar still demands a keyword. The reader answers *"is this expressible"*, not *"is this a
type keyword"* — you need both, and the existing error message for the colon case stays.

## STEP 2 — the build screams; fix what it names

**Measured: 67 distinct type tokens across `@arg`/`@ret`, of which 28 carry an angle head.** You do not
need a census — once step 1 lands, the validator names every one. Sample:

```
:wat::core::Option<wat::core::Bytes>            →  (:wat::core::Option :- [:wat::core::Bytes])
:wat::core::Option<wat::kernel::Process<I,O>>   →  nested — mind the inner form
:wat::core::Fn(wat::kernel::Peer<S,R>)->wat::core::nil
        ↑ the retired `fn(…)->…` spelling AND an angle. The surviving function-type spelling is the
          bracket form `[arg… :-> ret]` — confirm against a live use before rewriting it.
```

⚠ For each, the replacement must be **the spelling the language actually accepts** — verify by running
the token through `--check` in a real annotation position, not by pattern-matching what looks right.
The nested and function-type cases are where a plausible-looking rewrite will be wrong.

## STEP 3 — a negative control, kept

A doc comment naming an inexpressible type must fail the build, and that must stay true. Ship a test
proving the validator refuses one, **with its positive twin** proving `:wat::core::Bytes`,
`(:wat::core::Vector :- [:wat::core::i64])` and `:wat::core::<` all pass. Rows 1-2 below are that pair.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★★ | `@ret :wat::core::Option<wat::core::i64>` | ⛔ REFUSED, naming the file and line |
| 2★★★ | `@ret :wat::core::Bytes` · `(:wat::core::Vector :- [:i64])` · `:wat::core::<` | ✅ all accepted |
| 3★★ | the 28 offenders | fixed, and each verified as a spelling `--check` accepts |
| 4★★ | `@ret Bytes` (no colon) | ⛔ still refused with the existing colon message |
| 5★ | build + doc tests | green |

**Row 2 decides it.** Row 1 goes green for a validator that refuses every type — which would fail the
whole intrinsic corpus. Only the legal spellings still passing proves it adjudicates rather than
rejects, and row 4 proves you kept the colon rule rather than replacing it.

## STOP triggers

- **STOP-1 — a token the reader refuses is NOT a retired spelling** but something the docs legitimately
  need to say. Report it; that is a finding about the reader or the annotation grammar, not a doc typo.
- **STOP-2 — the replacement spelling for the `Fn(…)->…` token is not obvious** from a live use in the
  corpus. Report it rather than guessing; a wrong type in the API reference is the defect this stone
  exists to end.
- **STOP-3 — adding the `wat-reader` dependency creates a cycle.** The manifests say it cannot; if
  cargo disagrees, report what it says.

## Boundaries

- `crates/wat-doc/`, its Cargo.toml, and the `@arg`/`@ret` annotations the build names.
- **Do NOT sweep `;;` comments in `.wat` files.** 798 lines, its own sibling stone — nothing validates
  them, so there is no door to teach and it is a genuine classified sweep.
- **Do NOT touch ordinary Rust doc comments.** Only `@arg`/`@ret` TYPE TOKENS are adjudicated; `Vec<T>`
  and `Option<String>` in prose are Rust's own generics and are none of this stone's business.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- The orchestrator runs the full floor and clippy centrally.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.
`cargo wat` uses the STALE installed binary; always `./target/release/wat`.

## Your report

Rows 1, 2 and 4 verbatim from one run — the refusal, the acceptances, and the colon rule still firing
— because that triple is the whole proof. Then the 28, with the spelling each became and how you
verified it. Any STOP that fired, with the arm captured verbatim BEFORE you diagnosed it. What
surprised you.
