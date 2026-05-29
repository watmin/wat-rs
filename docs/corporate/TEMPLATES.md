# Templates

Copy-paste templates for the four core artifacts. Edit as needed; the structure is load-bearing, the wording isn't.

---

## 1. ADR Template (Architecture Decision Record)

For any change touching > 1 file or > 50 lines. Write BEFORE implementation.

```markdown
# ADR-NNNN — <short title; what is being decided>

**Status:** PROPOSED | LOCKED | SHIPPED | RETIRED  
**Authors:** <names>  
**Date:** <YYYY-MM-DD>

## Why this change

<1-3 paragraphs. The problem this addresses. Why now. What goes wrong if we don't.>

## What this delivers

<Concrete outputs. New API surface. Deleted/changed components. Tests.>

## Locked decisions

### D1 — <decision name>

<Statement of the decision. Rationale. Alternatives considered and rejected with reasons.>

### D2 — <decision name>

…

## Trap-door audit

### T1 — <potential failure mode>

<What could break this design under load / at scale / over time. Mitigation if any.>

### T2 — <…>

…

## STOP triggers — rejection criteria

These conditions mean "throw this away and re-plan":

1. **STOP-1** — <condition that means the design is structurally wrong>
2. **STOP-2** — <…>
3. **STOP-3** — <time-box: e.g., "if implementation hasn't shipped in N weeks, re-design">

Each STOP is REJECTION criteria (not "deal with it later" — actually throw the design away if hit).

## Verification plan

After implementation, the following must hold:

| Claim | How verified | Expected |
|---|---|---|
| <claim> | <command / test / review step> | <expected outcome> |
| … | … | … |

## What this unblocks / closes

<Downstream work this enables. Past ADRs this supersedes.>

## Cross-references

- <other ADRs, design docs, ticket links>
```

---

## 2. PR Checklist Template

The verification grid that ships with every PR. Author fills in expected; reviewer fills in actual.

```markdown
## PR Verification Grid

| Row | Claim | Verification command | Expected | Actual |
|---|---|---|---|---|
| 1 | <e.g., new test passes> | `npm test path/to/new.test.js` | 1/0 | <fill at review> |
| 2 | <e.g., baseline tests preserved> | `npm test` | N/0 (N from baseline) | <fill at review> |
| 3 | <e.g., type-check passes> | `tsc --noEmit` | 0 errors | <fill at review> |
| 4 | <e.g., lint clean> | `npm run lint` | 0 new warnings | <fill at review> |
| 5 | <e.g., build clean> | `npm run build` | exit 0 | <fill at review> |
| … | … | … | … | … |

## STOP triggers fired (any of these = reject the PR; do not merge)

- [ ] Test cascade caused regressions outside the touched surface (broken unrelated tests)
- [ ] New code paths lack failing-test-first proof (or the proof was tampered to make it pass)
- [ ] Unnamed deferrals (`// TODO` without an issue / ticket / ADR reference)
- [ ] Type-system gaps the design said would be closed remain open
- [ ] PR diff exceeds the design doc's stated scope

## Honest deltas

<Anything sonnet-style discoveries surfaced mid-implementation that the design didn't predict. Brief; bullet points. These are NOT scope creep — they're learnings worth recording.>

## Cascade depth

<For changes that ripple: how many sites needed update? What was the diagnostic stream's length?>

## Named follow-ups (if any)

- [ ] <follow-up 1> — tracked at <ticket / next ADR / issue>
- [ ] <follow-up 2> — tracked at <…>

(If you have follow-ups WITHOUT named tracking — fix that before merging.)
```

---

## 3. Failing Test First Template

Commit this BEFORE the implementation PR. It must fail at HEAD on EXACTLY the missing piece.

```typescript
// failing-test-first/featureX.spec.ts
// 
// Disconfirming proof for Feature X.
//
// PRE-IMPLEMENTATION: this test MUST fail at HEAD. The failure must trace to
// EXACTLY the substrate gap (not some unrelated issue). If the failure is for
// a different reason, the test is wrong — fix the test, not the implementation.
//
// POST-IMPLEMENTATION: this test passes. The test going green is the proof.
//
// Implementation PR: <link to follow-up PR; or "follow-up PR pending">

