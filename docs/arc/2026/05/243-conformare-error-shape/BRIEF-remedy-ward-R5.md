# BRIEF — remedy/ ward R5 (vigilia divergence sweep)

**Target:** `src/remedy/` (`mod.rs`, `rank.rs`, `retirement.rs`, `distance.rs`)
**Goal:** drive the live 8-spell vigilia cast to **L1 + L2 = 0** so `src/remedy/` can earn its vigilatum stamp (the 5th warded home). This is the REMARKABLE bar: every L1 and L2 closes; L3 is taste and is left.
**Mode:** sonnet writes the substrate; orchestrator re-casts vigilia + stamps. **Do NOT commit. Do NOT write a vigilatum stamp.** Leave the tree dirty for the orchestrator's re-cast.

---

## Hard rules

- **Anchor:** your cwd MUST be `/home/watmin/work/holon/wat-rs`. Run `pwd` first. If it shows any `.claude/worktrees/` path, `cd /home/watmin/work/holon/wat-rs` and use `git -C /home/watmin/work/holon/wat-rs` for any git. NEVER operate in a worktree path.
- **Write scope:** `src/remedy/*.rs` PLUS exactly two pre-authorized lines in `src/check.rs` — the `use crate::remedy::nearest_match;` import (~line 4586) and its call site (~line 4616), for the Fix 4 rename ONLY. (Crawl confirmed `nearest_match` is consumed there; the rename is a mechanical identifier swap, no logic change. check.rs is flat-untrusted with no ward to drift, so this is authorized, not a scope violation.) Any OTHER edit to check.rs — or any edit reaching any third file — STOP and report.
- **No commit, no stamp, no push.** Leave the working tree dirty.
- **Build/test commands are cargo** (this is wat-rs Rust, not the Python holon lib — do NOT use `run_with_venv.sh`).

---

## The vigilia cast result being closed (8 inward spells, live MCP, embedded-by-value)

```
purgare    : CONVERGED
sequi      : CONVERGED
temperare  : CONVERGED   (cold/bounded path; allocations are L3, left)
cernere    : CONVERGED   (every RETIREMENT_TABLE replacement form verified live — no migration phantoms)
conformare : CONVERGED   (remedy defines NO error type; Remedy/RemedyKind are payload data, not span-bearing)
struere    : 1 L1 + 1 L2
intueri    : 2 L1 + 1 L2   (union across two independent casts)
solvere    : 1 L2
```

**Aggregate: DIVERGES.** Five fixes below close every L1 + L2. The lead fix (Fix 1) collapses three findings at once.

---

## Fix 1 — `score`-into-the-variant (the lead; closes struere L1 + intueri L1#1)

**Finding (struere L1, `mod.rs:62-77` + `retirement.rs:60`):** `Remedy { kind: RemedyKind, score: u32 }` admits the illegal state `(kind: Retirement, score: 5)`. `score` means edit-distance for `Typo` but MUST be `0` for `Retirement` — enforced only by constructor discipline, not the type. The illegal combination is representable (tests construct `Remedy` directly).

