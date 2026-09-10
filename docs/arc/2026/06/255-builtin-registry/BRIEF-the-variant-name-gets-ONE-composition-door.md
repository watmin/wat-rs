# BRIEF — a variant's name gets ONE composition door

**Read `DESIGN-the-dot-flip-is-a-COMPOSITION-problem.md` first.** This is steps ① and ② of its
order: **mint the door, and impose it.** ⛔ **The separator does NOT change in this stone** — the
flip is step ③ and it lands only once the door exists.

## The work

`crates/wat-reader/src/identifier.rs` — the one-name grammar — has TEN decomposition accessors
(`leaf` · `path` · `receiver` · `method` · `prime` · `deprimed` · `namespace` · `is_reference`) and
exactly one constructor (`bare`). **Every accessor SPLITS a name; nothing COMPOSES one.** So a
variant's name is built by hand at **15 sites in 2 spellings**, and one of the fifteen already
disagrees with the other fourteen.

Mint one composition function beside the decomposers; point all fifteen sites at it.

## The fifteen — all measured, all `(enum_path, variant_name)`

```
src/runtime.rs:4108   ⚠ "{}::{}", e.type_path.trim_start_matches(':'), e.variant_name
src/runtime.rs:8628 · 8828 · 8968      "{}::{}", ev.type_path, ev.variant_name
src/closure_extract.rs:2251            "{}::{}", ev.type_path, ev.variant_name
src/rete/expr_ir.rs:1320               "{}::{}", e.type_path, e.variant_name
src/declare/register.rs:1341 · 1370    "{}::{}", enum_def.name, variant_name
src/declare/preregister.rs:314         "{}::{}", type_name, variant_name
src/value/observe.rs:363               "{}::{}", ev.type_path, ev.variant_name
src/types.rs:709 · 1028                "{}::{}", name, variant_name
src/check.rs:1870                      "{}::{}", enum_path, variant_name
src/edn/render.rs:3918 · 4628          "{enum_leaf}.{variant_name}"    ← the RENDER spelling
```

## ⛔⛔ THE HAZARD YOU WILL HIT FIRST — and it is DOCUMENTED, DELIBERATE, and OPPOSITE

The two value types store the same kind of name in two shapes, each saying so in its own doc:

```
EnumValue.type_path    (src/value/value.rs:1151)
    "matches the enum's declared name verbatim (`:trading::types::PhaseLabel`)"   WITH a colon
AggregateValue.class   (src/value/value.rs:1009)
    "Colon-free FQDN of the declared type (e.g. `"wat::kernel::Process"`)"        WITHOUT one
```

That is why `runtime.rs:4108` trims and the other twelve do not — and why its sibling arm, in the
SAME `match`, does `format!(":{}", a.class)` to ADD one. **Two arms of one function, opposite
shapes.**

⛔ **DO NOT unify the storage.** Both conventions are documented and load-bearing; changing either
is a different stone with a much larger blast radius. **The door must state its precondition and
hold it.** Decide explicitly — does it take the enum path AS STORED and normalize internally, or
does it require one shape and leave callers to present it? Either is defensible; **say which, in the
function's doc, and make all fifteen call sites obey it.** An undocumented assumption here just
recreates the drift one level up.

★ `runtime.rs:4108`'s trim is then either RIGHT FOR EVERYONE (the door normalizes, and twelve sites
were quietly wrong) or WRONG FOR IT (the door requires the stored shape, and 4108 was compensating
for something else). **Measure which, and say so.** Do not preserve the trim by reflex.

## The door

One function, beside the decomposers it inverts, in `crates/wat-reader/src/identifier.rs`. It takes
an enum path and a variant name and returns the composed name. Its doc must state:

- the precondition (above), and why;
- that it is the INVERSE of the decomposition accessors already in that file;
- that the separator is **this function's decision, made once** — and that the dot flip is a change
  to this body and nothing else.

⚠ Its two spellings are still TWO functions or one function with an explicit mode — the render
sites compose `Enum.Variant` from a LEAF (`enum_leaf`), not from a path. **Do not force them into
one signature by flattening that difference**; if they genuinely need two doors, mint two and say
why in both docs. What must not survive is fifteen hand-rolled `format!`s.

## Blast radius

`crates/wat-reader/src/identifier.rs` (the door) + the 15 call sites listed above.
**No behaviour change of any kind.** Every composed string must be byte-identical to what it is
today — this stone moves WHERE a name is built, never WHAT is built.

## Acceptance

```
cargo build --release
cargo nextest run --release -E 'test(enum) or test(variant)'          unchanged
cargo nextest run --release -E 'test(probe_arc296)'                   unchanged
target/release/wat <a program constructing a variant>                 byte-identical output
```

★ **Nothing should change observably.** If any test's expected value moves, that is a FINDING that
two of the fifteen were composing different strings — report it verbatim, do not update the
expectation.

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If you change the separator anywhere — STOP. That is step ③ and it is not this stone.

**STOP-2.** If you change `EnumValue.type_path` or `AggregateValue.class`'s storage convention —
STOP. Both are documented and deliberate; unifying them is its own stone.

**STOP-3.** If any test's expected output changes — STOP with the verbatim diff. This stone is
behaviour-preserving by construction, so a moved expectation means two sites disagreed and you have
found the drift the door exists to end.

**STOP-4.** If a call site cannot use the door without a cast, a strip, or a re-add — STOP and name
it. That site is telling you the precondition is wrong, and it is the most valuable thing you can
report.

**STOP-5.** If you find a SIXTEENTH composition site — STOP and report it. My census read all
thirteen `::` sites and both `.` sites; a sixteenth means the census pattern missed a spelling.

## Tier

You edit and report. Run the acceptance commands, foreground, and nothing else. **Do NOT run
`scripts/floor.sh` or `cargo clippy`** — the orchestrator runs those centrally, once, on a quiescent
tree. **Do NOT commit.**

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.
