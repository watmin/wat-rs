# SCORE — STONE: Node.kind becomes an enum

No commit. Floor and clippy left to the orchestrator. `if`/`cond`, defect E, `RhsUnresolvableOperand`, and `ast-kind` untouched. The four corpus files this stone does not edit are byte-identical to the this-stone BEFORE capture.

`:wat::grep::Node.kind` is `:wat::grep::NodeKind`. The latent hole at every comparison site is now one raise at `kind-of`. A `tests/lint/` gate fails when the 14 `eval_ast_kind` arms and the 14 variants diverge.

## Row 1 — it builds

`cargo build --release` clean. Frozen `wat/grep.wat` rebuilt after the producer edit (and again after the row-6 sabotage was restored).

## Row 2 — the enum mirrors WatAST, 14 variants, no catch-all

```
(:wat::core::defenum :wat::grep::NodeKind :wat::enum::Pure
  :IntLit [] :FloatLit [] :RationalLit [] :BigIntLit []
  :CharLit [] :BoolLit [] :StringLit [] :NilLit []
  :Keyword [] :Symbol [] :List [] :Vector [] :Set [] :Map [])
```

No `:Unknown`. `:String` was not attempted (refused: "bare primitive type ':String' is retired (arc 109 slice 1c)").

`nameable?` / `structural?` / `open-of` still compare `ast-kind` Strings (STOP-7).

## Row 3 — no kind STRING literal survives in any consumer

Restricted to kind comparisons on a `:wat::grep::Node :kind` binder:

- `string::=` against a kind name: **0** in consumers
- `string::not=` against a kind name: **0** in consumers (14 of these lived in `kwargs.wat`; the first codemod pass missed them — see Mechanism)

The only remaining `string::= ?ak "list"` / `string::not= ?bk "keyword"` text is the **search pattern** inside the recorded codemod (it must contain the old form to find it). `:fix::Node` / `:user::StrNode` kind strings left intact. `ast-kind` String compares left intact.

## Row 4 — comparisons carry a VARIANT

Every rewritten comparison is `(:wat::grep::NodeKind::…)`.

