# BRIEF — STONE 255.57: `nil` is a proper type (K1), then finish 255.56

**Drawn 2026-09-27 against the local `main` @ `1d211aada`** (255.56's switch, `6225697a9`, is committed locally on a
**red** floor and is not pushed). **Executor: grok, via pulsare.** Two parts, **two commits**. Commit locally on
`main`; **do not push**. Then `pulsare_yield kind=scored` once, after part 2.

## The rulings (builder, 2026-09-27)

- **K1:** *"rip the bandaid off and make nil a proper thing … we were using rust's `()` for nil but i think we need to
  finally kill it."* `nil` stops being an alias of `Tuple([])` (`src/types.rs:2082-2106`). It becomes a **leaf type,
  the path `:wat::core::nil`**, like `i64`, with its own runtime value. `Value::Unit` is renamed `Value::Nil`. The
  **empty tuple stops existing as a type**: a tuple type needs at least one slot, the rule its constructor already
  enforces (*"tuple must have at least one element"*).
- **W1:** *"if anything is meant to convey data and only data, record is the holder for it — struct is only for things
  who must hold onto resources and non-data fields."* `:test::Wrapper` (`wat-tests/edn/roundtrip.wat:20`) becomes a
  `defrecord`.
- Standing: Z1 (`:..` is one or more), Q1 (only pure data is `Equatable`/`Orderable`), Refuse, E-a.

## Read first

- `WEIGH-STONE-255.56-the-operators-ask-the-classes.md` § "Resume weighed" and the SCORE's resume section (the 13
  reds, verbatim).
- `src/types.rs:2082-2106`: the `nil` alias, the canonicalize special-case, and the `BareLegacyUnitType` walker note.
- Measured at the draw: **40** `TypeExpr::Tuple(vec![])` / `Tuple(Vec::new())` sites meaning "unit" (`check.rs` 25,
  `types.rs` 8, `freeze.rs` 2, `declare/register.rs` 2, `closure_extract.rs`, `edn/render.rs`, `intrinsic/special/
  use_form.rs` 1 each); **13** `":()"` / `"()"` renderings; **120** `Value::Unit` uses. `:wat::core::nil` is written
  **1949** times in `.wat`, and its spelling does not change.
- `wat/class.wat`: the `nil` `Equatable` edge.

## Part 1 — commit "255.57: nil is a proper type"

1. **The type.** `:wat::core::nil` is registered as a builtin leaf (a path), not an alias. Every "unit" construction in
   `src/` builds that path. `TypeExpr::Tuple` with zero slots is refused wherever a type is parsed or built, with a
   reason that names `nil`. Nothing produces `:()`.
2. **The value.** `Value::Unit` → `Value::Nil` (a mechanical rename; keep behaviour). EDN writes and reads `nil`. `values_equal`
   keeps `(Nil, Nil) => Some(true)`. `values_compare` stays without a `Nil` arm (not `Orderable`).
3. **Renderings.** No message, `format_type`, or EDN output prints `:()` or `()` for `nil`. It prints `:wat::core::nil`
   / `nil`. Update goldens that pinned `:()` **as data**, reading the expected form. If a golden is compared as text
   today, change only its content, and name it in the SCORE (the goldens-as-data stone follows).
4. **The classes.** The `nil` `Equatable` edge in `wat/class.wat` is now an ordinary leaf edge. `nil` is not
   `Orderable`. The retired `BareLegacyUnitType` steering stays: `:()` in source remains refused, now because no such
   type exists.

## Part 2 — commit "255.56: finish" (on top of part 1)

5. **W1.** `:test::Wrapper` becomes a `defrecord` (same fields, same `:- [E]`). Its EDN round-trip and `assert-eq` rows
   pass.
6. **Scratch housekeeping.** The three scratch files that compare `_` operands
   (`255-home-10-math-stat-seq-examples.wat:25`, `255-home-12-ast-verify-examples.wat:52`,
   `255-struct-field-is-a-constant-projection.wat:57`) pin their operands with a typed consumer (a small `defn` whose
   parameters name the type, as `wat.doctest/matches?` does; no typed `let`, no `ann-form`).
   `probe-eq-generic-instantiation.wat`'s call of `eq-generic` on two functions moves to a `.wat.bad` whose Rust test
   asserts `BoundNotSatisfied` with its names. That is the hole 255.56 closes.
7. **Green across both.** Release floor, clippy, census (pre = `.census/2026-09-27T06-55-26Z.txt`), delta. Every census
   rc flip is named: the three contract goldens flip 0→1 (expected, per the 255.56 amend). Any other flip is a
   finding.

## Expectations (the orchestrator re-runs these)

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh` | all passed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| no `:()` | `grep -rn 'Tuple(vec!\[\])\|Tuple(Vec::new())' src` and a search for `":()"` renderings | none that mean `nil` (each remaining hit named) |
| census / delta | as above | only the three golden flips; NEW 2 / RECOVERY 0, same `new.txt` |

## STOP triggers (checked against the work list: none fires on a site the list orders changed)

- **STOP-1:** a **non-empty** tuple, or any non-`nil` type, changes behaviour because of part 1 (any floor red that is not
  a `:()`/`nil` rendering or a site this brief names). Quote it and STOP.
- **STOP-2:** some Rust code needs a real zero-slot tuple for something other than `nil` (a genuinely empty product).
  Report it and STOP.
- A STOP means STOP: report, do not work around it.

## Doctrine

- There is no known flake. On a red: do not re-run; quote the failing block verbatim from `.floor/`; name the arm.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.57-nil-is-a-proper-type.md` beside this brief (covering both parts). Commit with
  `git add -- <paths>`, never `-A`. **Do not push.**
