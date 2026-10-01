# BRIEF — STONE 255.77: cutover stone 3 — the UUID type goes home: `:wat::core::Uuid` → `wat.uuid/UUID`

**Drawn 2026-10-01 against `main` @ `c16915636`.** **Executor: a Sonnet subagent, working solo** (so it runs the floor,
in the foreground; the builder's rule 2026-10-01). A strike: `src/` (the type's key and its literals), a recorded
codemod over the `.wat` sites, tests. Commit locally on `main` (`git add -- <paths>`, never `-A`); **do not push**. Your
final message is your report.

## The rulings

- **The cutover's end state (2026-09-27):** `wat.type/` holds only the 24 hard primitives; other typed things live in their
  own homes: `wat.time/Instant`, **`wat.uuid/UUID`**, `wat.holon/HolonAST`. `wat.core` is utility, never a type home.
  The sequence (`WEIGH-STONE-255.65-size-the-cutover.md`): stone 3 is the home renames, and the measurement found only
  `Uuid` actually moves (`Instant`/`Duration`/`HolonAST` already live in their homes).
- **U1 (builder, 2026-10-01):** one stone, no T-door. The type's key becomes **`:wat::uuid::UUID`**, spelled in `.wat` as
  **`wat.uuid/UUID`** (`canonical_identity` maps the symbol to that key; the orchestrator confirmed a symbol-spelled home
  type works as a header, a record field and a type argument with `wat.time/Instant`). The old spelling
  `:wat::core::Uuid` is **refused with a retirement remedy** naming `wat.uuid/UUID`, through the existing retirement
  table (`src/remedy/retirement.rs`, the `:wat::core::Uuid/v4 → :wat::uuid::v4` rows are the precedent).
- Equality is data equality; never compare a type by its name string (2026-09-27).

## The measurement (orchestrator, at `c16915636`)

- **Not changing:** the verbs already live in `:wat::uuid::` (`v4`, `v5`, `nil`, `from-string`, `to-string`, `version`,
  `rfc4122-variant?`); values print as standard EDN `#uuid "…"` (`crates/wat-edn/src/writer.rs:231`), so no tag or golden
  changes are expected. If a golden does change, that is a finding: report it.
- **`.wat` type positions (11):** `tests/rete/probe_arc278_6a_purity.wat:7`, `tests/types/probe_arc241_stone8_defstruct_c04.wat:6-7`,
  `tests/types/uuid_v4_returns_typed_uuid.wat:1` (comment), `tests/value/wat_arc221_keyword_nil_tag_atomization.wat:119,130`,
  `wat/class.wat:58`, `wat/service.wat:128`, `wat/telemetry.wat:15` (comment), `:77`, `wat/telemetry/journal.wat:30`,
  `wat/telemetry/span.wat:20`. Re-derive this list yourself (the codemod's census), including any `.wat` inside Rust
  string literals.
- **Rust:** `src/types.rs:3560` and `:9267` (registration lists); `src/edn/render.rs:2611` (decode arm);
  `src/function/subsume.rs:285`; `src/intrinsic/uuid.rs:95,188,241,279` (`expected:` messages); and the variant
  `Value::wat__core__Uuid` (23 uses across `src/closure_extract.rs`, `src/edn/render.rs`, `src/function/subsume.rs`,
  `src/holon/ast.rs`, `src/intrinsic/uuid.rs`, `src/runtime.rs`, `src/value/observe.rs`, `src/value/value.rs`), whose
  name encodes the retired path: rename it (FM 14, a retired name leaves no live internal identifier).
- **The trap, `wat/telemetry.wat:305-322`:** `framing-floor-of` sizes a record's wire frame by comparing
  `(:wat::core::ast-name t)` to strings, including `"wat.type/Uuid"` → 36 bytes. After the rename that string no longer
  matches and the branch falls to `:else 0` **silently** (a record with a UUID field gets a frame budget 36 bytes short;
  nothing goes red). Note also that `field-types-of` renders the type today as `wat.type/Uuid`, which is not one of the
  24: find out why (a renderer mapping `:wat::core::X` to `wat.type/X` without asking the closed set), and say.

## The work

1. **Pre-census** (`scripts/replay/census.sh`) on the unmodified draw, and a test that **pins today's
   `framing-floor-of`** for a record with a UUID field and for one with each of the other three typed branches (the
   numbers it returns today, before any change). This test must still pass after the rename.
2. **The key:** register the type as `:wat::uuid::UUID`; rename the Rust literals and the `Value` variant; the old
   `:wat::core::Uuid` (and any `wat.core/Uuid` / `wat.type/Uuid` spelling the door accepts today) is refused with a
   retirement remedy naming `wat.uuid/UUID`.
3. **The corpus:** a recorded codemod `wat-scripts/fixes/uuid-type-goes-home.wat` (copy the shape of
   `rename-four-families-to-their-homes.wat`) rewrites every type-position spelling to `wat.uuid/UUID`; dry-run on a `/tmp`
   copy and `diff`, then apply listing every path; idempotent. If the old spelling must be refused in the same stone,
   read `wat/fix.wat`'s header STASH-DANCE note. Wat inside Rust string literals: edit directly.
4. **The trap:** `framing-floor-of` compares each field type **as data** (the type node against the expected type node, or
   the type's canonical key through the type door), not by `ast-name` text, for all four branches. The renderer finding
   from the measurement: cure it if it is a renderer that invents `wat.type/` for a non-primitive (route it through the
   closed set), or report it if the cure is wider than this stone.
5. **Tests:** the retirement refusal names `wat.uuid/UUID`; `wat.uuid/UUID` checks and runs in a header, a record field,
   a set element and a map key (it is Equatable); a UUID value round-trips through EDN unchanged; the pinned
   `framing-floor-of` numbers hold.

## Gates

| what | how | expected |
|---|---|---|
| the pinned frame floors | your test from item 1 | identical before and after |
| no old spelling left | a search for `:wat::core::Uuid`, `wat.core/Uuid`, `wat.type/Uuid`, `wat__core__Uuid` | only the retirement row, its test, recorded codemods' own text, and history in docs |
| idempotent | the codemod over the converted files | 0 changes |
| census | `scripts/replay/census.sh --diff` against your pre-census | no rc flips except files you converted (list each) |
| release floor | `scripts/floor.sh`, **in the foreground**, nothing else running | all passed; the count against 6336 at `e266c8e1a`, plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change (a site the rename reaches, a golden whose only change is the spelling, its own
  tests): capture it **verbatim** from `.floor/<stamp>/`, cure it, run a **new** floor. Never re-run unchanged code for a
  green. The prior green floor is `.floor/2026-10-01T08-16-09Z` (6336/6336).
- **STOP-1:** the printed form of a UUID value or of a type in a user-facing message changes in a way that is not just
  this rename (an EDN tag, a golden's structure). Quote it and STOP.
- **STOP-2:** a `framing-floor-of` pinned number changes. Quote both and STOP.
- **STOP-3:** a Uuid site outside the measured list that needs a design decision (a registry, a reflection verb, a
  persisted form on disk). List it and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you (`.wat` migrations by the recorded codemod). Capture `rc=$?` on the next statement. Never wait
with `pgrep -f`. Run every build, floor and clippy in the foreground and block on it. Never write a number, file:line or
example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.77-the-uuid-goes-home.md` beside this brief, commit it, **do not push**.
