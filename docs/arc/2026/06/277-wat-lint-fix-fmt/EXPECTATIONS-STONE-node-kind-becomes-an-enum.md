# EXPECTATIONS — STONE: Node.kind becomes an enum

Every row names an **exact value or an exact shape**. And — the mirror failure this arc keeps
producing — **no row demands something the stone's own scope makes impossible**: `wat/grep.wat`'s
source changes, so its formatted output MUST differ, and row 9 says so rather than pretending
otherwise. `[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

| # | what | command | expected — EXACT |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | ★★★ **the enum mirrors `WatAST`, 14 variants, no catch-all** | read `wat/grep.wat` | `:IntLit :FloatLit :RationalLit :BigIntLit :CharLit :BoolLit :StringLit :NilLit :Keyword :Symbol :List :Vector :Set :Map` — **14**, and **no `:Unknown`** |
| 3 | ★★★ **no kind STRING literal survives in any consumer** | `grep -rc '"list"\|"vector"\|"keyword"\|"symbol"\|"map"\|"set"'` over the 49 files, **restricted to kind comparisons** | **`0`** kind comparisons remain on strings |
| 4 | ★★★ **all 155 comparisons carry a VARIANT** | census the rewritten files | **`155`**, every one `(:wat::grep::NodeKind::…)` |
| 5 | ★★★ **the boundary raises ONCE, at `grep.wat`** | read `grep.wat` | exactly **one** `:else` → `assertion-failed!` naming the unknown discriminant. **No consumer gains a raise.** |
| 6 | ★★★ **THE SYNC GATE EXISTS AND FAILS ON DRIFT** | add a 15th variant to `NodeKind` (or remove one), run the new lint | **RED**, naming the divergence. **Restore after.** Without this the stone trades one drift class for another. |
| 7 | ★★★ **and the gate PASSES on the real tree** | the new lint | green — 14 arms ↔ 14 variants |
| 8 | ★★★ **the `wat-grep` CLI tests still pass** | `tests/cli/wat_grep__*` | all green. **9 test files; wat-grep is not the formatter and has its own consumers.** |
| 9 | ★★★ **formatted output BYTE-IDENTICAL for the four files this stone does not edit** | digests captured at draw time | `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt.wat cae5250235f652ec` — **unchanged**. ⚠ `wat/grep.wat` is EXCLUDED and expected to differ: this stone edits its source. |
| 10 | ★★ width unchanged | `277-file-width-census.wat` | `deporder` **0/102** · `spawn` **9/174** · `fmt.wat` **2/147** · `io` **0/104** |
| 11 | ★★ the 614 unchanged | `run-examples.wat /tmp/fmt-614.wat` | `N=614 CHANGED=614 INLINE=284 OVER120=0 WORST=104` |
| 12 | ★★ file endings | `277-file-ends-with.wat` | `trailing-empty-parts=1` |
| 13 | ★★ no comment lost | `277-comments-both-sides.wat` | `28/28` · `85/85` · `429/429` · `45/45` (grep.wat's own count may move **only** if its source comments changed — say which) |
| 14 | ★★ idempotent | every fixture + the real files | `IDEMPOTENT=true` |
| 15 | ★★★ **a RECORDED codemod, idempotent AND effective** | `wat-scripts/fixes/<name>.wat` | second run **`CHANGED=0`** **AND** a positive control on a pre-migration file from git HEAD → **`CHANGED=1`** with the exact rewrite. **Both halves, or an inert codemod passes.** |
| 16 | ★★ the 11 recorded migrations still load | `every_wat_scripts_file_loads` | `1 passed` — they were rewritten too |
| 17 | every fixture `--check` clean | `wat --check` | clean |
| 18 | walls / hygiene | `ClaimedUnder` · `'col'` in rules · `'120'` in rules | `0` · `0` · `0` |
| 19 | floor (ORCHESTRATOR) | `scripts/floor.sh` | `5179+` run, **0 FAILED** |
| 20 | clippy (ORCHESTRATOR) | `-D warnings --all-targets` | `0` |

**Runtime prediction:** 90-150 min — the largest sweep this arc has run. The enum and the boundary are
small; 49 files, the sync gate, and the dry-run diff are the work.

## Trap-doors named in advance

- **Row 6 is the row this stone exists for.** Rows 2-5 close the *old* drift (a typo'd `"lst"`
  matching nothing) while opening a *new* one: Rust gains a 15th `WatAST` variant, `eval_ast_kind`
  goes `E0004` and the author adds an arm, and the wat side silently raises at runtime the first time
  a file contains that shape. **A migration with no gate is a trade, not a fix.**
- **Row 5 is the honest claim, and it is NOT `Break`'s.** `Break` REMOVED a raise; this one
  RELOCATES one, because `ast-kind` returns a String and a `String → enum` map cannot be exhaustive
  on its input. A SCORE that claims the raise disappeared is claiming the wrong thing.
- **Row 8 is the one a formatter-shaped mind forgets.** `:wat::grep::Node` is wat-grep's fact type
  before it is the formatter's; 7 rule files and 9 CLI tests depend on it.
- **Row 15 needs BOTH halves.** A codemod that does nothing reports `CHANGED=0` too.
- **Row 3's grep must be restricted to KIND comparisons.** `"list"` also appears as ordinary text and
  in FQDNs like `:wat::core::list`; a bare count will report phantoms.
  `[[feedback_anchor_a_census_to_the_site_never_the_substring]]`
- ⚠ **`:String` is a REFUSED variant name** — `"bare primitive type ':String' is retired (arc 109
  slice 1c)"`. Measured across all 14 candidates; it is the only one. This is why the enum mirrors
  `WatAST`'s names.
- ⚠ **`wat/*.wat` is FROZEN into the release binary.** `cargo build --release` between an edit and a
  measurement, or you are measuring the old world.
- **A very wide red mid-migration is EXPECTED.** 155 sites; the checker is the cascade
  (`docs/SUBSTRATE-AS-TEACHER.md`). Do not stash, do not revert.
