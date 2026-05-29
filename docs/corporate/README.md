# The Dungeon-Crawl Kit (corporate edition)

A discipline kit for shipping non-trivial software changes. Translated from the `wat-rs/` "dungeon-crawl" methodology with the flavor layer stripped for corporate context.

This kit answers: **how do we ship changes that are obvious, simple, honest, and serve the caller — without making mistakes the type system, tests, and review can catch in advance?**

## What this kit ships

| File | Purpose |
|---|---|
| `README.md` (this file) | Entry point; the four questions; translation table; quick start |
| `TEMPLATES.md` | Copy-paste templates: ADR, PR checklist, failing-test-first, named-follow-up |
| `DISCIPLINES.md` | Deeper dive on the load-bearing disciplines (failure engineering, cascade, HARD CUT, trap-door) |

Start with this file. Pick one or two disciplines to introduce on one project. Expand from there.

## The four questions — the decision compass

Run on every architectural / design decision, **in order**. Each answer is atomic YES or NO. "Medium" means you haven't decomposed enough.

1. **Obvious?** Will a fresh reader immediately understand what this does and why?
2. **Simple?** Is it composed of atomic pieces, each doing one thing?
3. **Honest?** Does it tell the truth about what it does, surface its limitations, and not paper over gaps?
4. **Good UX?** Does it serve the caller well?

**Obvious + Simple + Honest must hold BEFORE Good UX matters.** Any NO disqualifies.

Use this for: choosing between options A/B/C; deciding whether to split a PR; reviewing a design proposal; deciding whether to ship or wait.

Worked example — choosing between two approaches:

> *Path α — atomic ship: Obvious YES; Simple YES (one structural concern); Honest YES (one commit, one narrative); Good UX YES (clean reader experience). 4/4.*
>
> *Path β — split ship: Obvious MARGINAL; Simple NO (the structural act is one thing; splitting it creates either a broken intermediate or a process-as-structure artifact). Disqualified at axis 2.*

The four questions catch what momentum-instinct misses. Run them BEFORE committing to a path, especially when one path "feels right" and you're tempted to skip the discipline.

## Load-bearing patterns (summary; full treatment in `DISCIPLINES.md`)

### 1. Design before code

For any change touching > 1 file or > 50 lines, write a design doc first. Lock decisions explicitly (numbered: D1, D2, …); enumerate trap-doors (T1, T2, …: what could break this); enumerate STOP triggers (what conditions mean we throw this away and re-plan). 30–90 min upfront saves days downstream.

Format: see `TEMPLATES.md` § ADR Template.

### 2. Failing test first

Before writing implementation, write a test that **proves the gap**. Commit it failing. Implementation goes in a follow-up commit; the test going green is the proof.

This is TDD with two sharpenings:
- The failing test gets committed BEFORE the implementation PR
- It must fail on EXACTLY the missing piece (not for some unrelated reason)

If the test fails for the wrong reason, the test is wrong — fix the test, not the implementation.

Format: see `TEMPLATES.md` § Failing Test First.

### 3. Verification checklist (acceptance grid)

After implementation, run an independent verification. Write down what you'll check (the grid); run each check; record the verbatim result. The grid becomes the PR description / definition-of-done.

Pattern: each row of the grid is "Claim | Verification command | Expected result | Actual result." When all actuals match expecteds, ship. When they don't, the gap is named in the grid.

Format: see `TEMPLATES.md` § PR Checklist.

### 4. Named-follow-up deferrals (no unnamed TODOs)

When you legitimately cannot do something in the current PR, the deferral MUST name the follow-up. Not "// TODO." Not "future work." Either:

- An issue / ticket number that exists
- A named follow-up PR / project ("Stone 241.5" — the next named work unit)
- An architectural reason scope-bounding it ("out of this PR's scope; tracked in design doc §X")

Unnamed deferrals decay. Named ones close.

The discipline: if you can't name the follow-up, you can't defer.

### 5. Failure-engineering ladder

When you find a bug, ask: "what would make this CLASS OF BUG impossible?"

- **✅ Convention** — comment / lint rule / code-review checklist
- **✅✅ Construction-time** — assertions, invariant checks, builder patterns that reject invalid combinations
- **✅✅✅ Type-system-impossible** — refactor types so the bug literally cannot be expressed

Aim for ✅✅✅. When you ship at ✅, document why ✅✅✅ wasn't reachable so the next person can climb the ladder.

Worked example: a function taking a slice with a "must have ≥ 3 elements" precondition encoded in a comment is ✅. Changing the signature to `&[T; 3]` (typed array of exactly 3) is ✅✅✅ — the precondition is structurally guaranteed.

### 6. HARD CUT vs shim (atomic legacy retirement)

When replacing an old API/form/pattern with a new one, decide upfront: HARD CUT or deprecation-cycle?

- **HARD CUT**: delete the legacy AT THE SAME COMMIT that migrates all callers. No alias. No shim. The legacy doesn't survive into history's living state.
- **Deprecation cycle**: legacy stays callable + emits warnings; alias to new; callers migrate over N releases; eventual removal.

HARD CUT is cleaner but requires that you OWN all callers (single repo; small team; no external customers depending on the legacy). Deprecation cycle is the corporate default when external API/SDK is involved.

The four questions on HARD CUT:
- Obvious? Yes — one commit retires the legacy.
- Simple? Yes — the structural act is one thing (substrate-change + caller-migration are TWO FACES of the same act).
- Honest? Yes — no half-finished deprecation state on disk.
- Good UX? Depends on caller ownership. If you own all callers → yes. If external customers depend on the legacy → NO; must use deprecation cycle.

### 7. Substrate as teacher (let the diagnostics drive)

When a structural change ripples through many sites, don't pre-plan the migration. Let the compiler/test diagnostics drive:

