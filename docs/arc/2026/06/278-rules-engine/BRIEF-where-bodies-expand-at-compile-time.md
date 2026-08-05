# BRIEF — a `where` body is CODE; teach the expander that, once, in the one place

**Builder's ruling, 2026-08-05:** *"we should just expand where bodies at compile time."*

Full grounding and the reasoning behind every choice below:
**`DESIGN-STONE-where-bodies-expand-at-compile-time.md`** (task #78). Read it first — it is short,
and it records two routes that are already closed so you do not re-derive them.

Anchor `/home/watmin/work/holon/wat-rs/`; verify with `pwd`. Baseline at draw time, **my own
`--release` re-runs at `5851a316`**:

```
floor    4356 tests run: 4356 passed, 262 skipped
clippy   clean
gate     9 pair(s), 98 rows — wat == Clara on every shape
```

---

## THE WORK — one classification, consulted by three passes

`src/resolve/boundary.rs` is **the single source of truth** for "which of this head's arguments are
code and which are data." Its header records why: `walk`, `normalize` and `expand_form` each had a
hand-rolled `if`-chain, and **the chains drifted** (arc 251.1 ward).

Add a variant. The classification, stated exactly:

> **`:wat::rete::make-rule`** — `items[1]` (the rule name) is ordinary **code**. `items[2]` (the
> quoted `:when` vector) is **DATA, EXCEPT the BODY of each `(:wat::rete::where …)` form inside it,
> which is CODE**. `items[3]` (the quoted `:then` vector) is **data**.

`Boundary::MatchesSubject` is the working exemplar in all three passes — copy its shape.
`expand_form` consults it at `src/macros/expand.rs:458`; that is the site to mirror.

### ⛔ STOP-1 — the hook is `make-rule`, NOT `defrule`

Measured. Four producers quote a when-vec:

| producer | site |
|---|---|
| `defrule`'s template | `wat/rete.wat:2314` |
| **`sift-rules-defsvc`** | **`wat/query.wat:189`** |
| a hand-built rule literal | `wat-scripts/scratch-pad/probe-rule-lits.wat:33` |
| **direct** `make-rule` calls | `wat-scripts/scratch-pad/probe-sift-body-direct.wat:14,17` |

`defrule` is the sugar. Hooking it silently misses the sift engine's own generator. If you find a
**fifth** producer, that is a finding — report it, do not quietly widen.

### ⛔⛔ STOP-2 — expand the `where` BODY ONLY. This one has already bitten.

A condition vector holds fact patterns — `(:probe::Req (?a <- :a))` — whose heads are
**aggregate-shaped**. Post arc-294 item 9a's construction flip, an aggregate name **is a registered
kwargs companion macro**. Walk a pattern as code and `kwargs-lower` fires on raw DSL clauses as if
they were kv-pairs.

Not hypothetical: that is verbatim why `MatchesSubject` exists. Read
`src/macros/expand.rs:445-455` — it documents the identical failure for `matches?` patterns, in
those words.

**Only the body of a `(:wat::rete::where …)` form is code. Everything else in the vector stays
byte-identical.**

### ⛔ STOP-3 — `walk` and `normalize` match `Boundary` EXHAUSTIVELY

Adding a variant is a compile error in both until handled. That is the point (the substrate hands
back the worklist). Handle each deliberately — for both passes a `where` body genuinely IS code, so
the same classification should serve all three. **If either pass wants a different answer, STOP and
report** — a region that is data for one pass and code for another is a real finding.

### ⛔ STOP-4 — no fourth encoding

The classification goes in `boundary.rs`. A fresh `if head == ":wat::rete::make-rule"` anywhere
else is the exact defect that module was built to kill.

---

## ★ THE ACCEPTANCE TEST — two probes must FLIP

Both are on disk and both are **RED today, by design**:

- `wat-scripts/scratch-pad/probe-cond-rete-where.wat` — `(:wat::rete::core::cond …)` in a `where`
- `wat-scripts/scratch-pad/probe-cond-in-where-baseline.wat` — the core spelling, same shape

Today both raise `#wat.runtime/UnknownFunction`. **After this stone both must print a real hit
count.** Update each probe's header comment: they stop being gap-witnesses and become positive
controls. Say so in the file, and say what they measured before.

The control that must NOT move: `wat-scripts/scratch-pad/probe-rete-if-in-where.wat` → `hits=1`.

---

## ⛔ STOPs (the rest)

- **⛔** No `_` wildcard arm on an enum scrutinee — `Boundary` is exhaustive by design.
- **⛔** Do not touch the `:then` vector. The RHS is a separate question (task #61 already ruled
  derived fact fields are copies only).
- **⛔** Do not arm law A's third conjunct. That is #57 and it comes after this.
- **⛔** Do not commit, stash, push, or touch git.
- **⛔** Every verification runs in the FOREGROUND and blocks. Your turn ends when the numbers are
  in your hands, not when a command is launched.

## Verify — FOREGROUND, blocking

```
cargo build --release
./target/release/wat wat-scripts/scratch-pad/probe-cond-rete-where.wat        # must FLIP to a hit count
./target/release/wat wat-scripts/scratch-pad/probe-cond-in-where-baseline.wat # must FLIP
./target/release/wat wat-scripts/scratch-pad/probe-rete-if-in-where.wat       # must stay hits=1
cargo nextest run --release
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Read the **Summary line**, never a piped exit code. Strip ANSI before matching it
(`sed 's/\x1b\[[0-9;]*m//g'`) — a coloured `Summary` defeats a naive grep and has bitten here twice.

## EXPECTATIONS

| # | what | expected |
|---|---|---|
| 1 ★★ | **the two gap-probes FLIP** | a real hit count, both spellings |
| 2 ★★ | **the `where` corpus does not move** | `9 pair(s), 98 rows — wat == Clara on every shape`. **A single moved derived fact is the alarm**, not a nit |
| 3 ★★ | **condition patterns untouched** | no `kwargs-lower` error anywhere; the 38 `.wat` files holding a `where` all still load |
| 4 ★ | the `if`-in-`where` control | still `hits=1` |
| 5 ★ | the classification is in `boundary.rs` | one variant; no new `if`-chain |
| 6 ★ | `walk` / `normalize` handled deliberately | each arm reasoned, not `_`-defaulted |
| 7 ★ | floor / clippy | ≥ **4356/4356/0** · clean |

Rows 1, 2, 3 re-run by the orchestrator by hand.

**Runtime prediction: 60–100 minutes.** Time-box 200.

**Trap doors:**
1. **Expanding the whole `:when` vector.** STOP-2 — it fires `kwargs-lower` on fact patterns. This
   is the single most likely way to break the build, and the failure will look unrelated to `where`.
2. **Hooking `defrule`.** STOP-1 — misses `wat/query.wat`'s generator.
3. **Descending into `quote` generically** instead of only under `make-rule`'s arg 2. Quote carries
   data everywhere else in the language and must keep doing so.
4. **Taking the `_` arm** to silence the two exhaustive matches. STOP-3 — those compile errors are
   the worklist.
