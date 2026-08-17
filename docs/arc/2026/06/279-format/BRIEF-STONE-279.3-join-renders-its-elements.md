# BRIEF — 279.3 · `join` renders its elements (chain-D, the join half)

> **Rewritten 2026-08-17 to option A.** The first version of this brief specified a wat `defn`. A
> rider built it exactly and the **stdlib died before `main`** — `core.wat` ↔ `string.wat` is a real
> dependency cycle and the Rust intrinsic is what breaks it. That version is gone; what follows is
> the ruled design. Nothing below asks you to define anything in wat.

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming, and a Monitor cannot wake you either. Run every verification in the
**FOREGROUND** and block on it: your turn ends when the numbers are in your hands, not when the
command is launched.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.**

## The work in one paragraph

`(:wat::core::string::join "," [1 2 3])` does not type-check today, because `join`'s `TypeScheme`
hardcodes `Vector<String>` and its evaluator demands `Value::String` per element. Make `join`
generic over the element type and render each element through the **total** `str` (279.2), so it
behaves like Ruby's `ary.join(',')`. Three edits in two files, plus one new shared function.

## Read in order

1. `docs/arc/2026/06/279-format/DESIGN-STONE-279.3-join-renders-its-elements.md` — the ruling. Read
   the `⛔ CORRECTION` section and everything after it; the part **above** that banner is the
   superseded wat-defn design, kept only as record.
2. `src/runtime.rs:23340-23367` — `eval_str`. **This is the shape you are extracting.** Its two-arm
   match IS `str`'s totality.
3. `src/string_ops.rs:455-508` — `eval_string_join`, the evaluator you are changing.
4. `src/check.rs:16598-16610` — `join`'s `TypeScheme`, the signature you are changing.
5. `src/check.rs:17056-17071` — `:wat::eval-ast!`'s scheme. **The exemplar for a generic scheme**:
   `type_params: vec!["T".into()]` with `TypeExpr::Path("T".into())` inside a `Parametric`'s args.
   Copy this shape.

## The four moves

### 1 — Mint the one door, in `src/string_ops.rs`

```rust
pub(crate) fn render_str_total(v: &Value, types: Option<&crate::types::TypeEnv>) -> String {
    match v {
        Value::String(s) => (**s).clone(),
        other => crate::edn_shim::value_to_edn_string_with(other, types),
    }
}
```

Place it next to `render_unquoted` (`string_ops.rs:510`). Doc-comment it as **`:wat::core::str`'s
rendering, factored so `str` and `join` cannot drift** — and say why each arm exists (below).

### 2 — `eval_str` (`runtime.rs:23359`) becomes its first caller

Replace the inline match with `crate::string_ops::render_str_total(&v, sym.types().map(|a| a.as_ref()))`.
Behaviour must be **byte-identical** — this is a pure extraction. Keep the existing 296/279.2
comment; move it onto the door if that reads better.

### 3 — `eval_string_join` (`string_ops.rs:493-506`) renders instead of demanding

The `for item in pieces.iter()` loop currently matches `Value::String` and **errors on anything
else**. That whole arm goes: each element becomes
`render_str_total(item, sym.types().map(|a| a.as_ref()))`. The `expected: "Vec<String>"` mismatch on
the *pieces argument itself* (`string_ops.rs:488`) **stays** — a non-Vec second argument is still an
error; update its `expected` string to `"Vec<T>"` so it stops lying.

### 4 — The `TypeScheme` (`check.rs:16598`) goes generic

```rust
env.register(
    ":wat::core::string::join".to_string(),
    TypeScheme {
        type_params: vec!["T".into()],
        params: vec![
            string_ty(),
            TypeExpr::Parametric {
                head: "wat::core::Vector".into(),
                args: vec![TypeExpr::Path("T".into())],
            },
        ],
        ret: string_ty(),
        rest_param_type: None,
    },
);
```

⚠ **`TypeExpr::Var` is NOT the type-parameter constructor.** `Var(u64)` is a synthetic unification
variable the checker allocates; a scheme's own parameter is a `Path` (`types.rs:73-98`). If you find
yourself writing `Var("T")`, you are following an earlier draft that was wrong — use the exemplar at
`check.rs:17056`.

### 5 — Truth in comment (one line, same file)

`render_unquoted`'s doc comment (`string_ops.rs:510`) claims to be *"the `:wat::core::str`
semantics"*. It has not been since `25d9d015`; `str` is `render_str_total`, and `render_unquoted`'s
only remaining caller is `interpolate` (`string_ops.rs:602`). Correct the comment to say what it is.
**Do not change `interpolate`'s behaviour** — see Out of scope.

