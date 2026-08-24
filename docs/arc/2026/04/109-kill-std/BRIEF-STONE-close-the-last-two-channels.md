# BRIEF — close the last two channels a dead type reaches a user through

The rule this campaign enforces: **a type must reach a user in a spelling the reader accepts.** Four
channels violate it. Two are closed. **These two are not, and both were measured open minutes ago.**

The tree is CLEAN and the floor is green at 4916/4916. Copy the report shape of
`SCORE-STONE-the-last-comma-lives-in-a-symbol.md`.

## The state, measured

```
1. RENDERED parametric types   format!("{}<{}>")            CLOSED  (64a8fa5a0)
2. DOC @arg/@ret               starts_with(':') validation  CLOSED  (f82dc6de1)
3. FUNCTION types              Fn(A,B)->C printed           OPEN    ← yours
4. HARD-CODED PROSE            122 string literals          OPEN    ← yours
```

## CHANNEL 3 — the function type

```
:expected ":wat::core::Fn(wat::core::i64)->wat::core::i64"     ← what prints
           [:wat::core::i64 :-> :wat::core::i64]                ← what you must write
```

For **two or more arguments the printed form cannot be read at all** — `Fn(A,B)->C` carries a comma
inside a keyword body, refused since the comma strike. The substrate prints a function type its own
reader would reject.

```
src/check.rs:16274   format!(":wat::core::Fn({})->{}", in_parts.join(","), …)
src/check.rs:16306   format!("wat::core::Fn({})->{}",  in_parts.join(","), …)   ← the same shape twice
```

**One shared helper, not two edits** — two copies of one rendering is the defect this arc has removed
eleven times.

⚠ **And there is a READER for the retired spelling**: `src/types.rs:5304`,
`s.strip_prefix("wat::core::Fn(")`. Determine whether it is still reachable — R6 measured that
`Fn(single-arg)->ret` **lexes** (the keyword lexer treats balanced parens as body text) while the
multi-arg form does not. So this may be half-live. Report which, and do not delete it on the strength
of the render change alone.

## CHANNEL 4 — 122 hard-coded strings

User-visible, at the moment a user is most looking for guidance:

```clojure
(:wat::core::nth "not a vector" 0)
  → :expected "Vector<T>, List<T>, PersistentVector<T>, or WatAST"
```

```
src/runtime.rs                35     src/types.rs                    2
src/collection/eval.rs        29     src/intrinsic/reflect.rs        2
src/check.rs                  22     crates/wat-reader/src/lexer.rs  2
src/collection/infer.rs       16     + 6 files with 1 each
crates/wat-edn/interop-tests   7
```

⚠ **NOT by regex.** `Vec<T>`, `Arc<Function>`, `Cow<'_, [WatAST]>` live in the same files and the same
functions. The discrimination is whether the string names a **wat** type shown to a **wat user** — read
the message, do not pattern-match. A rewritten Rust generic is worse than an unrewritten wat type.

⚠ **Some are class C.** `crates/wat-reader/src/lexer.rs`'s two are likely inside the refusal message
that *teaches* the retired spelling by naming it — quoting the dead form in order to reject it is
correct and must stay. Check every one for that shape before rewriting.

## The gate — this stone is not done without one

Channels 1 and 2 each got one: a rune, and a validator that fails the build. **A sweep with no gate is
a moment.** Draw one over string literals in diagnostic positions — `expected:`, `reason:`, `message:`
— with earned `rune:lint(...)` exemptions carrying a reason for each class-C quote.

**Positive-control it**: plant a violation, confirm it fails and names the site, remove the plant.
`tests/lint/one_param_spec.rs` and `no_angle_suffix_strip.rs` are the shape and both are already
positive-controlled.

## ★ And the census that ends this — scoped from the RULE

Four channels were found one at a time, each after I declared the previous one closed, because every
census asked *"where does the code parse or render a type?"* A string literal naming a type is neither.

**Your last task is to enumerate the CHANNELS, not the shapes:** through what paths can a type name
reach a user? Rendering, diagnostics, doc directives, reflection output, `Display` impls, panic
messages, EDN tags, the REPL. For each, state whether it is closed and how you know. **A channel you
cannot rule out is a finding**, not an omission.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★★ | a function-type mismatch | prints `[:wat::core::i64 :-> :wat::core::i64]`, **paste-able back into source** |
| 2★★★ | a 2-arg function type | prints readably — the old form could not be read at all |
| 3★★ | `(:wat::core::nth "x" 0)` | names only spellings the reader accepts |
| 4★★★ | Rust generics in messages | UNTOUCHED — `Vec<T>`, `Arc<…>` still say what they say |
| 5★★ | the lexer's own refusal message | still quotes the dead form (class C — it teaches by naming) |
| 6★ | the gate | drawn, positive-controlled |
| 7★★ | the channel enumeration | every channel named, with its status and evidence |

**Rows 1 and 4 decide it.** Row 1 must be verified by **pasting the printed string into a real program
and running it** — that is how channel 1 was proven, and "it looks right" is not the same claim. Row 4
is what separates a discriminating pass from a regex.

## STOP triggers

- **STOP-1 — `types.rs:5304`'s `Fn(` reader is reachable and load-bearing.** Report what reaches it.
- **STOP-2 — a string is genuinely ambiguous** between a wat type and a Rust type. Report it quoted;
  do not guess.
- **STOP-3 — the gate cannot pass on a green tree** without exemptions so broad it means nothing.
  Report the shape you tried; ship the sweep without it rather than a rune that lies.

## Boundaries

- `src/check.rs`, `src/runtime.rs`, `src/collection/{eval,infer}.rs`, `src/types.rs`, the tail files,
  and one gate.
- **Do NOT touch the `.wat` `;;` comments or the guides** — a six-rider sweep just landed there; R6's
  `.rs` comment tail is separately outstanding and is NOT yours.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY.
- **You may not spawn sub-agents.** If the slice is too large, report that and stop.
- The orchestrator runs the full floor and clippy centrally.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

Row 1 verbatim AND the program you pasted it into. Row 4's count of Rust generics deliberately left.
The channel enumeration in full. Whether `types.rs:5304`'s reader survives and why. Any STOP, with the
arm captured verbatim BEFORE you diagnosed it. What surprised you.
