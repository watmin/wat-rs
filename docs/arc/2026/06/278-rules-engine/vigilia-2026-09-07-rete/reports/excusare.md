# Excusare — Report: `src/rete/kernel/**` (excl. tests) + `wat/rete/oracle/**`

> Written verbatim as returned. `&lt;`/`&gt;`/`&amp;` are HTML-entity artifacts of the agent's output.

## Enumeration (exhaustive)

Grepped `rune:`, `#[allow`, `#![allow`, `#[ignore`, `#[expect`, `#[deny`, `#[forbid` across all 28 target files. Found:

- **56 `rune:` exemption tags** applied to a code site (2 additional `rune:`-adjacent lines were prose *mentioning* the convention, not an applied exemption)
- **9 `#[allow(clippy::too_many_arguments)]`**
- **0 `#[ignore]`**, **0 `#![allow]`**, **0 `#[expect]`/`#[deny]`/`#[forbid]`**, **0 unsafe blocks**, **0 TODO/FIXME/HACK**

**Excluded as non-exemptions** (verified by reading): `stratify.rs:350` — a record that an exemption was **refused**, the opposite of a live override. `arm.rs:714` — continuation prose inside the `arm.rs:705` comment block, not a second tag.

**Total exemptions weighed: 65.** 65 enumerated, 65 weighed — the two numbers match.

## Verdicts

### Class A — `rune:struere(...)` — 16 sites — ALL HOLDS (Phase 2)

Each cites a named `DESIGN-STONE-*` and a structural warrant checkable in the type/function it guards. Every guarded item was read and confirmed: `node.rs:83` (nine-kind enum, `NodeKind::ALL` at `:36-79`); `session.rs:20/61/85/97` (Copy spans into fire-scoped pools); `session.rs:109` (`fact_at`, two exhaustive arms); `session.rs:616` (`I64Row`, `I64_ROW_CAP` fixed array at `:618-623`); `session.rs:1686/1716` (in-range slices); `mod.rs:1525` (`key_of` panic at `:1532-1535`); `mod.rs:1922` (two stones); `acc.rs:13/81/157/201/214` (refusals via `acc_refusal`, `OperandSlot` 3-way enum at `:144-153`). Closure: none.

### Class B — `rune:sequi(performance-counter)` — 13 sites — ALL HOLDS (Phase 2)

`census.rs` ×12 and `arm.rs:727`. Entire file read: every counter is `#[cfg(test)]`, `None`-by-default, armed only through a paired `with_*_census` that restores prior state, with a `#[cfg(not(test))]` no-op twin. Verified structurally at every site. HOLDS, L1.

### Class C — `rune:perspicere(read-once)` — 8 sites — ALL HOLDS

`census.rs` ×7 and `arm.rs:1072`. Checked each: none of the declarations is re-spelled through an alias. HOLDS, L2 (terse but checkable).

### Class D — `rune:perspicere(intentional-structure)` — 2 sites — HOLDS

`alpha.rs:284` (`Arc::clone` shared across every `aid`, confirmed at `:290-293`); `pass.wat:245` (`Option` vs `Some(empty)` distinction implemented directly in the match arms).

### Class E/F — `sequi(ambient-context)` / `circumspicere(accepted-by-design)` — 2 sites — HOLDS

`arm.rs:705` — cites two named stones; the comment itself records a **2026-08-25 recategorisation** where `sequi` caught this same site mislabelled `host-idiom`, i.e. this rune already survived one Phase-2-style re-audit. `arm.rs:717` — lease boundary, cites `DESIGN-STONE-intern-zero-mutex`, thread-affinity is the checkable mechanism.

### Class G — `rune:temperare(simplicity-win)` — 5 sites — HOLDS, mixed strength

`mod.rs:494` — **measured**, dated, floor-tied (206 / 245583 = 0.084%). Strongest of the five. `mod.rs:311` — duplication argument, self-evident. `root_join.rs:53`, `rules.rs:682`, `filter.rs:57` — all argue "n is small" **without a cited measurement**; the structural warrant is statable (the population is bounded by the *compiled* network/rule count, not runtime fact volume) and each `n` was confirmed to be a compile-time structure. HOLDS, L2 — weaker than the measured sibling; closure: if the axis is ever suspected to grow, take the measurement `mod.rs:494` took.

