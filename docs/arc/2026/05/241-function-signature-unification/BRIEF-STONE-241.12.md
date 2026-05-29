# BRIEF — Stone 241.12 — `:wat::core::defalias` mint + alias-cascade completion

You are sonnet. NEW Stone 241.12 — mints the missing def*-prefix-family surface form for binding aliases. The substrate cannot complete the HARD CUT discipline for `:wat::core::define` until defalias exists. Stone 241.13 INSCRIPTION closes arc 241 after this.

## CRITICAL doctrine (pre-authorized per `feedback_hard_cut_admits_no_bypasses`)

**HARD CUT IS TOTAL.** The retired form `:wat::core::define` dies EVERYWHERE in the substrate. There is NO privileged path. There is NO substrate-internal bypass. There is NO "stdlib uses it internally so it's OK."

When you encounter a substrate-internal `:wat::core::define` use, it is ONE OF:
- An alias binding → migrate to `:wat::core::defalias`
- A function binding → migrate to `:wat::core::defn`
- A primitive use → migrate to `:wat::core::def`
- One of the 6 acceptable categories (HARD-CUT-rejection error text, retirement table entries, historical comments, probe test source, retirement_lookup fixtures, rejection-routing predicates) → leave

If you find yourself classifying a use as "privileged path" or "intentional bypass" — STOP. That framing is a doctrine violation. The use migrates. Round 2 was killed for exactly this framing; do not repeat.

## What to do

### S1 — Mint `:wat::core::defalias` user-facing surface form

Per intueri cast 2026-05-29 late: defalias is L0 + REMARKABLE. Form shape:

```scheme
(:wat::core::defalias :new::name :original::name)
```

Two positional keyword args:
- `args[0]` — NEW name (the alias)
- `args[1]` — ORIGINAL name (existing binding)

Both names exist post-defalias. Additive (no destruction of original).

Place the parser entry near the existing def*-prefix forms (defstruct/defenum dispatch in `src/types.rs` or wherever `parse_type_decl` routes). The form parses to whatever AST/structure the runtime expects to register the alias.

### S2 — `:wat::runtime::define-alias` DIES; defalias is the SOLE alias mechanism

**User direction 2026-05-29 late: *"at the end of this work :wat::runtime::define-alias is dead - :wat::core::defalias is the only way to do name aliasing."***

The substrate has ONE alias form, not two layers. Stone 241.12 mints `:wat::core::defalias` AND retires `:wat::runtime::define-alias`. The 26 callers migrate. The runtime form gets a HARD-CUT-rejection arm at check.rs + a retirement-table entry.

Implementation: defalias IS the parser + the registration code path. No separate runtime mechanism; no compile-to-runtime-form indirection. The substrate uses defalias directly.

### S2.5 — HARD-CUT-rejection arm for `:wat::runtime::define-alias`

Mirror Stone 241.8/241.9/241.11 HARD-CUT-arm pattern at `src/check.rs`:

