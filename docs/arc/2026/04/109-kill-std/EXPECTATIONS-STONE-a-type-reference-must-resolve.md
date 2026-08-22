# EXPECTATIONS — a type reference must RESOLVE

Written BEFORE the strike, against `4ff967b98`. Brief:
`BRIEF-STONE-a-type-reference-must-resolve.md`. DESIGN ruled D1-A · D2-A · D3-B.

## The scorecard

| # | what | my command | expected |
|---|---|---|---|
| 1★ | phantom in an UNCALLED declaration is rejected | `--check` on the fixture | EXIT 1, names `:user::NoSuchType` at the DECLARATION |
| 2★★ | phantom WITH a caller names the TYPE, not the caller | `--check` on the fixture | EXIT 1, no `TypeMismatch` blaming parameter #1 |
| 3 | phantom in a return slot | `--check` | EXIT 1, names the type |
| 4 | phantom as a parametric form | `--check` | EXIT 1 |
| 5 | phantom in a record field | `--check` | EXIT 1 |
| 6✅ | forward references still legal | the two-file control pair | real value EXIT 0 · `i64` EXIT 1 naming `:user::Later` |
| 7✅ | type variables still legal at scale | `scripts/floor.sh` | **4859/4859 + N new**, 0 FAIL |
| 8 | tests live under `tests/`, none under `wat-scripts/` | `git status --short` | nothing new under `wat-scripts/` |
| 9 | call-head exemption untouched | `git diff src/resolve/walk.rs` | `is_resolvable_call_head`'s reserved-prefix arm unchanged |
| 10 | clippy | `--workspace --all-targets --release -- -D warnings` | 0 |

Row 7's count is arithmetic, not observation: 4859 + however many tests rows 1-5 add. **A count that
lands anywhere else gets EXPLAINED before it is accepted.**

## The rows that can lie, and how

**Row 1 can pass while the stone is half-built.** The uncalled case has no check error, so
`(Ok, Some(resolve_err))` fires and the new diagnostic surfaces. The CALLED case goes through
`freeze.rs:1308`, where the check error wins unless every check error is an `UnknownCallee` — and a
`TypeMismatch` is not one. **So row 1 green + row 2 red is the predicted outcome of a correct pass
with the precedence untouched**, and it reads exactly like success. Row 2 is the only row that
distinguishes them.

**Row 7 is the only row that tests the `type_params` scoping at all.** Rows 1-5 are all phantoms;
none of them contains a type variable. A pass that treats every `Path` as a name to resolve passes
rows 1-5 perfectly and lights up `wat/` in the hundreds. If row 7 is green and rows 1-5 are green,
the scoping works; either alone proves nothing about it.

**Row 6 catches the rejected option.** If the rider validates at registration time (D1-D), forward
references break. Its own control matters: exit 0 alone is meaningless while type names go
unresolved, so the pair must be run — the real value passing AND the `i64` failing by name.

**Row 9 exists because the two halves look alike.** The reserved-prefix exemption is correct for CALL
heads and wrong for types; a rider tidying "the same rule in two places" would delete an earned
exemption. It must be untouched.

## Independent prediction

**Runtime: 60-110 minutes.** Longer than any stone this session. The sweep itself is mechanical, but
three things carry real discovery cost: the `TypeExpr` recursion has five variants and the
`Parametric.head` is a name that must be checked as well as its args; STOP-1 requires reading the
checker's scope construction rather than probing it; and STOP-2 may require touching a precedence rule
that three named contracts pin.

**Trap-doors named in advance:**
- **Row 2 quietly dropped.** The likeliest failure. The rider ships row 1, reports success, and the
  called case keeps its old message. My scorecard treats row 2 as the stone; a report without its
  verbatim output is Mode B regardless of what else passed.
- **The bound set applied at the wrong level** — e.g. one global set of every `type_params` seen,
  rather than per-declaration. That passes rows 1-7 and silently accepts `T` in a monomorphic
  declaration that never bound it. I check this by adding a probe of my own after the fact: a
  non-parametric `defn` naming `:T` must be REJECTED.
- **A stdlib violation "fixed" to make the pass green.** STOP-3. A real unresolvable type in `wat/`
  outranks this stone; renaming it to get a green floor would bury the most valuable thing the sweep
  can produce.
- **`Parametric.head` unchecked.** Walking only `args` passes row 4 if the args are concrete. My
  fixture for row 4 uses a phantom HEAD with a legitimate arg for exactly this reason.

## Mode

- **Mode A** — all ten rows, row 2 with verbatim output, STOP-1 answered by reading rather than
  guessing, any stdlib violations reported before being touched.
- **Mode B** — ships, but row 2 unproven or weakened, or the precedence changed without accounting for
  the three contracts it carries.
- **Mode C** — a STOP fires. Ship nothing; the report is the deliverable. STOP-1 and STOP-3 firing
  are GOOD outcomes — they are findings this stone exists to surface.
