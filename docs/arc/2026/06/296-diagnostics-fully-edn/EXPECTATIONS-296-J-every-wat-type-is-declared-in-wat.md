# EXPECTATIONS — 296 J: every type wat uses is declared in wat

Written BEFORE the strike.

⚠ **THE SWEEP'S FAILURE MODE IS SILENT AND THE FLOOR IS A WEAK WITNESS.** A dropped type parameter,
a lost doc line, or a variant renamed in transcription can all leave the floor green — the corpus
mostly reaches these types through constructors that never re-check the declaration. Rows 2, 4 and 5
exist because rows 8-9 cannot see those.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⛔ zero hand-written enum literals remain | `grep -c 'TypeDef::Enum(EnumDef {' src/types.rs` | **0** |
| 2 | ⛔ every one is registered FROM WAT | `grep -c 'wat_enum_register_from!' src/types.rs` | **25** (+3 already landed = 28 total across the tree) |
| 3 | the declarations exist | `grep -rc 'defenum' wat/core.wat wat/holon.wat wat/kernel/outcomes.wat wat/edn.wat wat/eval.wat wat/stream.wat` | sums to 25 new |
| 4 | ⛔ params survive, per type | a test asserting `type_params` for each of the **9** parametric ones, in order | exact match to the retired literals. H-3's probe is the model; arity ≥2 cases (`AcceptOutcome [R S]`, `ConnectOutcome [S R]`) are the ones a fixed-offset bug passes at arity 1 and fails here — note `Connect` is `[S,R]`, NOT `[R,S]` |
| 5 | ⛔ prose survived | `git diff` + the SCORE | the 1247 doc lines are IN the wat declarations. Any line deliberately dropped is named per case (STOP-2) |
| 6 | new files joined the load set | `cargo nextest run --release -E 'test(wat_record_from_sources_are_loaded)'` | green — the existing gate catches a missed entry for free |
| 7 | the re-declarations are NoOps | the stdlib loads | no `Duplicate`; byte-equivalence held (STOP-1) |
| 8 | the floor | `./scripts/floor.sh` unpiped, Summary line | `0 failed` |
| 9 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |
| 10 | ⛔ THE WALL EXISTS AND BITES | add a hand-written `EnumDef` literal to `src/types.rs`, run the lint, then revert | the lint goes **RED**, naming the site. A wall nobody proved can fire is a wall that does not exist |
| 11 | the wall has no exemptions | `grep -c 'allow\|rune:' <the new lint>` | 0 on day one (STOP-5) |

## RUNTIME PREDICTION

**90–150 min.** 25 transcriptions is the bulk and it is mechanical; the cost centres are the 1247
prose lines (row 5), the four new files' load-set positions (row 6), and the wall plus its own
sabotage proof (row 10).

## TRAP DOORS

- **Row 10 is the stone.** Everything else is a sweep. A lint that has never been shown to fire is
  the "gate whose success condition is its own vacuity" the sibling lint's header warns about — and
  that header is in the file being copied.
- **`ConnectOutcome` is `["S","R"]` and `AcceptOutcome` is `["R","S"]`.** Opposite orders, adjacent
  types, both arity 2. Transcribing one from the other's shape is the single likeliest silent error
  in this stone.
- **Purity is an argued decision per type**, not a field to copy without reading. `types.rs`
  carries the rationale inline for several.
- **Byte-equivalence is enforced by the LOADER, loudly** — a wrong transcription refuses to load
  rather than drifting. That is the one place this sweep is protected for free; do not mistake it
  for coverage of rows 4 and 5, which it cannot see.