**Finding (intueri L1#1, `mod.rs:68-69`):** the `score` field doc says `0` is "(ordering sentinel, not a distance)" for Retirement — a lie; `retirement.rs:93` correctly calls it an exact table hit (distance 0). The field's dual meaning is the root.

**Fix — make the illegal state unrepresentable.** Move the distance INTO the only variant that has one:

```rust
/// Discriminates the source of a [`Remedy`].
///
/// Variant declaration order IS the Eq-consistency tiebreaker in `Remedy`'s `Ord`
/// (`Typo` before `Retirement`). The order carries ZERO ranking meaning — `score()`
/// + `form` decide all real cases — but DO NOT reorder these variants: it would
/// silently change tie resolution between otherwise-identical remedies. (See solvere
/// fix below — this doc IS that fix.)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RemedyKind {
    /// Levenshtein-derived from a candidate set — the user likely mistyped.
    /// Carries the edit distance (always ≥ 1; exact matches are filtered upstream).
    Typo(u32),
    /// Explicit retirement-table lookup — the form was valid in a prior arc and was
    /// HARD CUT. The replacement is the current canonical form. No distance: an exact
    /// table hit, not a fuzzy match.
    Retirement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remedy {
    /// The candidate form offered as a replacement. (doc as before)
    pub form: String,
    /// Discriminates the remedy source; for typos, carries the edit distance.
    pub(crate) kind: RemedyKind,
    /// Optional migration caveat … (doc as before)
    pub note: Option<String>,
}

impl Remedy {
    /// Ranking score: the Levenshtein distance for a typo; `0` for a retirement
    /// (an exact table hit — distance zero — which sorts ahead of every typo).
    pub fn score(&self) -> u32 {
        match self.kind {
            RemedyKind::Typo(distance) => distance,
            RemedyKind::Retirement => 0,
        }
    }
}
```

Note: the `score` struct field is REMOVED; `score()` (a `pub fn`) replaces it so external read access is preserved.

**Cascade (mechanical — the compiler names every site):**
- `mod.rs` `Ord::cmp` (lines ~104-115): `self.score` → `self.score()`, `other.score` → `other.score()`. Keep the `kind` and `note` tiebreakers and their Eq-consistency comment (still correct, still needed — `Typo(0)` is type-representable even though upstream never builds it, so the tiebreakers keep `a == b ⟺ cmp == Equal` airtight).
- `mod.rs` `kind_annotation` (lines ~160-165): `RemedyKind::Typo => format!("typo, distance {}", r.score)` → `RemedyKind::Typo(distance) => format!("typo, distance {distance}")`. Retirement arm unchanged.
- `rank.rs` `nearest_match` (lines ~63-68): `Remedy { score: dist, kind: RemedyKind::Typo, … }` → `Remedy { kind: RemedyKind::Typo(dist), … }` (drop the `score` field).
- `retirement.rs` `retirement_lookup` (lines ~102-107): `Remedy { score: 0, kind: RemedyKind::Retirement, … }` → `Remedy { kind: RemedyKind::Retirement, … }` (drop the `score` field).
- **All tests** that construct `Remedy { …, score: N, kind: RemedyKind::Typo, … }` → `Remedy { …, kind: RemedyKind::Typo(N), … }` (drop `score`). Retirement test constructions drop `score: 0`. Any test reading `.score` → `.score()`. (Affected tests live in `mod.rs` and `rank.rs` `#[cfg(test)]` blocks — the compiler lists them; `retirement.rs`'s `retirement_score_is_always_zero` reads `r.score` → `r.score()`.)

---

## Fix 2 — `RETIREMENT_TABLE` tuple → named struct (closes struere L2)

**Finding (struere L2, `retirement.rs:60`):** `RETIREMENT_TABLE: &[(&str, &str, Option<&str>)]` — the three columns (retired / replacement / note) are positional; swapping the two `&str` columns compiles silently (tests catch it, the type does not).

**Fix — name the columns so a swap is unrepresentable-wrong:**

```rust
/// One retirement-table row: a retired form, its current replacement, and an
/// optional migration caveat. Named fields make a column swap a compile concern,
/// not a test-caught accident.
struct RetirementEntry {
    retired: &'static str,
    replacement: &'static str,
    note: Option<&'static str>,
}

const RETIREMENT_TABLE: &[RetirementEntry] = &[
    RetirementEntry { retired: ":wat::core::struct", replacement: ":wat::core::defstruct", note: None },
    // … one RetirementEntry per existing row, same data, same order …
];
```

Update `retirement_lookup` to `.find(|e| e.retired == needle).map(|e| Remedy { form: e.replacement.to_string(), kind: RemedyKind::Retirement, note: e.note.map(str::to_string) })`. Update the `retirement_score_is_always_zero` test's `for (retired, _, _) in …` destructure to `for entry in RETIREMENT_TABLE { … entry.retired … }`. Preserve every row's data and the doc comment exactly.

---

## Fix 3 — expand opaque doctrine references (closes intueri L1#2 + L2)

**Findings (intueri):** bare `D6` / `D7` / `D10` decision-code citations don't resolve at the use site.
- `mod.rs:123` `Format rules (per D7):` → drop the code; the rules follow immediately. Use `Format rules:`.
- `mod.rs:25` and `mod.rs:185` `Per D10 (lazy invocation discipline)` → the parenthetical already carries the meaning; drop the bare code: `Lazy invocation discipline:` / `Lazy invocation discipline — call ONLY at error construction paths.`
- `retirement.rs:18` `future-vapor entries are forbidden (per D6).` → drop the code; the surrounding prose already states the rule. `future-vapor entries are forbidden — only shipped retirements appear here.`

Rule: each line must stand alone without an external decision-code lookup. Keep the *meaning*, drop the *opaque label*.

---

## Fix 4 — `nearest_match` → `nearest_matches` (closes intueri L2 cardinality)

**Finding (intueri L2, `rank.rs:50`):** the fn returns `Vec<Remedy>` (up to `TOP_N = 5`) but the singular name reads as one result.

**Fix:** rename `nearest_match` → `nearest_matches` at: the `fn` definition (`rank.rs`), the `pub use rank::nearest_match;` re-export (`mod.rs:46`), the call site in `remedies_for` (`mod.rs:191`), and all test references.

**FIRST run** `grep -rn 'nearest_match' src/ tests/ examples/` to find every consumer. `nearest_match` is on the module's PUBLIC surface (`pub use`), so it may have consumers OUTSIDE `src/remedy/`. If any live consumer exists outside the home, **STOP and report it** before renaming — do not silently edit outside `src/remedy/`. If it's only used inside the home + its tests, do the rename.

---

## Fix 5 — `RemedyKind` tiebreaker contract (closes solvere L2)

**Finding (solvere L2, `mod.rs:80-87, 113-114`):** `RemedyKind` variant declaration order silently doubles as the `Ord` tiebreaker; a future reorder changes sort behavior with no stated contract.

**Fix:** this is the doc comment on `RemedyKind` already shown in Fix 1 ("Variant declaration order IS the Eq-consistency tiebreaker … DO NOT reorder"). Confirm it lands on the enum definition itself (where the order is defined), not only on the `Ord` impl. No code change beyond the doc.

---

## What NOT to touch (settled-rejected / L3 — leave alone)

- **render.rs extraction** — reviewed and rejected in a prior round. `render_remedies` + `kind_annotation` + `note_suffix` stay in `mod.rs`.
- **constructor naming "jargon"** — rejected prior round.
- **cold-path allocations** (`to_string`, `format!` in render/lookup) — temperare classified L3 (bounded cold path); leave.
- **`levenshtein` early-return `as u32` cast / rolling-table** — temperare CONVERGED; not in scope.
- Do not add runes. Every finding above is solvable + non-perf-impairing → it must be FIXED, not runed.

---

## Gates (all must pass before you report done)

```
cargo build -p wat
cargo test -p wat --lib remedy    # remedy module tests — MUST stay 61 passed / 0 failed (verified baseline)
cargo test -p wat --lib           # root-package lib suite must stay green (no new failures)
cargo clippy -p wat --all-targets # no new warnings
```

NOTE: use `-p wat` (the root package). Bare `cargo test --lib` from this dir runs 0 tests (false green) — do NOT use it. The load-bearing gate is `cargo test -p wat --lib remedy` = **61 passed / 0 failed**; if your changes alter that count, the delta must be explained (e.g. a test you intentionally rewrote for the `Typo(u32)` shape) — a DROP in passing count with no explanation is a STOP.

If any gate fails, the failure IS the next instruction (substrate-as-teacher) — read it, fix the named site, re-run. Do not stop on a red you can resolve; do stop + report any red you cannot, or any edit that would reach outside `src/remedy/`.

---

## Report (your final message — the SCORE)

1. **Files touched** (must be only `src/remedy/*.rs` — list them).
2. **Per-finding disposition:** each of the 5 fixes — DONE / how, with the line(s) changed.
3. **Gate results:** the four cargo commands above, pass/fail with counts (`cargo test --lib` N passed / 0 failed).
4. **`nearest_match` rename scope:** confirm whether any consumer existed outside `src/remedy/` (and that you STOPPED if so).
5. **Dirty set:** `git status --porcelain` (must be only `src/remedy/*.rs`; NO other file).
6. Anything surprising (an honest delta), if any.

Do NOT commit. The orchestrator re-casts the live 8-spell vigilia on your dirty tree; if it converges L1+L2=0, the orchestrator writes the hashless vigilatum stamp and lands the ward in one atomic commit.
