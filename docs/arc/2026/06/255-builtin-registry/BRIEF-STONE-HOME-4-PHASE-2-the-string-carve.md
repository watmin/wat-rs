# BRIEF — HOME #4, PHASE 2: the string carve

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-HOME-4-the-string-carve.md` — read whole.

**Phase 1 shipped** (`266065d0f`): the doctest runner collects instead of raising, and names each
failure by `fqdn`. That is what makes this phase's examples verifiable as they land — the reason
the carve waited.

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you. Run every
command in the FOREGROUND and block on it.

**You may not spawn sub-agents.** Anchor: `/home/john/work/holon/wat-rs`. `pwd` first. You do not
commit, push, stash, revert, or checkout.

`cargo build --release` is yours. A NARROW `cargo nextest -E 'test(...)'` is yours for the rows
below. `scripts/floor.sh` and clippy are NOT — the orchestrator takes those centrally.

## The work

Carve the **19 Rust-implemented `:wat::string::*` verbs** out of `src/string_ops.rs` into a new
`src/intrinsic/string.rs`, each with a `#[wat_intrinsic]` preamble. This is the last link of
`CHAIN-rendering-before-the-string-home.md`, and it lands on FINAL names because stone E
(`23efc6056`) already moved them:

```
concat  contains?  declare-acronyms  ends-with?  interpolate  join  kebab->pascal-in
length  pascal->kebab  pascal->kebab-in  split  starts-with?  subs  to-bool  to-f64
to-i64  to-lowercase  to-uppercase  trim
```

⚠ **Three `:wat::string::*` verbs are NOT yours** — `capitalize`, `kebab->pascal`,
`strip-leading-colon` are wat defns in `wat/string.wat`, not Rust intrinsics. They stay. The carve
is the Rust 19.

## ⛔ THE HAZARD: A HALF-DONE CARVE STILL WORKS

`src/runtime.rs:5394` consults the registry **before** the match, and says why:

> *"Registered wins, always: a literal arm below this point can no longer shadow a registration by
> sitting higher in the match."*

So if you register a verb and leave its old `match` arm in `runtime.rs`, **everything passes**. The
registration wins; the arm is silently dead code. Nothing goes red. `Bytes::to-hex` — arc 255's
first home — has NO arm left; that is what a finished carve looks like.

**Row 2 exists solely for this**, because it is the only row that can catch it.

## The shape

`src/intrinsic/bytes.rs:34-44` is the worked reference. Each handler carries a `///` preamble the
`#[wat_intrinsic]` macro parses — it sniffs arity, emits the arity-checking shim, and
`inventory::submit!`s the (fqdn → shim) pair. No explicit `register()` call:

```rust
/// Markdown prose, GFM — flows straight to the wiki page body.
///
/// @added 1.0.0   @Purity Pure   @Determinism Deterministic   @Category Transform
/// @arg     s :wat::core::String  the string to trim
/// @ret     :wat::core::String    the string with leading and trailing whitespace removed
/// @example (:wat::string::trim "  x  ") #=> "x"
/// @see     :wat::string::to-lowercase
#[wat_intrinsic(":wat::string::trim")]
```

The module must be `mod`-declared in `src/intrinsic/mod.rs` or its submissions never link.

⚠ **Every `@example` you write WILL be executed** — that is phase 1's gift and row 4's bar. Write
ones you have run. An example is not a decoration; it is a test with a doc's syntax.

## The rooms

1. **`src/intrinsic/bytes.rs`** — the shape, end to end. Copy it.
2. **`src/intrinsic/mod.rs:1-30`** — the registry's own doc: what each field is for, and the
   accretion discipline (satisfy a forcing-signal by USE, never silence it).
3. **`src/string_ops.rs`** — 1254 lines, the source. Note what you are NOT taking (below).
4. **`src/runtime.rs:5390-5400`** — the lookup-before-match hoist and its comment.
5. **`wat/string.wat`** — the 3 wat defns that stay.

## ⚠ `string_ops.rs` IS FOUR DOMAINS — take only one

```
:wat::string::*        19   ← yours
:wat::core::Uuid/*     11   ← NOT string. Leave it.
:wat::core::char/*      2   ← NOT string. Leave it.
:wat::core::regex::*    1   ← NOT string. Leave it.
```

A "string carve" that quietly relocates UUID generation is a carve that lied about its subject.
Those three get their own homes, drawn by someone else. If the file ends up oddly named for what
remains, say so in your report — do NOT rename it (STOP-4).

## The acceptance rows YOU run

- **Row 1 — all 19 are registered.** `#[wat_intrinsic]` count in `src/intrinsic/string.rs` is
  exactly 19, and the registry total goes 146 → 165. Report both numbers.
- **★ Row 2 — NO leftover match arms.** `grep -c '":wat::string::' src/runtime.rs` is **0**. This is
  the row nothing else can catch, because a leftover arm is silently dead rather than broken.
- **Row 3 — `metadata-of` answers for each of the 19.** Run it per verb; report the full list of 19
  answers, not a sample.
- **★ Row 4 — the doctest count is STILL 5.** Not 5+n. Every example you wrote passes.
  `cargo nextest run --release --run-ignored all -E 'test(verify_examples_reports_no_failures)'` and
  report the `left:` number. **If it is above 5, your examples are the difference** — find which,
  using the runner's per-`fqdn` reasons (phase 1 built exactly that).
- **Row 5 — the other three families are untouched.** `Uuid` 11, `char` 2, `regex` 1 still in
  `string_ops.rs`; `git diff` shows no move.
- **Row 6 — behaviour is unchanged.** The existing string tests still pass:
  `cargo nextest run --release -E 'test(string)'` — report the Summary.

Report each row's command and output **verbatim** — never a summary, never a `| head`/`| tail`
window. A row you could not run is reported as not-run, never as passed.

## Blast radius

- `src/intrinsic/string.rs` — created
- `src/intrinsic/mod.rs` — one `mod` line
- `src/string_ops.rs` — the 19 handlers leave; Uuid/char/regex stay
- `src/runtime.rs` — the 19 match arms are DELETED

Nothing in `wat/`. No new verbs. No renames.

## STOP triggers — each ships NOTHING and surfaces the gap

1. **A verb's signature does not fit the `#[wat_intrinsic]` fixed-arg form** — variadic `concat`,
   or an arity the macro cannot sniff. STOP and report which verb and what the macro said. Do NOT
   reshape the verb's signature to fit the macro; that changes the language.
2. **Row 4 rises above 5** and you cannot make your own examples pass. STOP and report the failing
   `fqdn` + reason verbatim. An example that will not pass is either a wrong example or a real bug
   — the same STOP that fired in phase 1, and telling them apart is not a doc edit.
3. **Deleting a match arm changes behaviour** — a test goes red that the registration should have
   covered. STOP; that means the arm and the handler were not equivalent, which is a finding.
4. **You want to rename or split `string_ops.rs`.** STOP — say it in the report instead. What the
   file should be called once it holds only Uuid/char/regex is a decision, not a cleanup.

A STOP means: leave the tree as it is, write the report, end your turn.

## What you own that nobody can reconstruct

Row 3's 19 `metadata-of` answers, row 4's number, and anything that surprised you — a verb whose
signature fought the macro, an `@example` that was harder to make true than expected, or a
behaviour difference between the arm and the handler.
