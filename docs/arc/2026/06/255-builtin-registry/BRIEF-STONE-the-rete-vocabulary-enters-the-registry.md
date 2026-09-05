# BRIEF — the rete vocabulary enters the registry, all 37, and the two orphans with it

Design: `[[DESIGN-STONE-the-rete-vocabulary-enters-the-registry]]` (this dir). Read it first — it
carries the measured ground and why *restriction* was rejected in favour of plain aliases.
Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C` for any git read.

Three parts, **in this order**. Parts 1 and 2 clear the last two rows Part 3 needs.

---

## PART 1 — `:wat::core::cond` gets a declaration row

**Rooms:** `src/intrinsic/special/control_flow.rs:20-35` (`:wat::core::if`'s row — the axes to
copy, and the doc shape) · `src/intrinsic/special/defclause.rs:55-80` (a handler-less DECLARATION
row — the shape for a form with no runtime call site) · `wat/core.wat:1455` (`cond`'s defmacro —
read what it expands to) · `src/rete/purity.rs:1245-1258` (the checker ALREADY treats `cond`
clause-aware as chained `if`; your row must agree with it).

The axes, from `if`, unchanged:

```
@Purity Preserving · @Determinism Preserving · @Totality Preserving
@ExpandTime Legal  · @Category ControlFlow
```

The row carries **no handler** — `cond` is expanded by the macro system and has no runtime call
site, exactly like the 51 rows that already have none. Put it wherever the file layout says a
`:wat::core::` control-flow declaration belongs; `control_flow.rs` beside `if` is the obvious home
unless reading it says otherwise.

Write the doc comment the way `defclause.rs` writes its expand-time ground: say WHY each pole,
naming the mechanism. `Preserving` on three axes because a `cond`'s properties ARE its clauses'.

---

## PART 2 — `:wat::core::reduce`'s alias MOVES from wat to the registry

`wat/seq.wat` holds `(:wat::core::defalias :wat::core::reduce :wat::core::foldl)`. **Delete it**
and mint a registry alias row instead. Two authorities for one name is what the RULING forbids, so
this is a MOVE, never an addition.

The row declares `@alias :wat::core::foldl` and **no axes** — copy `:wat::rete::i64::>`'s shape
(`src/intrinsic/special/rete_alias.rs:83-102`), including its doc paragraph explaining that an
alias's axes ARE its target's and are resolved after folding.

**Rooms:** `src/intrinsic/special/rete_alias.rs:83-102` (the shape) · `wat/seq.wat` (find the
`defalias` and its surrounding commentary — several comment blocks describe `reduce`; they must
end up describing what is true after this change) · `src/collection/transform.rs:760-775`
(`foldl`'s axes — what will be inherited).

---

## PART 3 — the 37 rete rows become alias rows

`src/intrinsic/special/rete_alias.rs` is the home and already holds 37 such rows. Add one per
unregistered `RETE_OPS` entry, each declaring `@alias <its core_name>` and **no axes**.

Derive the list yourself — do not copy one from the design:

```bash
grep -oP '^\s+rete_name: "\K[^"]+' src/rete/vocabulary.rs | sort -u        # 74
grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ --include=*.rs | sort -u
# the unregistered ones are the difference; each row's core_name is in vocabulary.rs beside it
```

`@arg`/`@ret` come from the target's own row — the alias takes the same arguments and returns the
same answer. `:wat::rete::i64::>`'s `@ret` line shows the register: *"the target's answer,
unchanged."*

Both frozen ledgers in `src/intrinsic/mod.rs` (`REGISTRY_MEMBERSHIP_GAP_A`, `GAP_B`) will go red
naming exactly the names you registered. **Let them name the edit; do not pre-compute their new
contents.** Remove precisely the names the gate reports and no others.

---

## STOP TRIGGERS — each is a rejection: ship nothing, report, let me re-plan

**STOP-1 — the ledgers name the edit, never you.** If a ratchet reports a name you did NOT
register, or does not report one you did, STOP. That mismatch means the registration did
something other than what it looks like, and it is more interesting than the stone.

**STOP-2 — `reduce` must still WORK after its alias moves.** This is Part 2's real risk: a wat
`defalias` writes to `sym.functions` (door 3 of `head_ok`'s resolution order), and Phase 3a
(*resolve asks the registry*) has NOT shipped. `rete_alias.rs:83` claims a registry alias
dispatches through `alias_of` — but that is a claim about `:wat::rete::` rows, and you are moving
a `:wat::core::` name. **Prove it, do not assume it:** a probe that CALLS `(:wat::core::reduce …)`
and gets `foldl`'s answer, run against the built binary, and `--check` on a file that calls it. If
`reduce` becomes unresolvable, **STOP** — Part 2 then depends on Phase 3a and the design is wrong.
Parts 1 and 3 can still stand; say so and stop there.

**STOP-3 — no axes on an alias row.** If any alias row needs an axis restated to make a gate pass,
STOP. That is the *restriction* shape the design rejected on measured grounds, and it means the
measurement was wrong — which I want to hear, not to have papered over.

**STOP-4 — a red is a red.** Do NOT re-run. Copy the failing test's whole stdout+stderr block
verbatim, name the exact assertion that fired, report. Do not weaken an assertion to make it pass.

---

## What you run, and what you do not

Yours: `cargo build --release`, `target/release/wat --check <file>`, scoped
`cargo nextest run --release -E '<expr>'` — and the three census probes under
`wat-scripts/scratch-pad/255-b0-*.wat`, which are the instruments that measured this stone's
ground and will re-measure it. **Do not run the full floor**; I run that centrally, once, when the
tree is quiescent. Do not commit, push, stash, or revert. Do not spawn sub-agents.

You are a rider, not the orchestrator: **ending your turn ENDS you.** Run every verification in
the FOREGROUND and block on it — your turn ends when the numbers are in your hands.

## Report

The derived list of unregistered rete rows, and whether it was 37 · what the ledgers named, before
and after · STOP-2's proof that `reduce` still resolves and answers, with the probe output
verbatim · the `255-b0-*` probes re-run after the change (registered-row count, and what the 37
now inherit) · anything that surprised you.