### Class H — `rune:lint(cited-name-absent)` — 6 sites — ALL HOLDS

`rules.rs:167`, `production.rs:7`, `acc.rs:328`, `alpha.rs:93`, `alpha.rs:169`, `hash_join.rs:21`. Grepped and confirmed none of the cited names exists as a live symbol.

### Class I — `rune:lint(gather-walk-not-examining)` — 2 sites — HOLDS

`mod.rs:840`, `mod.rs:948` — both walk a HashJoin's own index bucket, not an Acc/Neg/Exists gather bucket, which is what `census.rs:229-238` scopes the counter to.

### Class J — `rune:intueri(naming)` — 1 site — **ILLEGITIMATE-AT-BIRTH** (Phase 1)

`wat/rete/oracle/fire.wat:54` — `walk-filter-ids` actually runs accumulate+filter+hash-join. Gated per-component: the "historical" clause explains *why* the mismatch exists but is not a warrant for permanence; "rename would fork every oracle fire caller" is a bare cost-of-change plea, not a domain-truth, and names no stone/arc/ticket — so it also fails the OPEN-DEFERRAL test. Per the borderline-resolves-NO rule, I cannot state in one sentence *from the code alone* why this name must remain wrong forever rather than merely being inconvenient to fix. **ILLEGITIMATE-AT-BIRTH, L2.** Closure: rename it and its callers, or replace the rune with a genuine structural argument.

### Class K — `rune:excusare(no-falsifier)` — 1 site — HOLDS

`session.rs:1834` — verified structurally: no `[profile.release]` override and no `.cargo/config.toml` override of `debug-assertions`, so a release floor builds with `debug_assertions = false`; the guarded `debug_assert!` compiles out and the test module does not exist in that build. The reason names what was tried and why it fails.

### Class L — `#[allow(clippy::too_many_arguments)]` — 9 sites — 4 HOLDS, 5 ILLEGITIMATE-AT-BIRTH

**HOLDS (4):** `hash_join.rs:41` and `join_after_filter.rs:31` — explicit on-point reason ("a context struct would have to be built at every call site in the per-fact hot path purely to satisfy a lint"). `hash_join.rs:427` and `:503` — inherit that argument by direct, checkable pointer ("Twin of `hj_step4_term2`, and lifted for the same reason"). L2, inherited not restated.

**ILLEGITIMATE-AT-BIRTH (5)** — bare `#[allow]`, no reason at the site, no inheritable sibling reason:
- `filter_after_join.rs:17` — the "fix-list F" text used elsewhere in this same pass family is absent here.
- `round_census.rs:25` — `#[cfg(test)]`-only, called nowhere near a hot path; the lint's complaint is arguably *more* valid here.
- `alpha.rs:338` — the only comment present addresses `#[cold]`/`#[inline(never)]`, not argument count; being explicitly a **cold** path, the hot-loop excuse would not even apply.
- `mod.rs:469`, `mod.rs:2128` — no reason at the site.

Closure for all 5: bundle into a context struct (as `RoundScratch`/`JoinIdx`/`FireCtx` already do elsewhere in this codebase for exactly this problem) or add the on-point justification the two legitimate siblings carry.

## Aggregate

| | Count |
|---|---|
| **Total exemptions weighed** | **65** |
| HOLDS | 59 |
| ILLEGITIMATE-AT-BIRTH | 6 |
| OPEN-DEFERRAL / STALE-GUARD / CLOSED-DEFERRAL / ORPHANED | 0 |

**The 6 struck, in priority order:** `alpha.rs:338` (cold path, hot-path excuse inapplicable — least defensible); `mod.rs:469`; `mod.rs:2128`; `filter_after_join.rs:17`; `round_census.rs:25`; `fire.wat:54`.

No OPEN-DEFERRAL targets were found to check for shipped status — none of the 65 reasons cites a pending stone/arc/ticket as its warrant. Classes B and H (19 of the 59 HOLDS) are the most mechanically verifiable in the set — each checked against the actual `#[cfg(test)]`/no-op-twin shape or the actual absence of the cited symbol, not merely read for plausibility.