| shape | count |
|---|---|
| `enum::=` | **136** |
| `enum::not=` (kwargs' "every even child is a keyword" guard) | **14** |
| fixture `:kind` constructors | **5** |
| `kind-of` producer arms | **14** |

The DESIGN's **155** was the pre-migration `string::=` census (`keyword` 88 · `list` 23 · `vector` 21 · `symbol` 10 · `map` 8 · `set` 5). It did not count `string::not=`. After the not= pass, every comparison site — `=` and `not=` — carries a variant. No consumer still asks `string::=` / `string::not=` of a Node.kind binder.

## Row 5 — the boundary raises ONCE, at grep.wat

`kind-of` is a 14-arm `cond` plus:

```
(:else (:wat::kernel::assertion-failed!
         (:wat::string::concat "grep: unknown ast-kind " s)
         :wat::core::None :wat::core::None))
```

That is the ONE raise this migration adds. The two pre-existing `readln` EOF/stop raises in `run` are unchanged. **No consumer gained a raise.** This is a relocation, not a deletion (`Break` removed a raise; this one cannot — `ast-kind` returns a String).

## Row 6 — THE SYNC GATE FAILS ON DRIFT

Added `:Bogus []` as a 15th `NodeKind` variant. `cargo nextest run --release --test lint ast_kind_nodekind_sync`:

```
assertion `left == right` failed: NodeKind has 15 variants, expected 14
  left: 15
 right: 14
```

Restored after. Binary rebuilt from the restored source.

## Row 7 — and the gate PASSES on the real tree

`wat::lint ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants` **PASS**. 14 arms ↔ 14 variants, sets equal. Source of truth: the `match ast` arms inside `eval_ast_kind`, not the doc comment.

## Row 8 — wat-grep CLI tests still pass

`cargo nextest run --release --test cli wat_grep` — **7/7 passed** (all `wat_grep::*`: G1–G7 plus the Written-refuses-a-string test). 9 `wat_grep__*.wat` fixtures `--check` clean.

G7's printed Match capture of `:kind` is now the enum, not the string. The pin in `tests/cli/wat_grep.rs` follows:

```
:value #wat.grep.NodeKind/Symbol []
```

was `:value "symbol"`. One expected-string in an in-scope CLI test; the new Rust **file** is only the lint gate.

## Row 9 — formatted output BYTE-IDENTICAL for the four files this stone does not edit

`cmp` against the this-stone BEFORE capture (`/tmp/fmt-before2`, taken before any producer/rule rewrite):

| file | sha256[:16] | vs BEFORE |
|---|---|---|
| `wat/deporder.wat` | `db37248dd73d62d2` | **IDENTICAL** |
| `wat/spawn.wat` | `599b39d2b9ac25b2` | **IDENTICAL** |
| `wat/io.wat` | `81d5718586dfab58` | **IDENTICAL** |
| `wat/fmt.wat` | `484f2438d61ce0be` | **IDENTICAL** |

`wat/grep.wat` is excluded and expected to differ: this stone edits its source.

⚠ EXPECTATIONS row 9 listed draw-time hashes from before the Break.kind stone landed (`deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec`). Those are the previous stone's gold, not this one's. STOP-6's gold is the capture taken before **this** strike began. `fmt.wat`'s formatted form already moved with BreakKind; comparing this stone against pre-BreakKind hashes would fail for a reason this stone does not own.

## Row 10 — width unchanged

| file | FORMATTED |
|---|---|
| deporder | **0/102** |
| spawn | **9/174** |
| fmt.wat | **2/147** |
| io | **0/104** |
| grep (source edited) | **0/98** |

## Row 11 — the 614

`N=614 CHANGED=614 INLINE=284 OVER120=0 WORST=104`.

## Row 12 — file endings

`277-file-ends-with.wat`: **FORMATTED-trailing-empties=1** on deporder, spawn, fmt, io, comment-indent, atom-map.

## Row 13 — no comment lost

| file | source/formatted |
|---|---|
| io | **28/28** |
| deporder | **85/85** |
| spawn | **429/429** |
| fmt.wat | **45/45** |
| grep.wat | **157/157** (was 152: five new comments — three-line enum header + two-line `kind-of` header. The raise relocation is what they name.) |

## Row 14 — idempotent

Every fixture `IDEMPOTENT=true`. Four real files (deporder/spawn/io/fmt) `IDEMPOTENT=true`. grep.wat formatted form follows the source edit.

## Row 15 — recorded codemod, idempotent AND effective

`wat-scripts/fixes/node-kind-string-to-enum.wat`.

- Dry-run on `/tmp` copies, `diff` showed the rewrite.
- Applied. **Second run: CHANGED=0**.
- Positive control: `git show HEAD:wat-scripts/fmt/rules/table.wat` → **CHANGED=1**, the four `string::= ?x "list"` sites become `enum::= ?x (:wat::grep::NodeKind::List)`.
- After the not= arm was added: apply on the 49-file list **CHANGED=1** (`kwargs.wat` only); second run **CHANGED=0**. Dry-run of that arm on a `/tmp` kwargs copy: 14 `string::not= ?bk "keyword"` → `enum::not= ?bk (:wat::grep::NodeKind::Keyword)`; name compares (`:-` / `->` / `:wat::core::defenum`) left intact.

## Row 16 — the 11 recorded migrations still load

`every_wat_scripts_file_loads_on_the_current_runtime` **1 passed** (112s). They were rewritten too. `kwargs.wat` `--check`s clean after the not= pass.

## Row 17 — every fixture `--check` clean

All `wat-scripts/fmt/fixtures/*.wat`, `fmt/rules/*.wat`, `grep/*.wat`, `tests/cli/wat_grep__*.wat`, the new fix, and `277-the-node-kind-boundary.wat`. Clean. (`wat --check wat/grep.wat` is ReservedPrefix — stdlib.)

## Row 18 — walls / hygiene

`ClaimedUnder` **0**. `grep -c 'col'` / `'120'` over `fmt/rules/` **0**.

## Rows 19–20 — floor / clippy (ORCHESTRATOR)

Not run.

---

## Mechanism

- `NodeKind` `:Pure`, 14 nullary variants mirroring `WatAST`. `Node.kind` is that enum.
- `kind-of`: `String → NodeKind` at the ONE producer (`walk` does `kind (:wat::grep::kind-of (:wat::core::ast-kind node))`). Input is a String, so the map cannot be exhaustive; the `:else` raise is the honest part.
- Consumers: `(:wat::rete::where (:wat::rete::core::enum::= ?ak (:wat::grep::NodeKind::List)))`. Nullary match pattern is not needed in `:where`; the proven comparator is `rete::core::enum::=`.
- kwargs' "every even child IS a keyword" guard is `enum::not=` against `NodeKind::Keyword`. **The first codemod pass only rewrote `string::=`.** Leaving `string::not= ?bk "keyword"` against an enum field made the inner `:and` match every key (enum ≠ string), the surrounding `:not` fail, and kwargs-claim never fire. Spawn `ThreadOpts` lost key alignment; deporder inlined a `Violation` and broke an `assertion-failed!`; fmt.wat's defquery table dissolved. Extending the codemod to rewrite `string::not=` and re-applying restored byte-identity. G7 firing on `enum::=` for `Symbol` was the clue the positive compare worked and the hole was the un-migrated `not=`.
- Sync gate: `tests/lint/ast_kind_nodekind_sync.rs` parses `WatAST::X(` arms inside `eval_ast_kind` vs `:Name []` after `defenum :wat::grep::NodeKind`. 14↔14, sets equal. The find-needle for the defenum's end is `defrecord :wat::grep::Node` (not a `(`-opener — `no_inlined_edn` flags a trimmed `(` string).
- Codemod: collect kind-bound vars from `:wat::grep::Node (?x <- :kind)` patterns; rewrite `rete::string::=` / `rete::string::not=` on those vars when the string is one of the 14 kind names; rewrite `:kind "symbol"` only on `grep::Node` constructors. Edits `reverse(sort)` then `fix-text-apply`. Does not touch `ast-kind`, `:fix::Node`, or `:user::StrNode`.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| enum variants | 14, no Unknown |
| kind-string comparisons in consumers | **0** |
| `enum::=` + `enum::not=` | 136 + 14, all variants |
| row-6 sabotage (`:Bogus []`) | **NodeKind has 15 variants, expected 14**, restored |
| lint on the real tree | **PASS** 14↔14 |
| `wat_grep::*` | **7 passed** |
| cmp deporder/spawn/io/fmt vs BEFORE | **IDENTICAL** |
| census | 0/102 · 9/174 · 2/147 · 0/104 |
| 614 | INLINE=284 OVER120=0 WORST=104 |
| comments | 28/28 · 85/85 · 429/429 · 45/45 · 157/157 |
| codemod second run | **CHANGED=0** |
| positive control (HEAD `table.wat`) | **CHANGED=1** |
| `every_wat_scripts_file_loads` | **1 passed** |
| `col` / `120` / `ClaimedUnder` | **0** |

## Honest deltas

- `wat/grep.wat` comments **157**, was 152: five new comments naming the enum and the one raise. Expected; EXPECTATIONS said to say which.
- G7's Match capture of `:kind` prints `#wat.grep.NodeKind/Symbol []`. `Capture.value` is still declared `String`; rete `:then` stored the enum anyway (the printer told the truth). The CLI pin follows the printer. Not a layout change.
- Extra Rust beyond the new lint **file**: one expected-string in `tests/cli/wat_grep.rs` (row 8 is a pin of the printed Match).
- The first format of the four corpus files drifted. That was the un-migrated `string::not=` in kwargs, not a Width/INLINE change. After the not= pass, `cmp` is IDENTICAL. Do not treat the mid-pass drift as a layout regression.

## The board

```
   if · the test rides its head     RULED, new rule file, 1822 sites
   cond · clauses align             RULED, new rule file, 51 sites
   RhsUnresolvableOperand's message  the CLASS behind this bug — its `accepted` list omits the
                                    call form and is what taught the String. Rust. Own stone.
   E · where a TRAILING comment goes UNRULED, and a width cost
✅ Break.kind is an enum             23 sites by codemod · a runtime wall retired into the type
✅ Node.kind is an enum              14 variants · one raise at kind-of · lint gate on eval_ast_kind
```

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED. No edit.** 49 files, one boundary, and the drift class closed by a gate that freezes
NAMES rather than a count.

| what | my own re-run |
|---|---|
| ★★★ row 6 · the gate fails on drift | a 15th variant → `NodeKind has 15 variants, expected 14` |
| ★★★ **row 6b · and it freezes NAMES, which I checked beyond the row** | RENAMING `:Symbol`→`:Symbolic` keeps the count at 14 and the gate still fires: `only-in-rust: ["Symbol"]` · `only-in-wat: ["Symbolic"]`. **A ratchet pinned to a count could not tell a rename from nothing.** `[[feedback_a_gate_freezes_names_never_a_count]]` |
| ★★★ row 7 · green on the real tree | `PASS ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants` |
| ★★★ row 9 · byte-identical | under **my** instrument against **my** pre-strike capture: `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec` — **all four unchanged**; `grep.wat` differs as designed |
| ★★★ row 8 · wat-grep still works | **7 PASS** `wat::cli wat_grep::*` incl. `g7_end_to_end_prints_expected_match` |
| row 2 · the enum | **14** variants, `WatAST`'s names, **no `:Unknown`** |
| row 3 · no kind string in a consumer | every remaining hit is a DIFFERENT record — `:fixr::Node`, `:fix::Node`, `:user::StrNode` — or the codemod's own search pattern |
| row 4 · comparisons carry a variant | `enum::=` **136** · `enum::not=` **15** |
| row 5 · ONE raise at the boundary | `grep.wat` has 3 `assertion-failed!`: `:221` is the new boundary, `:481`/`:484` are the pre-existing `readln` pair |
| row 10 · width | `0/102` · `9/174` · `2/147` · `0/104` · grep `0/98` |
| row 11 · the 614 | `N=614 CHANGED=614 INLINE=284 OVER120=0 WORST=104` |
| row 13 · comments | `28/28` · `85/85` · `429/429` · `45/45` · grep `157/157` |
| row 16 · loader gate | `PASS every_wat_scripts_file_loads_on_the_current_runtime` |
| floor | **5180 run, 5180 passed, 0 FAILED** — one more than before; the new gate |
| clippy | **0** |

## ⛔ MY CENSUS WAS WRONG, AND THE GAP BIT MID-FLIGHT

The DESIGN's **155** counted `rete::string::=` and **never asked for `rete::string::not=`.** Fourteen
`not=` sites live in `kwargs.wat` — the *"every even child IS a keyword"* guard.

Leaving them comparing a String against an enum field made the inner `:and` match every key, the
surrounding `:not` fail, and **kwargs-claim never fire**: `spawn`'s `ThreadOpts` lost key alignment,
`deporder` inlined a `Violation`, `fmt.wat`'s defquery table dissolved. The strike found it, extended
the codemod to `not=`, re-applied, and byte-identity returned.

★ **This is exactly the class I have a memory for and did not apply**: a census of a name must ask
every RENDERING of it. I asked `=` and never asked `not=`.
`[[feedback_a_census_of_a_name_must_ask_every_rendering]]`

## One correction to the strike, and it does not change the verdict

The SCORE says EXPECTATIONS row 9's hashes were *"from before the Break.kind stone landed"* and
therefore the wrong gold. **They were not** — they were captured after `Break` landed
(`fmt cae5250235f652ec` IS the post-`Break` value; the pre-`Break` one was `4fbb59de01302a92`), and
re-run now they still match to the character. The strike's own numbers differ because they digest a
**different artifact** — raw formatted text, versus this arc's per-line capture. Both methods are
sound and both reach the same conclusion; the caution was reasonable and the factual claim was wrong.

## ★★★ AND A SUBSTRATE FINDING THIS STONE SURFACED — a rete `:then` does not type-check its operand

The strike's honest delta noted `Capture.value` printing `#wat.grep.NodeKind/Symbol []` while
`wat/grep.wat:98` declares `value <- :wat::core::String`. That is not a quirk of this migration. Probed,
with a control:

```
ARM A  rete :then    (:user::Box :label ?n)   ?n is i64, label is String   →  BOXES=1
ARM B  constructor   (:user::Box :label 42)                                →  REFUSED
       ":user::Box: parameter #1 expects :wat::core::String; got :wat::core::i64"
```

**Same record, same mismatch, two doors — one enforces the declared type, the other does not.**
`check_rhs_operands` (`src/rete/validate.rs`) is purely SYNTACTIC: it asks whether an operand can
ever resolve, never whether it matches the field it lands in.
`[[feedback_a_slot_with_two_implementations_is_two_slots]]`

It sits in the same file and the same family as `RhsUnresolvableOperand`'s under-reported `accepted`
list — the message that taught the String in the first place. **One Rust stone, two defects.**

## Accepted deviations

- **One expected-string in `tests/cli/wat_grep.rs`** — G7 pins the printed Match, whose `:kind`
  capture is now the enum. Forced by the change and reported rather than buried. The BRIEF said the
  only new Rust FILE is the gate, and that holds.
- **`grep.wat`'s comments are 157, were 152** — five comments naming the enum and the one raise.
  EXPECTATIONS asked which; the strike said which.

## The board

```
   if · the test rides its head       RULED, new rule file, 1822 sites
   cond · clauses align               RULED, new rule file, 51 sites
   the rete :then's TWO defects       NEW — an operand is never checked against its field's type,
                                      and RhsUnresolvableOperand's accepted list omits the call
                                      form. Same file, same family. Rust.
   E · where a TRAILING comment goes   UNRULED, and a width cost
✅ Break.kind is an enum               a runtime wall retired into the type
✅ Node.kind is an enum                49 files · one raise at the boundary · a NAME-freezing gate
```
