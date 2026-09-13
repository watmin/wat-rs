# BRIEF — the malformed census

**Read `DESIGN.md` beside this first.** It carries why this is a `wat-scripts/` program and **not** a
`wat/lint.wat` rule, the two classification axes, and the two-sided control that must pass before any count is
quoted.

## The work, in one paragraph

`RecvOutcome::Malformed` appears **487** times; the record calls ~61 of them "live placeholders" and nobody can
apply the builder's *"service items only"* filter without opening files. Write a **report-only census program**
that walks the form trees of every `.wat`, finds each `Malformed` arm whose body **raises** (three forms), and
emits a row carrying **the enclosing top-level form's head** and **the file's home** — so the filter is a
column. It never writes.

## The rooms

| where | why you are going there |
|---|---|
| `wat/fix.wat:23–52` + any `wat-scripts/fixes/*.wat` | ⭑ **THE PROVEN SHAPE.** Each codemod has a `:user::grep` **finder** entry point that reports matches without applying, driven by `wat --grep <file>.wat` with the path list on **stdin**. You are building that with **no apply half.** |
| `wat-scripts/fixes/bind-deleted-on-delete-response-success.wat` | this session's worked example: `node-edits` gated on a head, `match-edits` folding over `(drop ch 2)` — i.e. **arms**, and `arm-pattern-edits` keying on an arm's first child. That is exactly the walk you need, minus the edits. |
| `wat/deporder.wat:126–145` | `collect-kwds` — a recursive form walk collecting qualified keywords, with `structural?` for recursion. The other worked walker. |
| `wat/lint.wat:50–57` | `Finding [rule file line col severity message fix]`. ⚠ Reuse it **only** if convenient; `fix` must stay `None`. You may equally emit your own record — you are not editing `wat/lint.wat`. |
| `wat-scripts/fanout/circuit.wat:538–543` | ⭑ **CONTROL A — must NOT be flagged.** The poisoned-call arms, migrated at `15ddc5e35`: `((…Malformed _cause) "malformed")` returns a value. ⚠ **The same physical line also holds a `Stopped` arm that DOES raise** — which is why a line-oriented instrument cannot do this job. |
| `wat-scripts/topic/sns-fanout.wat` (topic-worker `-disrupt`) | CONTROL A's twin, same migration, same line-sharing hazard. |
| any `wat/` arm carrying `UNMIGRATED PLACEHOLDER` | ⭑ **CONTROL B — must BE flagged.** A known raiser. |

## What each row must carry

```
file : line : col
enclosing-top-level-head    defservice | defsurface | defn | deftest | other
home                        wat | wat-scripts/service | wat-scripts/scratch-pad | tests | docs
raises?                     which form — assertion-failed! | raise! | panic!
```

★ `enclosing-top-level-head` is the column the builder's *"service items only"* filter reads. `home` is how
probes and fixtures — **where raising is correct** — are separated at a glance. A row missing either is a row
that still costs a file-open.

## ⛔ Three things that will make the census wrong

1. **A line-oriented pass.** Several arms share one physical line. **Walk forms.** Verified: my own grep for the
   migrated form returned lines that also contain `assertion-failed!` from a neighbouring arm.
2. **Keying on `assertion-failed!` only.** Raising is three forms — `assertion-failed!` (3654), `raise!` (10),
   `panic!` (1). Missing the last two loses 11 sites.
3. **Quoting a count before the controls pass.** Control A must be **absent** from the output and control B
   **present**. State both in the report.

## Verify

- ⭑ **The two-sided control, printed in the report itself**: control A absent, control B present.
- **Explain the gap between 487 token occurrences and the finding count** — the difference is non-raising arms
  plus non-arm mentions, and a reader will ask.
- `./scripts/floor.sh`, read the **Summary line** → green at **5239**. Your program lives under
  `wat-scripts/`, so `every_wat_scripts_file_loads` type-checks it; a rotted census reddens the floor.
- ⛔ **Never a piped exit code** (a type-error run this session reported `$?` = 0 through `| head`; true exit 3).
- `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- `git diff --stat -- src/ wat/` must be **EMPTY** — no stdlib, no Rust.
- Idempotent by nature (it writes nothing), but **run it twice and show identical output**. ⚠ **Generate the
  path list SORTED** (`... | sort`) — an unsorted glob makes row order filesystem-dependent, which would red
  row 12 on a perfectly correct census. Generate the list, never hand-type it.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — migrate nothing.** Not one arm, however obvious. If you find a row whose cure seems certain,
   put it in the report with your reasoning; the builder sequences.
2. **STOP-2 — do not add a rule to `wat/lint.wat`** or otherwise touch stdlib. DESIGN explains why a permanent
   gate firing ~59 times is noise rather than a gate.
3. **STOP-3 — if either control fails, STOP and fix the instrument before reporting any count.** A census
   quoted from an unvalidated pass is the defect this campaign has hit five times.
4. **STOP-4 — do not resolve "service item" for ambiguous rows.** Report both axes and mark the row ambiguous.
   That classification is the builder's.
5. **STOP-5 — if walking forms cannot reach an arm's body** (e.g. the body is a macro call that expands to a
   raise), STOP and report the shape. Guessing whether a macro raises would put a wrong row in a census whose
   whole value is trust.

## Shape to copy

`wat-scripts/fixes/bind-deleted-on-delete-response-success.wat` for the arm walk, and
`the-crash-surface-is-enumerated/FINDING-the-crash-surface.md` for how a census reports its own instrument's
corrections and controls.
