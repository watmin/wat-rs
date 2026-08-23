# BRIEF — the callable turbofish dies too, and its probe becomes a `.wat.bad`

Builder's ruling: *"i say the feature to enforce correct is imposed and this test becomes a
`.wat.bad` proving its correct."*

③ made angle-bracket **types** illegal. This closes the last island: `:ns::Type/method<A,B>` — the
explicit type-application spelling at a **call site**.

## Why it is illegal, and it is not about the brackets

**`,` is whitespace in EDN, and in wat.** Measured on the current build:

```
(:wat::core::Vector :- [:wat::core::i64] 1, 2, 3)   →  [1 2 3]   EXIT 0
```

`Head<K,V>` is the *only* construct in the language that gives a comma meaning — its parser splits on
it. That is the core EDN violation, and the turbofish inherits it whole. The angles were the symptom.

## The population is ONE

```
tests/types/probe_arc232_generic_method_type_application.wat:13
    [b (:user::Mk/mk<wat::core::i64,wat::core::i64> (:wat::spawn::thread))]
```

One occurrence in 1532 `.wat` files, and it is the probe **for the feature itself**. Nothing in the
stdlib, nothing in `wat-scripts/`, no service. The seam records call-site type application as
*"REJECTED today, site count still UNMEASURED"* — it is measured now, and it is 1.

## The work

**Refuse a `<` in a CALLABLE name** at the resolution door, the way ③ refuses it in a type.
`runtime::canonical_callable_name` (`src/runtime.rs:4256`) currently STRIPS it:

```rust
pub fn canonical_callable_name(kw: &str) -> &str {
    if !kw.ends_with('>') { return kw; }
    match kw.find('<') { Some(i) => &kw[..i], None => kw }
}
```

⛔ **STOP-1 — measure before you touch it.** That function has ~8 callers and exists to strip a
suffix. β-ii-b dropped `{p}` from 18 generated FUNCTION names, so the strip may now be **vacuous** —
nothing may generate an angle callable any more. Determine which is true **first**:

- **If the strip is vacuous** — no generated name reaches it with a `<` — then the honest change is
  to make it refuse (or delete it and let the lookup fail naturally), and the feature simply dies.
- **If some generated name still carries `<`** — report WHICH, with the generator's file:line, and
  STOP. That is a different stone: the generator must stop emitting it before the door can refuse.

Do not guess. Instrument it if you must — a `debug_assert!` on the strip path, run the floor in
DEBUG, and see whether it ever fires. (A `debug_assert` panic IS a real failure here; that is the
point of using one.)

## The probe becomes a negative fixture

`tests/types/probe_arc232_generic_method_type_application.{wat,rs}` currently asserts the turbofish
WORKS. Convert it:

- fixture → `probe_arc232_generic_method_type_application.wat.bad` (the `.bad` suffix keeps it out of
  the loader gate — `tests/services/*.wat.bad` is the established precedent)
- driver → assert `startup_from_file` returns `Err`, and that the diagnostic **names the offending
  keyword** and points at the legal spelling. Copy `tests/macros/probe_arc279_format.rs`'s shape:
  `is_err()` + `wat::assert_edn_matches_file!` against a captured golden.

★ **That test is the deliverable.** It is the negative control proving the feature is gone, and it is
the reason this stone exists rather than a one-line deletion. `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`

## Acceptance

| # | what | expected |
|---|---|---|
| 1★ | STOP-1 answered | say plainly whether the strip is reachable, with the evidence |
| 2★★ | the turbofish is refused | the `.wat.bad` fixture → `startup_from_file` is `Err`, naming the keyword |
| 3★★ | a legal call still works | the same probe's *non*-turbofish call path still resolves — the refusal did not eat ordinary method dispatch |
| 4 | the floor | `scripts/floor.sh` green |
| 5 | clippy | 0 under `-D warnings` |

**Row 3 decides it.** Row 2 goes green for a door that refuses *every* callable. Only an ordinary
`:ns::Type/method` call still dispatching proves you refused the turbofish and not method dispatch
itself.

## Boundaries

- `src/runtime.rs` (the one function and whatever STOP-1 finds), and the one test pair.
- Do NOT hand-edit any other `.wat`. There is exactly one site.
- Do NOT commit, push, stash or amend. Keep the index EMPTY — no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- `scripts/floor.sh` is allowed; it is row 4.

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

STOP-1's answer with its evidence, first. Rows 2 and 3 verbatim, together. The golden's content. The
floor's Summary line. Whether `canonical_callable_name` survived, died, or changed shape. What
surprised you.
