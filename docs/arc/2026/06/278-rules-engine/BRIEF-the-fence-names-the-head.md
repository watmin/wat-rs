# BRIEF — the fence must name the head

**Anchor:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`; any path containing
`.claude/worktrees/` is harness state — re-cd, and use `git -C <anchor>` for git reads.

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no notification
is coming. Run every command in the FOREGROUND and block on it.

## The defect

A rete `where` predicate must be pure ∧ deterministic. When it isn't, the fence refuses like this
(`wat/rete.wat:569`, and its accumulator twin at `:698`):

```
compile-condition: where expr must be pure and deterministic
```

**It does not say which verb, or which axis.** The rule is refused and the author is told nothing
actionable. That fails the substrate's own standard — R29 `RVINA ERVDIT`: *the ruin IS the lesson*; a
diagnostic that names no remedy teaches nothing. It has been blind since stone 6a.

## Why it is a prerequisite, not a polish

A third axis (`total?`) is queued behind this. Adding a conjunct to a blind message turns 98 corpus rows
into 98 refusals that name no verb — coordinates with no name on them. **Fix the diagnostic, then add the
axis**, and every scream is a worklist entry.

## The information already exists and is thrown away

`src/rete/purity.rs` — read the module doc first, then `classify_fn` and `eval_axis_predicate`. The walk
is shared across axes (`enum Axis { Pure, Deterministic }`, `:47`) and its own doc says the property is
*"falsified only by a concrete violating leaf, which short-circuits up."*

**So the walk knows the offending head at the moment it fails, and discards it to return `false`.** This
is the same shape as 24x's `peer.rs:118` wildcard that erased `RecvError::Shutdown` — a distinction
destroyed at a boundary. The fix is information restoration, not new analysis.

## Rooms — read in order

1. `src/rete/purity.rs` module doc (`:1-31`) — the two-axis model, DEFAULT-DENY, the arc-255 successor.
2. `src/rete/purity.rs:44-52` — `enum Axis`; `:59-62` `OpMeta`; `:98+` `intrinsic_meta` (7 construction
   sites, and the 110-verb `matches!` at `:116`).
3. `src/rete/purity.rs` — the shared `eval_axis_predicate` at `:525`, then its two public faces
   `eval_pure_predicate` at `:553` and `eval_deterministic_predicate` at `:563`.
4. `wat/rete.wat:560-570` — the `where` fence. `:690-700` — the accumulator fence (same message, same fix).
   Both re-verified on the disk 2026-08-02; the `Option/expect` message strings are at `:569` and `:698`.
5. `src/check.rs:19257-19270` — where `:wat::rete::pure?` / `deterministic?` are registered. A third
   sibling registers beside them. (Re-grounded 2026-08-02 — this brief originally said `:19227-19245`;
   the scalar-def gate landed ~30 lines above it in `b18888f8`. Line numbers in this arc drift; confirm
   any citation before you trust it.)

## The shape

**Do not change what `pure?` / `deterministic?` return.** They are `-> :bool`, they are correct, and
consumers exist. Derive them from the richer walk rather than replacing them.

Make the internal walk yield the first violating leaf instead of a bare `false`, and expose it:

```
;; PROVISIONAL NAME — see "naming" below
(:wat::rete::axis-violation <quoted-expr> <axis-keyword>) -> (:Option <record>)
;;   None    => the expr satisfies that axis
;;   Some(v) => v names the offending head (and its axis)
```

`pure?` becomes `(axis-violation e :pure) is None`. One walk, two surfaces.

**The returned value carries at minimum the offending `head`.** If the failing leaf has a `Span` in hand
at that point, carry it too and the fence can point at the sub-expression rather than the rule — that is
strictly better and is the R29 standard. **Report whether the span was available**; do not contort the
walk to manufacture one.

Then the fence message becomes something a reader can act on:

```
compile-condition: where expr is not pure — ':wat::kernel::println' performs IO
compile-condition: where expr is not deterministic — ':wat::core::Uuid/v4' is random
```

Both fence sites (`rete.wat:569` and `:698`) get it; the accumulator one should say "accumulator" not
"where".

## Naming — PROVISIONAL, cast owed

`axis-violation` and the record's field names are **placeholders the orchestrator wrote**. Do not treat
them as ratified. Use them to build, and **report the surface you ended up with** so intueri can be cast
on it before this lands. Naming decisions are cast, never narrated.

## STOP triggers — rejection criteria. Ship nothing, report.

- **STOP-1 — `pure?` / `deterministic?` change their return type or their verdicts.** Any existing caller
  seeing different behaviour is a halt. This stone is additive: same bools, same answers, new detail
  alongside.
- **STOP-2 — the walk cannot identify a single offending head.** If falsification is not attributable to
  one leaf (a composite/aggregate case), HALT and report the case rather than inventing an attribution.
  A diagnostic that names the *wrong* verb is worse than one that names none.
- **STOP-3 — a fence message would name a head for a rule that currently compiles.** The set of accepted
  `where` exprs must not move by one. This is a message change, not a semantics change.

## Gate

1. `cargo build --release --all-targets` → exit 0, **zero warnings**; `cargo clippy` likewise.
2. **The corpus is unmoved.** `./wat-scripts/perf/grid/check-where-shapes.sh` — 9 pairs, 98 rows, all
   agreeing. No cargo needed; ~35s. If a single row moves, that is STOP-3.
3. **A RED probe that shows the new message, mutation-proven.** A `where` holding a genuinely impure verb
   and one holding `Uuid/v4`. Paste the verbatim message for each, before and after your change.
   Put it in `tests/`, **not** `wat-scripts/` (every `.wat` there is loaded and type-checked by a corpus
   gate, so a deliberately-bad one goes permanently red).
4. Report whether a `Span` was reachable at the failing leaf.

**Do NOT run `cargo nextest`** — the orchestrator weighs the floor centrally. Baseline re-grounded
2026-08-02: **4268 run / 4268 passed / 0 failed / 262 skipped at `c246bc23`** (this brief originally
cited 4266/4266 at `72a1ac3d`, which predates the namespacing wall's follow-on stones).
Do not commit, push, stash, or revert.

## Report

Diff per file; build + clippy; the corpus-gate result; the verbatim before/after fence messages; the
final surface + names you used (for the intueri cast); the span answer; any STOP with evidence.
