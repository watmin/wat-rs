# EXPECTATIONS — `edn::write` stops emitting unreadable keywords

| # | what | the command | expected |
|---|---|---|---|
| 1 | the two-slash keyword is gone | `(:wat::edn::write :wat::holon::Hologram/make)` | `":wat.holon.Hologram/make"` — ONE slash |
| 2 | it reads back | `(:wat::edn::read …)` on row 1's output | parses; no `more than one /` |
| 3 | the five goldens become readable | parse each `.edn` | all five parse |
| 4 | each golden changed in ONE way | `git diff` per file | only the keyword's fold |
| 5 | `try_ns` refuses a slash-bearing name | a unit test in `wat-edn` | `Err` |
| 6 | the refusal is not vacuous | construct one and watch it fail | RED, naming the name |
| 7 | no existing row changed | the registry census | **571 · 85 · 52** |
| 8 | `fqdn_of`'s code is untouched | `git diff crates/wat-macros/src/edn_doc.rs` | comment lines only |
| 9 | the floor, doctests included | orchestrator, centrally | 5166/5166 or better |
| 10 | clippy | `--all-targets -D warnings` | 0 |

Row 4 is the honesty row. `UPDATE_EDN` rewrites the whole file, so a second drift would be blessed
in the same motion — tests-green and content-integrity are separate axes, and this campaign has
already had one golden regeneration where that mattered.

Row 8 is the other one: Part 3 is a **comment** fix. If code moves there, the stone did something it
was not asked to.

## Independent prediction

**30–50 minutes.** Part 1 is small if the shared implementation drops in cleanly. Part 2 is where the
unknown is — see trap door 1.

## Trap doors — named before, not after

1. **`try_ns`'s refusal may break a live caller.** I have NOT enumerated its callers. If something
   legitimately constructs a slash-bearing name — a foreign-EDN passthrough, a test fixture — Part 2
   stops and that is the finding (STOP-1).
2. **The five goldens may not be the whole population.** I grepped `tests/**/*.edn` for two-slash
   keywords. A keyword built at runtime and asserted inline would not appear. The floor is the real
   check, not my grep.
3. **`wat_keyword_to_clojure_symbol` returns `Option`; `keyword_from_wat_path` returns
   `OwnedValue`.** Sharing the implementation means reconciling a `None` (not a head/reference) with
   the existing `Err` arm's verbatim carry. Getting that wrong turns a plain data keyword into a
   verbatim-carried one — silent, and row 3 would not catch it. Rows 1 and 7 are the guard.

## What I will do on return

Re-run rows 1–8 myself. Rows 9 and 10 are mine alone and are the only verdict on green.
