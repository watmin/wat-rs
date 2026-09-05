# BRIEF — the special-form sketch becomes syntax, and the wall arms at zero

Design: `[[DESIGN-STONE-the-special-form-sketch-is-syntax-not-a-hypervector]]` (this dir) and
`[[RULING-holon-is-for-vsa-only-and-a-wall-will-say-so]]` (`docs/arc/2026/06/294-holon-returns-to-vsa/`).
Read both first. Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C` for any git.

Four parts, **in this order** — each later part depends on the earlier ones being done.

---

## PART 1 — the special-form signature sketch stops being a hypervector

A special form's signature is `(:head <slot> <slot> …)` — a form. It is stored as a
`HolonAST::Bundle` and converted to a `WatAST` at every use. Store the `WatAST`.

**Rooms, read in order:**
- `src/special_forms.rs:40-80` — `SpecialFormDef` (the `signature: HolonAST` field), `sketch()`,
  and `insert()` which calls it. This is the origin.
- `src/special_forms.rs:140-170` — the test that destructures `def.signature` as
  `HolonAST::Bundle` and asserts `as_keyword()` / `as_symbol()` on its children.
- `src/reflect/lookup.rs:31` (`use holon::HolonAST;`), `:110-125` (`Binding::SpecialForm`'s
  `signature: HolonAST` field), `:255-270` (where it is cloned out of the def).
- `src/reflect/verbs.rs:215-250` — builds the SAME shape by hand from registry `entry.args`,
  then calls `holon_to_watast(&sketch)` to hand back a `Value::wat__WatAST`. The holon here is a
  pure intermediate: built only to be un-built.
- `src/holon/ast.rs:641-700` — `holon_to_watast`. Read its Symbol/Keyword/Bundle arms; they are
  what makes the replacement provably identical, and they are also where STOP-1 lives.

**Sketch of the change** (fill it in; do not invent a different shape):

```rust
// src/special_forms.rs
pub struct SpecialFormDef {
    pub name: String,
    pub signature: WatAST,
    pub doc_string: Option<String>,
}