```rust
":wat::runtime::define-alias" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.12); use ':wat::core::defalias' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

### S2.6 — Append retirement-table entry

`src/remedy/retirement.rs` `RETIREMENT_TABLE` extends to 5 entries:

```rust
// Stone 241.12 — defalias replaces runtime define-alias
(":wat::runtime::define-alias",   ":wat::core::defalias"),
```

### S2.7 — Cascade migration of 26 `:wat::runtime::define-alias` callers

Mechanical migration; the form shape stays the same (two positional keywords); only the head changes.

### S3 — Audit substrate `:wat::core::define` uses for alias shapes

Run:
```
grep -rn ":wat::core::define" src/
```

Classify each:
- Pattern A (alias shape): `(:wat::core::define :ns::new :ns::existing)` where the body is a single keyword referencing an existing binding → migrate to `:wat::core::defalias`
- Pattern B (function shape): `(:wat::core::define (:ns::name -> :Ret) body)` → migrate to `:wat::core::defn`
- Pattern C (acceptable category) → leave; document in SCORE

Sonnet's per-site judgment based on the body shape.

### S4 — Audit reflection emitters

Run:
```
grep -n "Keyword.*wat::core::define" src/runtime.rs
```

For each AST-construction site producing `:wat::core::define` Keyword: judge whether the emission is for an alias shape (single-keyword body) or function shape. Migrate to emit `:wat::core::defalias` keyword or `:wat::core::defn` keyword accordingly.

### S5 — Cascade migration

Per `docs/SUBSTRATE-AS-TEACHER.md`: read failure → migrate site → re-run. The cascade is bounded by the actual alias-use count (estimated 10-30 substrate-internal sites). Auto-fixer pattern from Stone 241.10 + 241.11 is AVAILABLE if useful, but the small site count likely makes manual migration tractable.

### S6 — Pre-INSCRIPTION grep gate

After all migrations, the discipline gate must pass. Run these greps separately (vanilla, no chained pipes — FM 16 firewall awareness):

```
grep -rn ":wat::core::define" src/ tests/ wat/
```

For each match: is it one of the 6 acceptable categories per the doctrine above? If yes → leave. If no → migrate.

Goal: 0 non-acceptable matches.

### S7 — Probe verification

`tests/probe_arc241_stone12_defalias.rs` (already committed STRIKE-READY). 3 contracts; pre-stone 2/3 PASS (C01/C02 trivially via substrate no-op); C03 fails at HEAD (alias unresolved) → post-stone 3/3 PASS.

## Discipline

- HARD CUT total — no internal bypasses; no privileged paths (per `feedback_hard_cut_admits_no_bypasses`)
- src/argspec/*, src/lib.rs UNCHANGED
- src/remedy/retirement.rs MODIFIED (append `:wat::runtime::define-alias → :wat::core::defalias` entry per S2.6); other src/remedy/* unchanged
- Stone 241.x probes preserved; arc 237/238 probes preserved
- holon-rs NEVER touched (STOP-5)
- No new error variants
- Auto-fixer crate (if used) must be EPHEMERAL — DELETED before commit (per Stone 241.10/241.11 precedent)
- Pre-INSCRIPTION grep gate must be CLEAN post-stone (this is the gate Stone 241.11.fix round 2 failed; Stone 241.12 closes it)

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.12.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.12.md` — D1-D7 + T1-T6 + STOP
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.11.md` — auto-fixer ephemeral discipline + cascade pattern
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.10.md` — substrate-mint shape (defalias parser is structurally similar to defstruct/defenum parsing — keyword + simple positional args)
7. `/home/watmin/work/holon/wat-rs/src/types.rs` — find existing def*-prefix dispatch (defstruct/defenum/typealias/etc.); mirror the routing pattern
8. `/home/watmin/work/holon/wat-rs/src/check.rs` — find the existing HARD-CUT-rejection arms (struct/struct-restricted/enum/define)
9. `/home/watmin/work/holon/wat-rs/src/runtime.rs` — find `:wat::runtime::define-alias` runtime mechanism (26 callers; defalias compiles to this OR you rename per Option B)
10. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone12_defalias.rs` — 3-contract probe (2/3 PASS at HEAD; C03 disconfirms cleanly)

## Implementation sketch

1. Baseline: `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test probe_arc241_stone12_defalias 2>&1 | tail -3` (expect 2/3)
2. **S1+S2**: mint `:wat::core::defalias` parser + connect to runtime mechanism (Option A preferred)
3. **S3**: audit substrate `:wat::core::define` uses → classify each → migrate Pattern A to defalias
4. **S4**: audit reflection emitters → migrate alias-shape emissions to defalias keyword
5. **S2.5+S2.6+S2.7**: add check.rs HARD-CUT arm for `:wat::runtime::define-alias`; append retirement-table entry; migrate 26 runtime callers
6. **S5**: iterate cascade per substrate-as-teacher
7. **S6**: run pre-INSCRIPTION grep — must be 0 non-acceptable matches; ALSO verify `:wat::runtime::define-alias` returns 0 active uses (only retirement entry + HARD-CUT-rejection arm + historical comments)
7. **S7**: verify probe 3/3 PASS; lib ≥ 890; clippy ≤ 902; workspace build clean
8. Write `SCORE-STONE-241.12.md` (place at `docs/arc/2026/05/241-function-signature-unification/` — NOT at repo root)
9. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

1. Compile errors not traced to defalias mint or alias cascade
2. Lib < 890
3. **150 min elapsed**
4. holon-rs touched
5. Substrate `:wat::core::define` use classified as "privileged path" / "intentional bypass" without migration → D7 + `feedback_hard_cut_admits_no_bypasses` violation
6. `:wat::runtime::define-alias` survives as ACTIVE substrate use (any caller still using it post-stone outside HARD-CUT arm + retirement entry + historical comments) — D3 + user direction violation
7. Files outside permitted scope (src/types.rs / src/check.rs / src/freeze.rs / src/runtime.rs / wat/core.wat / stdlib wat / cascade target files / tests/probe_arc241_stone12_* / SCORE doc; remedy/* unmodified except no changes expected)
8. Stone 241.12 probe < 3/3
9. Stone 241.x or arc 237/238 probes regress
10. Clippy > 902
11. Auto-fixer crate survives commit (Stone 241.10/241.11 ephemeral discipline)
12. Pre-INSCRIPTION grep returns ANY non-acceptable matches post-stone

## SCORE doc spec

Mirror SCORE-STONE-241.11.md shape. Include:
- Header (Mode A; runtime; cascade size; auto-fixer used? deleted?)
- Phase A scorecard (probe + lib + clippy + structural)
- Migration cascade audit (per-site count; Pattern A vs B distribution)
- Final defalias parser body (verbatim)
- Pre-INSCRIPTION grep verification (the gate that this stone closes)
- Honest deltas (anything surfaced)
- NO Vigilia section (D6 — no namespaced home)

## Post-strike

Return one paragraph: defalias minted; alias-cascade depth; substrate `:wat::core::define` uses migrated (Pattern A vs B distribution); reflection emitters migrated; pre-INSCRIPTION grep final count (target 0 non-acceptable matches); Stone 241.12 probe 3/3; auto-fixer status (built? used? DELETED?); baselines; SCORE doc path (at arc dir).

The def*-prefix family completes with this stone. Stone 241.13 INSCRIPTION closes arc 241. Arc 237.8b reopens after. Strike clean.
