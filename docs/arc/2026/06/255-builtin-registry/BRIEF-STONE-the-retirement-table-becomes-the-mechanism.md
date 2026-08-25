# BRIEF — STONE: the retirement table becomes the mechanism it looks like

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-retirement-table-becomes-the-mechanism.md`
CENSUS: `docs/arc/2026/06/255-builtin-registry/NOTE-the-retirement-table-is-inert-for-half-its-rows.md`
Read both whole before you touch anything.

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first and stay there.

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Every command in the
FOREGROUND, blocking. Your turn ends when the numbers are in your hands.

**You may not spawn sub-agents.** Do not commit, push, stash, revert, or `git checkout`. There is a
`git stash@{0}` that must never be touched.

You may run `cargo build --release` and single named tests (`cargo nextest run --release -E
'test(<name>)'`) — you are writing a gate and cannot do that blind. **Not** the full floor, **not**
`cargo clippy`; the orchestrator runs those centrally.

---

## The work in one paragraph

`src/remedy/retirement.rs`'s `RETIREMENT_TABLE` looks like a lookup the substrate performs. It is
not — it is a lookup that **thirteen hand-written arms** perform, and the table is the data they
happen to share. Thirteen of its 35 rows produce a bare `UnknownFunction` with no help at all.
Consult the table at the two doors where an unknown name actually surfaces, and put a gate over the
**table** so the next row cannot be inert.

---

## ★ DO THIS FIRST — write the gate, and watch it name your worklist

Before either fix. It goes **red on thirteen rows today** and prints every one; that list is the
work. `docs/SUBSTRATE-AS-TEACHER.md` is the method and this is a clean instance of it.

**Three things about the gate are load-bearing:**

1. **It iterates `RETIREMENT_TABLE` itself.** Not a hand-list of names, not a count. If the gate
   holds its own copy of what to check, it is the same defect one level up —
   `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]` is exactly what this table is.
2. **It drives the real binary, end to end.** `tests/cli/wat_grep.rs` is the pattern
   (`Command::new(env!("CARGO_BIN_EXE_wat"))`). An in-process `check_program` call would report the
   thirteen GREEN — they pass the checker silently today and fail only at runtime. A check-only
   gate is a gate that cannot see the bug.
3. **Its assertion is the NEGATIVE, so it needs no exemption list.** For each row: the outcome must
   not be *a bare `UnknownFunction` with no replacement named*. A retirement message passes. A
   `TypeMismatch` that names the replacement passes. That admits the seven `vec`/`list`/`tuple`/
   `Some`/`Ok`/`Err`/`:None` rows without special-casing them — they are already diagnosed by a
   third path and are **not** your work.

Report the gate's red list before you fix anything. If it is not exactly thirteen, that difference
is more interesting than the fix and I want it first.

---

## DOOR 1 — check time, and it is the primary one

`src/check.rs:5628` — the silent-accept fallback, whose own comment reads *"HARVEST (236.2):
silent-by-intent — no scheme found for multi-arg form; accept and pass."*

Consult `retirement_lookup(k)` before accepting. A hit becomes a located `MalformedForm` carrying
`crate::remedy::remedies_for(k, std::iter::empty())` — **the same shape the working thirteen already
produce.** Copy one of them; `check.rs:955`'s `:wat::core::Char` arm is the clearest, and
`check.rs:4742` shows the `remedies_for` call in an `infer_list` arm.

⚠ The working thirteen must come out **byte-identical**. If door 1 makes one of them report twice or
report differently, that is a regression, not a nicety — acceptance row 5.

## DOOR 2 — runtime, and it does NOT widen the type

The `RuntimeErrorKind::UnknownFunction` construction sites. A dynamically built head (`eval-ast!`,
`keyword/from-string`) never passes the checker, so door 1 alone leaves a hole.

**`RuntimeErrorKind::UnknownFunction(String)` is a tuple variant carrying only the path**
(`src/value/signal.rs:195`). Fold the replacement into the message text:

```
unknown function: :wat::core::Uuid/v4 — ':wat::core::Uuid/v4' is retired; use ':wat::uuid::v4' instead
```

⛔ **Do NOT widen the variant to carry a structured `:remedies` list.** That is a question about the
error type's shape, it drags every `UnknownFunction` call site with it, and it is affirmatively out
of scope. Door 1 delivers the structured remedy; door 2 delivers the sentence. STOP-2 if you find
yourself editing `signal.rs`'s enum.

---

## Blast radius

`src/check.rs`, the `UnknownFunction` sites, and the new gate plus any fixture it needs. **No new
entries in `RETIREMENT_TABLE`** — it already has all 35 and this stone does not add names. **No
per-name arms** — if you are writing `if s == ":wat::core::Uuid/v4"`, stop and read STOP-1.

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — you are about to add a per-name arm.** The entire point is that the table stops needing
   them. If a row cannot be reached generically, report which and why; do not hand-write it.
2. **STOP-2 — the fix seems to need `RuntimeErrorKind::UnknownFunction` to carry remedies.** It does
   not; door 2 is a message change. Report what demanded it.
3. **STOP-3 — a working row's output changes.** The thirteen that already produce a retirement
   message must be byte-identical. Report the row and both outputs.
4. **STOP-4 — the gate's red list is not exactly the thirteen from the census.** Report the actual
   list first. A different number means the census moved and that is the finding.
5. **STOP-5 — a room's line number does not hold what this brief says.** Written against
   `ef8e8a3e4`. Report the mismatch rather than widening the search.

---

## Acceptance you can check yourself

```bash
# BEFORE the fix — the gate must be RED, naming 13 rows
cargo nextest run --release -E 'test(<your gate name>)'

# AFTER — green on all 35, and this must name :wat::uuid::v4
cat > /tmp/r.wat <<'W'
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::Uuid/v4)))
W
./target/release/wat /tmp/r.wat

# the three that were inert BEFORE the four-homes stone — they are the proof
# this is a substrate fix rather than ten more arms
#   :wat::core::Record::def · :wat::core::to-struct · :wat::holon::Record::def
```

## Report back with

- **The gate's RED list, before any fix** — every row it names. First thing in your report.
- The same gate green afterwards, with the row count.
- `(:wat::core::Uuid/v4)`'s message, verbatim, before and after.
- The three pre-existing inert rows' messages after the fix.
- **Proof the working thirteen are unchanged** — pick two, show their output is identical.
- Anything the brief got wrong. It was written by someone who has been wrong about this corpus
  repeatedly today, including about this very table.
- What you did NOT do, and why.