fn sketch(head: &str, slots: &[&str]) -> WatAST {
    let mut children = Vec::with_capacity(1 + slots.len());
    children.push(WatAST::keyword(head));
    for s in slots {
        children.push(WatAST::symbol(crate::scope::Identifier::bare(*s)));
    }
    WatAST::List(children, crate::rust_caller_span!())
}
```

`WatAST::keyword(impl Into<String>)` and `WatAST::symbol(Identifier)` already exist
(`crates/wat-reader/src/ast.rs:~230`) and stamp `rust_caller_span!()` themselves.
`reflect/verbs.rs` gets the same treatment and its `holon_to_watast(&sketch)` call disappears —
the value it returns is already a `WatAST`.

Update `special_forms.rs`'s destructuring test to match on `WatAST::List` and assert on
`WatAST::Keyword` / `WatAST::Symbol` children. **Assert the SAME strings it asserts today**,
except for the leading colon — see STOP-1.

---

## PART 2 — `require_bundle` goes home

`src/runtime.rs:7386-7406` holds `require_bundle`, a VSA helper whose only two callers are
`src/intrinsic/holon/atom.rs:1463` and `:1514`. Move it there, beside them. Behaviour unchanged,
signature unchanged. Arc 109's own precedent sits three lines below it in `runtime.rs`:
*"`require_ast_children` moved to `src/reflect/verbs.rs` … Behaviour unchanged."* Leave the same
kind of one-line moved-to marker where it was.

Its error string reads `"Bundle (signature head HolonAST)"` — stale prose from when the sketch
shared it, and the thing that made this site read as misuse on first census. Correct it to name
what it actually guards (a `Bundle` argument to the holon verbs). Do not change the error KIND.

---

## PART 3 — one rune

`src/runtime.rs:~20125`, `arc143_slice5b_value_to_watast_accepts_holon_ast` builds
`HolonAST::symbol(":foo")` to exercise `value_to_watast`'s holon coercion arm. The holon IS the
subject under test, so it stays — with a co-located rune in the form Part 4's wall reads:

```rust
// rune:lint(holon-not-vsa, test-fixture) — <reason>
```

The reason must say why the holon is the subject, not that it is convenient.

---

## PART 4 — the wall

New file `tests/lint/holon_is_vsa_only.rs`. `tests/lint/mod.rs` auto-registers siblings via
`build.rs`, so dropping the file in is the whole registration.

**Copy the shape from `tests/lint/no_rc_use.rs`** — module doc explaining the failure it exists
to prevent, a recursive `.rs` collector, a `LazyLock<Regex>`, a rune escape, a per-file scope
list. That file is the worked reference; mirror its structure rather than inventing one.

**The rule, from the RULING** — aimed at the ACT, not the WORD:

> Outside the VSA homes and the one carrier, no module may **CONSTRUCT** a `HolonAST`
> (`HolonAST::<ctor>(…)`) or **DECLARE** one in a field or return type (`x: HolonAST`,
> `-> HolonAST`, `Arc<HolonAST>`, `Vec<HolonAST>`).
>
> **Pattern-matching is ALLOWED** (a match arm is downstream of a construction the wall already
> governs). **Prose is ALLOWED** — a `HolonAST` inside a string literal is documentation.

```
VSA HOMES        src/holon/**  ·  src/intrinsic/holon/**  ·  src/lower.rs
                 src/record/update.rs  ·  src/edn/render.rs
THE ONE CARRIER  src/value/value.rs
SCOPE            src/  and  crates/*/src/
```

Distinguishing a construction from a match arm is the real work of this lint. `HolonAST::Bundle(x)`
is a pattern when it sits in match-arm position and a construction when it does not; the
lowercase-initial constructors (`HolonAST::bundle`, `::keyword`, `::symbol`, `::i64`, `::string`,
`::nil`, `::bool_`, `::char_`, `::f64`) are unambiguous constructions, and the CamelCase variants
are the ambiguous ones. Pick a discrimination you can state in one sentence in the module doc, and
state honestly in that doc what it cannot see.

**Then PROVE the wall is not vacuous.** Add a `HolonAST::bundle(vec![])` to a non-home file, run
the lint, confirm it goes RED **naming that file and line**, then remove it. Report the red's text
verbatim. A gate never seen firing is a claim, not a gate (R59 `NISI FRANGAS, NIHIL PROBAS`).

---

## STOP TRIGGERS — each is a rejection: ship nothing, report, let me re-plan

**STOP-1 — the byte-identity precondition.** `holon_to_watast`'s Symbol arm special-cases two
strings: `s == "nil"` returns a `NilLit`, and `s.starts_with(':')` returns a `Keyword`. So the
direct `WatAST::symbol(...)` build is identical to today's output **only if no slot name is `"nil"`
and none begins with `':'`.** Enumerate EVERY slot string — the literals in `special_forms.rs`'s
`insert()` calls, and what `reflect/verbs.rs` can produce from `entry.args` — and confirm it.
If any slot violates it, **STOP**: the replacement is not identity and the design needs re-deciding.
Report the enumeration either way; it is the evidence for the identity claim.

**STOP-2 — the wall's census is the wall's own, never mine.** The RULING names four residue items.
That list is what I expect, not what is true. If your lint reports an offender outside those four,
**STOP and report it.** Do not add it to the homes list, do not rune it, do not adjust the regex to
miss it. A gate tuned until it agrees with the orchestrator's list is the orchestrator's list.

**STOP-3 — no behaviour rides along with the move.** If Part 2 cannot be a pure relocation —
if the signature, the error kind, or a visibility must change to make it compile — **STOP**.

**STOP-4 — a red is a red.** If anything you run goes red: do NOT re-run it. Copy the failing
test's whole stdout+stderr block verbatim, name the exact assertion that fired, and report. Do
not weaken an assertion to make it pass; a test of a retired behaviour becomes a negative witness
of the retirement, and rewriting one is my call.

---

## What you run, and what you do not

Cheap targeted checks are yours: `cargo build --release`, `target/release/wat --check <file>`,
and a scoped `cargo nextest run --release -E '<expr>'` — including your own new lint. **Do not run
the full floor**; I run it centrally, once, when the tree is quiescent. Do not commit, push, stash,
or revert. Do not spawn sub-agents.

You are a rider, not the orchestrator: **ending your turn ENDS you.** Run every verification in the
FOREGROUND and block on it — your turn ends when the numbers are in your hands, not when a command
is launched.

## Report

The slot enumeration from STOP-1 · the before/after of one rendered signature, verbatim (use
`(:wat::runtime::lookup-form :wat::core::defstruct)` or the nearest reflection verb that renders
one) · what your lint found, whether or not it matches the RULING's four · the sabotage red's text
verbatim · which files moved · anything that surprised you.