describe('Feature X', () => {
  it('should <do the missing thing>', () => {
    // Setup minimal scaffolding that compiles and runs at HEAD.
    const result = featureX.doThing(input);
    
    // This assertion fails at HEAD because featureX.doThing doesn't exist yet
    // (or returns wrong result). Post-implementation it passes.
    expect(result).toBe(expectedValue);
  });
});
```

```bash
# Commit the failing test FIRST:
git add failing-test-first/featureX.spec.ts
git commit -m "test(featureX): failing-test-first proof for the missing thing

This test FAILS at HEAD because featureX.doThing doesn't exist (or returns
wrong result). Implementation PR pending — the test going green is the proof.

Test failure trace: <paste the verbatim failure output from your local run>"

# Then implementation PR cites the failing test:
# "Closes the gap proved by failing-test-first commit <SHA>"
```

The discipline: the failing-test commit IS the contract. The implementation PR's job is to make THAT test pass without breaking any other test. No moving the test goalposts.

---

## 4. Named Follow-Up Template

For deferrals. Use INSTEAD of `// TODO` or "future work" comments.

### Bad — unnamed deferral

```typescript
// TODO: handle the edge case where input is empty
function processInput(input: string[]) { /* ... */ }
```

```markdown
## Out of scope
- Handle edge case for empty input — future work
```

### Good — named follow-up

```typescript
// FOLLOW-UP: handle empty-input edge case
// Tracked: JIRA-12345 (sprint N+1); ADR-0042 § D3 scope-bounds this PR
function processInput(input: string[]) { /* ... */ }
```

```markdown
## Out of scope (named follow-ups)

- Handle edge case for empty input — JIRA-12345 (sprint N+1)
- Refactor processInput to streaming model — ADR-0043 (proposed)
```

The rule: if you can't name the follow-up (ticket / ADR / next-PR), you can't defer. Either ship it now, or explicitly decide it's not happening and remove the deferral comment.

Acceptable named-follow-up shapes:
- Issue / ticket number that exists
- ADR / design doc reference that bounds the scope
- Named follow-up PR ("PR #234 closes this")
- Architectural reason that scope-bounds the work ("out of this PR's scope because the type-system change is in a separate ADR")

Rejected shapes:
- "Future work"
- "Will be addressed later"
- "When pressure surfaces"
- "If a caller surfaces"
- Bare `TODO:` / `FIXME:` without a reference

---

## Quick-start commit messages

These commit messages embed the discipline. Cargo-cult them at first; you'll internalize the shape.

### Design-doc commit

```
docs(adrs): ADR-NNNN <title> — D1-DN locked; T1-TN audit; STOP triggers

<body explaining the decision context>

D1: <decision summary>
D2: <decision summary>
…

T1: <trap-door + mitigation>
T2: <trap-door + mitigation>
…

Verification plan: <link to grid in ADR>

Follows: <prior ADRs this builds on>
Supersedes: <prior ADRs this retires>
```

### Failing-test-first commit

```
test(feature): failing-test-first proof for <gap>

<test description>

PRE-IMPLEMENTATION: fails at HEAD on EXACTLY <the gap>.
Failure trace verbatim: <paste>

POST-IMPLEMENTATION (when impl PR ships): test goes green.

Tracked: ADR-NNNN <title>
```

### Implementation commit

```
feat(area): <what shipped>

Implements ADR-NNNN <title>; closes the gap proved by failing-test
commit <SHA>.

Verification grid:
- Row 1: <PASS — verbatim>
- Row 2: <PASS — verbatim>
…

Cascade: <if applicable — depth/file-count>
Honest deltas: <anything surfaced mid-implementation>

Named follow-ups (if any): <ticket / ADR refs>
```

### HARD CUT commit (when retiring legacy)

```
refactor(area): HARD CUT — <legacy thing> retires; <new thing> ships

Atomic: substrate change + N caller migration sites in one commit.

Removed: <list>
Added: <list>
Migrated: <N sites>

Per ADR-NNNN: HARD CUT chosen over deprecation cycle because:
- We own all callers (no external customers)
- Migration is mechanical (Pattern A / Pattern B in ADR § Migration)
- The atomic commit preserves no-broken-commits discipline

Verification: full test suite green; no legacy syntax remains.
```