## ★ THE TWO ARMS ARE BOTH LOAD-BEARING — do not collapse either

- **`Value::String` → bare.** The EDN encoder *quotes* strings. Route a top-level String through it
  and `(join "-" ["a" "b"])` becomes `"\"a\"-\"b\""`, silently corrupting **26 live call sites**.
- **The `types` argument.** Drop it and a record renders `{:field-0 1}` instead of `{:x 1}`. That is
  exactly the 296/279.2 fix whose comment sits above the line you are extracting.

Both are why this is **one door and not two hand-rolled matches**. If you find yourself writing the
two-arm match a second time inside `eval_string_join`, stop — that is the defect this shape exists
to prevent.

## The gate

| # | assertion |
|---|---|
| 1 | `(join "," [1 2 3])` → `"1,2,3"` — the row that does not work today |
| 2 | ★ `(join "-" ["a" "b"])` → `"a-b"` — **BARE, not `"a"-"b"`.** The Ruby contract AND the proof that per-element `str` did not start re-quoting |
| 3 | all **26** pre-existing live call sites green (`wat/` 13 · `tests/` 7 · `wat-scripts/` 5 excluding the 279.3 probe · `wat-tests/` 1) — the floor covers this; name how you confirmed it |
| 4 | `render_str_total` exists, and **both** `eval_str` and `eval_string_join` call it — `grep -c` it |
| 5 | `join`'s scheme has `type_params: vec!["T".into()]` and `Path("T")`; no `Vector<String>` remains for it |
| 6 | `render_unquoted`'s doc comment no longer claims to be `str` |
| 7 | a **kept** test covers rows 1 and 2 — `tests/kernel/wat_string_ops.wat` + its driver `tests/kernel/wat_string_ops.rs` (see `compute-split-join` / `assert_str` at `.rs:72` for the shape). Not a scratch probe you delete |
| 8 | floor GREEN via `scripts/floor.sh` — read the **Summary line**, never a piped exit code |
| 9 | `cargo clippy --release --all-targets` → **0** |
| 10 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13** |

Row 2 is the load-bearing row. Row 4 is what makes rows 1 and 2 stay true next year.

## What you report

- the `git diff` of `src/string_ops.rs`, `src/runtime.rs`, `src/check.rs`, and the two test files
- **measured output, verbatim**, for `(join "," [1 2 3])` and `(join "-" ["a" "b"])`
- the kept test's name and the command that runs it
- floor Summary **verbatim**; clippy count; `#[ignore]` count
- honest deltas — anything that surprised you

## STOP triggers — ship nothing on that axis; report and stop

- **STOP-1 — a `.wat` call site does NOT type-check** against `Vector<T>` where it used to against
  `Vector<String>`. It should: `Vector<String>` unifies at `T = String`. If one does not, name the
  site and the error — that is a finding about unification, not something to paper over with a
  second clause or a second scheme.
- **STOP-2 — row 2 renders `"a"-"b"`.** Then the String arm is missing or the encoder is being
  reached for a top-level string. **Do NOT special-case strings downstream to fix it** — find which
  arm was lost. If both arms are present and it still quotes, that is a finding about
  `value_to_edn_string_with`, not about this stone.
- **STOP-3 — extracting `eval_str` changes any `str` output.** Move 2 is a pure refactor; if a
  `str`-related test moves, the extraction is not faithful. Report the diff in behaviour.
- **STOP-4 — the `#[ignore]` count moves off 13.**
- **STOP-5 — an unintended red. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log at
  `.floor/`. Copy the failing test's **entire** stdout+stderr **verbatim** — never a summary, never
  a `| head`/`| tail` window — and name the exact assertion or match arm that fired. **There is no
  such thing as a known flake.**

## Out of scope — affirmative cuts, not deferrals

- **`interpolate`'s partiality.** It still renders through `render_unquoted` (5 arms, raises on the
  6th) while `str` is total. Real, measured, and **the builder's call, not this stone's** — widening
  it changes what the `format` macro does with a non-scalar, and only `str` was ruled total. Change
  the comment; do not change the behaviour.
- **`Seqable` as a nameable type.** `join` stays over `Vector<T>`. `collection/infer.rs:638` records
  three named blockers; it is chain-D's other half and its own stone.
- **The `wat.string/*` namespace rename.** That is chain-E.
- **Making intrinsics first-class values.** Arc 255 — `255/NOTE-an-intrinsic-cannot-be-passed-as-a-value.md`.
  It does not arise here: the rendering happens in Rust, so no lambda is needed.
- **`join'`.** Not minted. There is one `join`.
