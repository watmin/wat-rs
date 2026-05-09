# Arc 169 slice 1 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 30-90 minutes (opus agent).**

Reasoning:
- Arc 167 slice 1 (Vector mint + lexer + parser + tests) ran
  ~30 min opus
- Arc 169 slice 1 mirrors that shape but has THREE substrate
  consumer arms (parse_let_binding + eval_let + infer_let) where
  arc 167 had only the fn-sig consumer + value-position rejection
- The consumer arms are mechanical (mirror the existing Symbol +
  Vector arms); the StructPattern node itself is the foundation
- Test surface is ~11 cases vs arc 167 slice 1's 9 cases; same
  shape

**Time-box (2× upper-bound): 180 minutes.** If opus still
iterating at 90 min, in-flight check; hard cap at 180.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — `WatAST::StructPattern` variant minted | grep `WatAST::StructPattern` src/holonast.rs (or wherever WatAST lives): variant exists with Vec<WatAST> + Span fields | ✓ |
| B — `TAG_STRUCT_PATTERN` distinct tag | tag byte chosen + reported; distinct from existing tags | ✓ |
| C — Lexer mints `LBrace`/`RBrace` tokens | `{` and `}` produce Token::LBrace / Token::RBrace | ✓ |
| D — Parser produces StructPattern from `{...}` | `{outcome residue}` parses to `WatAST::StructPattern([Symbol("outcome"), Symbol("residue")], span)` | ✓ |
| E — Empty `{}` → clean MalformedForm | parser rejects with diagnostic naming the position | ✓ |
| F — Non-Symbol inside `{}` → clean MalformedForm | `{42}`, `{"x"}`, `{(...)}`, `{[a]}` all rejected at parse with naming the offending shape | ✓ |
| G — `parse_let_binding` StructPattern arm | third arm added; produces `LetBinding::StructDestructure { field_names, rhs }` | ✓ |
| H — `eval_let` / `step_let` runtime arm | each field name resolved against struct value; bindings emitted into local scope | ✓ |
| I — `infer_let` / `process_let_binding` check arm | rhs must be Struct; each field validated against struct's fields; unknown field → MalformedForm; type-mismatch on rhs → TypeMismatch | ✓ |
| J — All 11 test cases pass | new tests/wat_arc169_struct_destructure.rs file 11/11 green | ✓ |
| K — Workspace stays clean | inline pipeline shows `passed: N+11 failed: 0` (N = pre-slice-1 count of 2080) | ✓ |
| L — Slice branch on remote | branch carries the slice 1 commit(s); main untouched | ✓ |
| M — `wat/core.wat` defn macro untouched | `git diff wat/core.wat`: no changes | ✓ |
| N — No FM 10 reach | substrate gains a NEW ENTITY KIND (StructPattern AST variant); no type-system feature added; no Map-overload-via-shape-inspection bridge | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Struct registry lookup mechanism gap.** If the type-checker
  doesn't expose a clean way to look up fields by name on a struct
  type, surface as honest delta. The TypeEnv has `lookup_struct` or
  similar; verify before designing around it.
- **Hash identity / EDN serialization implications.** If
  StructPattern's hash needs special handling for arc 092 EDN
  roundtrip, surface — may affect tag byte choice or serialization
  arm.
- **Display / pretty-print integration.** If existing
  `render_value` / `Display` arms must learn StructPattern, surface
  the touched sites; if more than ~5 sites, may justify a sub-slice.
- **Parse-time validation surprises.** If shape inspection (all-
  Symbol contents) interacts oddly with macro-expanded forms (e.g.,
  defmacro emitting `{...}` content with non-Symbol children),
  surface — may require parse-time error vs check-time error
  reclassification.
- **FM 5 trap.** If you find yourself wanting to "just accept
  non-Symbol inside `{}` and let the consumer reject" or "just
  reuse Map and disambiguate by shape," STOP. The DESIGN settled
  on parse-time-shape-validation + StructPattern-as-purpose-built
  via four-questions. Bridging breaks the long-term-stability
  property.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 30-90 min band.

Sites touched by file:
- `src/lexer.rs`: ___ sites
- `src/holonast.rs` (or equivalent): ___ sites
- `src/parser.rs`: ___ sites
- `src/runtime.rs`: ___ sites
- `src/check.rs`: ___ sites

Honest deltas surfaced: ___ (count + brief).

Tag byte chosen: 0x___

## What's next (orchestrator-side, post-slice-1)

When slice 1 ships green:
- Slice 2 closure paperwork:
  - SCORE-SLICE-1 (this slice)
  - INSCRIPTION (arc 169 closure)
  - 058 changelog row (lab repo)
  - USER-GUIDE update (let section gets struct-destructure example)
  - Atomic squash-merge to main
- Arc 109 v1 milestone closure unblocks (per arc 109 INVENTORY § M)

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-1.md
to slice branch after scoring all rows + reviewing the diff +
re-running the inline pipeline locally for FM 9 verification.
