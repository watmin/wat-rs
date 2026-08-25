# BRIEF — Stone D: `\c` joins the literal lane (`WatAST::CharLit`)

DESIGN: `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-D-char-joins-the-literal-lane.md` — read it whole first.

PRIOR ART, and it is close: **`WatAST::BigIntLit` (arc 300 stone C1)** and **`WatAST::RationalLit`
(stone B)** did exactly this, twice, in this arc. Read `DESIGN-STONE-rational-C1-bigint.md` and then
`git log --oneline --all -- crates/wat-reader/src/ast.rs` to find C1's commit; its diff is the shape
of yours. `WatAST::NilLit` (arc 244) is the third instance.

---

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first and stay there.

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it. Your turn ends when the numbers are in your hands.

**You may not spawn sub-agents.** Do not commit, push, stash, revert, or `git checkout`. There is a
`git stash@{0}` in this tree that must never be touched.

There is one untracked file, `wat-scripts/fixes/rename-four-families-to-their-homes.wat`, belonging to
a different, blocked stone. **Leave it exactly as it is** — do not edit it, run it, or delete it.

### On cargo — this stone is the exception, and here is its boundary

Adding an enum variant is a compiler-guided cascade; you cannot do this work blind. So you MAY run:

```
cargo build --release 2>&1 | tail -40          # your worklist generator
cargo nextest run --release -E 'test(<one specific test>)'   # a single named test, when a row needs it
```

You may NOT run the full floor (`scripts/floor.sh`, a bare `cargo nextest run --release`) or `cargo
clippy`. The orchestrator runs those centrally when you report. Nothing else is running against this
tree, so the build lock is yours.

---

## The work, in one paragraph

`WatAST` has a `*Lit` variant for every scalar literal except `char`. `\a` desugars at parse time into
`(:wat::core::char/of "a")` — a three-node call where a one-node literal belongs. Add
`WatAST::CharLit(char, Span)`, point the parser at it, and let the compiler name every site that must
learn the new variant.

```rust
Token::Char(c) => Ok(Some(WatAST::CharLit(*c, span))),     // was: WatAST::List([Keyword, StringLit])
```

The lexer already resolves `\newline` / `\space` / `\tab` / `A` to a `char`, and the runtime already
holds `Value::wat__core__Char(char)`. **Nothing new is being represented.** You are removing a
desugar, not adding a capability.

---

## THE METHOD — the failures ARE the brief

Read `docs/SUBSTRATE-AS-TEACHER.md` before you start. This is that pattern:

1. Add the variant to `crates/wat-reader/src/ast.rs` and repoint the parser.
2. `cargo build --release` — it will fail, possibly with dozens of non-exhaustive-match errors.
   **That count is the progress meter, not a crisis.** Every error names a site that needs a `CharLit`
   arm.
3. Fix a category, rebuild, watch the count fall. Repeat to zero.
4. Then run the specific tests the acceptance rows name.

The three prior literal-lane additions in this repo landed 31, 33 and 42 arms across 16–18 files.
Expect the same order of magnitude. Each arm is mechanical: span accessor, hash, a constructor, a
type-string, an eval arm, a check arm, an EDN arm.

**Do not enumerate the sites up front and do not stop to ask whether the count is alarming.** Let the
compiler waterfall it.

### The one arm that is NOT mechanical

`src/wat_edn_bridge.rs:816` currently reads:

```rust
Edn::Char(c) => Err(WatEdnBridgeError::UnsupportedEdnForm { shape: format!("Char({c:?})") }),
```