1. Make the substrate change (e.g., delete the legacy API)
2. Run `cargo test` / `npm test` / etc.
3. Read the FIRST failure
4. Migrate that site
5. Re-run; repeat

Fail-count is the progress meter. Each round drops it. Don't enumerate all sites upfront; the diagnostic stream IS your migration brief.

This pattern handles 30+ site migrations efficiently. The temptation to pre-plan (write spreadsheet of all sites) is wasted effort — the diagnostics find them all without you.

### 8. Trap-door doctrine (build the missing dependency)

When a current change reveals that a prior decision blocks a new need, **build the missing piece forward**. Do not:
- Declare the new need "incoherent given the prior decision"
- Build around the constraint with a hack
- Revert the prior decision

Instead: extend the prior substrate to support the new need cleanly. The trap-door (the discovery) is good; the verdict that the constraint is immovable is the failure.

Worked example: a Vector type signature blocks a generic operation. Don't declare the operation incoherent; add a generic constraint to the Vector type that makes the operation expressible.

### 9. No broken commits

Every commit on the main branch must be green (compiles, tests pass). Broken intermediates violate this — even if "we'll fix it in the next commit."

This rules out certain decompositions: if a change is ONE structural act that produces a green tree, it must be ONE commit. Splitting it into "substrate change + caller migration" with a broken intermediate violates the discipline.

The commit cadence is "often" (small, focused, frequent), not "coverage" (every minute change). Quality over rate.

### 10. Inscription immutability

Past design docs, release notes, retrospectives are READ-ONLY. When you discover a past doc was wrong, do NOT amend it in place. Write a NEW doc that cites the old one and explains what changed. The historical record stays intact.

This is the same rule as `git log` — historical commits are immutable; future commits forward-correct.

Mutable artifacts: the current design doc for in-flight work, the team handbook, the onboarding doc. These accrete; you edit them.

Immutable artifacts: past release notes, past retros, past ADRs that already shipped. These don't change.

## Translation table — datamancy → corporate

| Datamancy / dungeon-crawl | Corporate / professional |
|---|---|
| `arc/NNN-name/` (work unit) | Project folder / Epic |
| `DESIGN.md` (locked decisions + trap-doors + STOP triggers) | ADR (Architecture Decision Record) / RFC |
| `FM 2-bis probe` (disconfirming test) | Failing test committed BEFORE PR |
| `BRIEF.md` (strike path for implementer) | PR description / implementation plan |
| `EXPECTATIONS.md` (verification grid) | Acceptance checklist / verification grid |
| `SCORE.md` (verification record) | PR verification / release notes |
| `INSCRIPTION.md` (closure paperwork) | Done-done definition + retrospective |
| `INTERSTITIAL` (realizations) | Engineering learnings doc |
| `CLIFFNOTES` (load-fast index) | Team handbook / onboarding doc |
| `STOP triggers` (rejection criteria) | PR rejection / definition of "done wrong" |
| `Vigilia` (multi-spell audit) | Code review with assigned reviewer concerns |
| `Spell` (named discipline check) | Skill def / lint rule / review checklist item |
| `HARD CUT` | Atomic legacy retirement (vs deprecation cycle) |
| `Trap-door` | Build the missing dependency |
| `Named follow-up` (FM 11) | Ticket-numbered TODO (never unnamed) |
| `Substrate as teacher` | Diagnostic-cascade migration |
| `Failure engineering ladder` | Make wrong shapes impossible |
| `Inquisitor + Shadowdancer` | Senior engineer + LLM/junior (role split) |
| `Inscription immutability` | Past records read-only; forward-correct |

## Smallest valuable introduction

Don't try to port the whole kit at once. Pick ONE project. Introduce two artifacts:

**Week 1**: Design before code.
- For any PR > 50 lines, write an ADR (`TEMPLATES.md` § ADR Template) before the implementation
- Locked decisions (D1, D2, …); trap-doors (T1, T2, …); STOP triggers
- 30–90 min upfront; saves days downstream

**Week 2**: Failing test first.
- For any bug fix or new feature, write a failing test committed BEFORE the implementation PR
- Test must fail on EXACTLY the missing piece
- Implementation PR's "done" criterion = the test goes green

Those two alone (without the rest) capture ~60% of the discipline's value. Add the four questions to your code-review template → 80%. Add named-follow-up deferrals (no unnamed TODOs) → 90%.

The rest is texture you can introduce over months or keep for yourself.

## When the kit doesn't apply

- **Trivial changes** (typo fix, dependency bump): skip the ADR; just merge
- **External API constraints** (can't HARD CUT): use deprecation cycle instead
- **Solo learning / spike work**: discipline is overhead when you're exploring; introduce it when you ship
- **Team not yet bought in**: introduce one artifact at a time; demonstrate value; expand

## How this kit operates

The kit is artifacts + disciplines. Artifacts are the templates in `TEMPLATES.md`; disciplines are the principles in `DISCIPLINES.md` and the four questions above.

The artifacts ship the discipline to colleagues without requiring them to learn the discipline first. The discipline lives in your head and in code review; the artifacts make the discipline's outputs visible.

The four questions are the only thing you have to memorize. Everything else can be checklist-driven once you've internalized the four questions.

## Cross-references

- `TEMPLATES.md` — copy-paste templates for ADR / PR checklist / failing-test-first / named-follow-up
- `DISCIPLINES.md` — deeper treatment of failure engineering, cascade discipline, HARD CUT decision, trap-door doctrine, inscription immutability
- Original "dungeon-crawl" methodology: `wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`, `wat-rs/docs/SUBSTRATE-AS-TEACHER.md`, `wat-rs/docs/DUNGEON-CRAWL.md` (flavor-bearing; consult when you want the source material)
