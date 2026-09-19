## CERNERE — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## SCOPE

Swept all 25 target files (20 Rust, 5 wat) for `:wat::rete::` phantom forms, checking three authorities:

1. **`RETE_OPS`** — read structurally from `src/rete/vocabulary.rs` (`rete_name: "…"` fields): **79 rows, not 108 as the cast handed me.** Re-derived with `grep -c 'rete_name: ":wat::rete::' src/rete/vocabulary.rs` → 79. This matches the test file's own measured comment (`tests/lint/rete_names_in_wat_scripts_resolve.rs:709`, *"Measured 2026-09-01: 79 `RETE_OPS` rows"*). **Delta: the handed 108 is wrong by 29; the correct count is 79.**
2. **`KNOWN_FORMS`** — one entry, `:wat::rete::core::defn`.
3. **Attestation** — every `:wat::rete::` token elsewhere under `src/` (194 distinct) and `wat/` (258 distinct).

Enumeration: extracted every distinct `:wat::rete::…` token in the 20 target `.rs` files (164 distinct) and separately in the 5 target `.wat` files (157 distinct), classified each into the `core::`/`holon::` registry namespace vs everything else, and diffed both buckets against their authority. Also extracted all 79 `(rete_name, core_name)` pairs from `RETE_OPS` and confirmed every distinct `core_name` (67 of them) is independently attested outside `vocabulary.rs` — **the registry itself contains no phantom rows.**

For the two-site finding below I additionally grepped the **entire repo** to confirm each name is unique to its own construction site — zero dispatch, zero row, zero wat-side call, zero doc attestation anywhere else.

## FINDINGS

### 1. `:wat::rete::exec_value` — `src/rete/expr_ir/eval.rs:59`

```rust
head: ":wat::rete::exec_value".into(),
reason: format!("slot {slot} is outside frame_len {}", frame.len()),
```

`exec_value` is the *Rust function's own name* (`pub(crate) fn exec_value` at line 69 of the same file) — an implementation detail, not a language form — surfaced as the `head` of a user-visible `MalformedForm` error. **Authority that should have declared it:** not a `RETE_OPS` row (wrong namespace anyway — not `core::`/`holon::`), and not attested as a dispatched op in `src/runtime.rs`'s dispatch table. Repo-wide grep returns exactly **one** hit — this line.

**The tell:** every real `:wat::rete::` name in this family is kebab-case (`exec-value` would be the spelling if it existed); `exec_value` is Rust `snake_case`, identical in shape to the confirmed target-1 phantom `:wat::rete::to_transient`.

**Sibling error sites in the same file use the real convention instead** — `head: ":wat::rete::core::match"` (line 771), `head: ":wat::core::string::subs"` (line 1110), or for a genuinely internal failure, a **plain, non-namespaced** string: `head: "compiled-exec"` (line 1295). This site should follow one of those two patterns, not invent a third that looks like a real wat form.

**Recommendation:** a plain non-namespaced diagnostic string (matching the `"compiled-exec"` precedent) — this is an internal slot-arithmetic bug surface, not a wat-visible form.

### 2. `:wat::rete::apply_op` — `src/rete/expr_ir/eval.rs:905`

```rust
head: ":wat::rete::apply_op".into(),
reason: format!("op index {op} is outside RETE_OPS"),
```

Identical shape: `apply_op` is the Rust function's own name (`pub(crate) fn apply_op` at line 890, same file), snake_case, leaked into a `:wat::rete::`-namespaced error head. Repo-wide grep returns exactly **one** hit. Not a `RETE_OPS` row, not dispatched, not attested anywhere else.

Both sites are internal-consistency guards (an out-of-range op index / slot index — a bug in `Program` construction, never reachable from well-typed user `where`/`:then` code), so the fix is cosmetic to the error label, not a behaviour change.

### 3. (weaker, secondary) `:wat::rete::core::vector::=` — `src/rete/clause.rs:551`

Inside `unrelated_heads_are_not_constraints`, a list of heads that must NOT classify as constraints. `classify_constraint_head` (`:157-186`) only recognises per-type equality for `i64|f64|string|bool|keyword|enum` — `vector` was never an admitted type, and no `RETE_OPS` row named `…vector::=` has ever existed (checked the full 79-row extraction). Repo-wide grep confirms this string appears **nowhere else**. It reads as a phantom FQDN — a plausible-looking name invented for this test rather than drawn from the spec — but functionally it is used correctly as a *negative probe*, which is the shape cernere's `spell-probe` rune category exists for. It carries no rune. **Lower confidence** than findings 1–2 because it is test-only data, never surfaced to a user, and the surrounding test's intent is legible without one — flagged mainly because it satisfies the letter of *"a name that looks valid but traces to no authority."*

## Prior-art / not re-reported

Confirmed still holding, not re-rowed: `factbag::count-of` dead-not-phantom, `AxisViolation.axis/span` unread fields, all ~130 oracle-side names, `clause.rs` runes, `matcher.rs` `Option`, `vocabulary.rs` stale prose counts, `purity.rs:1811`, `export.rs` pack/unpack, `compiled_cond.rs` fixture gap, `where_tree.rs` items. `:wat::rete::lower`, `:wat::rete::call-user`, `:wat::rete::cond-has-deferred-constraint?`, `:wat::rete::explain::constraint-not-rendered`, and all session/node-type names (`AlphaNode`, `Session`, `Rule`, etc.) were checked and are genuinely dispatched/attested — **not phantoms**.

**FINDINGS**