It must become a decode to `WatAST::CharLit`, and `Char` must leave the "no WatAST counterpart" list
in that function's doc comment at `:540`. **Arc 300 stone C1 already did this exact edit for
`BigInt`** — that doc line still records it (*"Arc 300 stone C1: `BigInt` now decodes to
`WatAST::BigIntLit` — no longer in this list"*). Copy that motion, including the doc-line update.

### Three comments that record the gap as a fact of the language

Each says the substrate has no `CharLit` and builds a workaround. After this stone, each is a lie:

| site | what it does now |
|---|---|
| `src/runtime.rs:21366-21377` | renders `HolonAST::Char` as `(:wat::core::char/of "c")` *"so that `(eval-ast! (to-wat char-holon))` round-trips"* — should build a `CharLit` |
| `src/closure_extract.rs:1999-2009` | encodes `Value::wat__core__Char` as a `char/of` call for portability — should build a `CharLit` |
| `crates/wat-reader/src/parser.rs:396-403` | the desugar's own comment block |

★ `runtime.rs` and `closure_extract.rs` are **round-trip pairs** — one writes, one reads. If you change
one and not the other, the round-trip breaks in a direction no build error will show you. Acceptance
row 3 is what catches that.

---

## Blast radius

`crates/wat-reader/`, `src/`, and whatever tests the cascade names. **No `.wat` corpus edits** — `\a`
already parses and keeps parsing; this stone changes what it parses *into*.

**`:wat::core::char/of` the VERB stays.** It has 17 real textual call sites and its own error surface
(length-1, BMP-only). You are changing what the reader *emits*, not deleting a verb. If you find
yourself removing the `char/of` handler, stop and read STOP-2.

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — a build error that is not a missing match arm.** A type error, a borrow error, or an
   error that suggests `CharLit` needs to carry something other than `(char, Span)`. Report the error
   verbatim and the site.
2. **STOP-2 — the change seems to require deleting or altering the `char/of` handler.** It does not.
   The verb and the literal are separate things. Report what demanded it.
3. **STOP-3 — a char literal stops round-tripping.** `\x` in, `\x` out. If any of the three sites above
   disagree with each other, report all three sites' current text rather than guessing which is right.
4. **STOP-4 — the cascade does not converge**, i.e. the error count stops falling or an arm cannot be
   written mechanically. That is a signal the variant's shape is wrong; report the arm and its file.
5. **STOP-5 — a room's line number does not hold what this brief says it holds.** Written against
   `151acb67e`. Report the mismatch rather than widening the search.

---

## Acceptance rows you can check yourself

Every one of these was run by the orchestrator at HEAD before this brief was written, so each is
reachable and the "before" value is known:

```bash
# ROW 1 — THE THESIS. At HEAD this prints ((:wat.core/char/of "a")). It must print a char literal.
cat > /tmp/d.wat <<'W'
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::read-string "\\a")))
W
./target/release/wat /tmp/d.wat

# ROW 2 — the literal forms still evaluate (all four print at HEAD)
cat > /tmp/d.wat <<'W'
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do (:wat::kernel::println \a) (:wat::kernel::println \newline)
                  (:wat::kernel::println \space) (:wat::kernel::println \tab)))
W
./target/release/wat /tmp/d.wat

# ROW 3 — round-trip. Prints \x at HEAD; must still print \x.
# ROW 5 — the VERB still works when written explicitly.
cat > /tmp/d.wat <<'W'
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do (:wat::kernel::println \x)
                  (:wat::kernel::println (:wat::core::char/of "y"))))
W
./target/release/wat /tmp/d.wat

# ROW 4 — the phantom-span census. 1461 at HEAD; must be 1411, with char/of ABSENT.
git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' > /tmp/paths.edn
./target/release/wat --grep ./wat-scripts/scratch-pad/probe-span-narrower-than-name.wat < /tmp/paths.edn | wc -l
./target/release/wat --grep ./wat-scripts/scratch-pad/probe-span-narrower-than-name.wat < /tmp/paths.edn | grep -c 'char/of'
```

Row 4 is the one that proves the phantoms are **gone rather than guarded** — it is the reason this
stone exists instead of a codemod guard.

---

## Report back with

- The cascade's waterfall: the error count after each rebuild, in order. That sequence is the honest
  record of the work's shape and the orchestrator wants it.
- The final arm count and file count, so it can be compared against NilLit's 42 / BigIntLit's 33 /
  RationalLit's 31.
- Rows 1–5, each with its actual output.
- **Every site you edited that this brief did not name** — with `file:line`. Most of them will be
  cascade-named and that is expected; call out any that surprised you.
- Anything the brief got wrong.
- What you did NOT do, and why.
