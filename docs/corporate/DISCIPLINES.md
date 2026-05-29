# Disciplines

Deeper treatment of the load-bearing patterns. Read after `README.md`. Use as reference when applying the disciplines to specific situations.

---

## Failure engineering — make wrong shapes impossible

The premise: bugs are not random. They are STRUCTURALLY POSSIBLE because some shape in the system fails to PREVENT them. Failure engineering asks: "for this CLASS OF BUG, what would make it structurally impossible?"

### The ladder

- **✅ Convention** — a code comment, lint rule, code-review checklist item. The bug can still be written; the convention says "don't." Useful when no stronger guarantee is available.
- **✅✅ Construction-time** — an assertion at object construction; a builder pattern that rejects invalid combinations; a runtime check that fails fast on misuse. The bug can be written but is caught at the boundary.
- **✅✅✅ Type-system-impossible** — a type signature that cannot express the bug. No assertion needed because the wrong shape doesn't compile.

Aim for ✅✅✅. When you ship at ✅ or ✅✅, document WHY ✅✅✅ wasn't reachable so the next person can climb the ladder.

### Worked example — precondition encoding

```typescript
// ✅ Convention only — the comment is the discipline
/** Caller must ensure arr.length >= 3 */
function processTriple(arr: number[]): Triple { /* ... */ }

// ✅✅ Construction-time — caught at the boundary
function processTriple(arr: number[]): Triple {
  if (arr.length < 3) throw new Error('precondition violated');
  /* ... */
}

// ✅✅✅ Type-system-impossible — the wrong shape doesn't compile
function processTriple(arr: [number, number, number]): Triple { /* ... */ }
```

The ✅✅✅ form: callers must construct a 3-tuple to call. They can't pass an array of unknown length. The precondition is structurally guaranteed.

### When to climb the ladder

Climb at design time, not bug-fix time. The "you found a bug at convention level" moment is the prompt:

> "What ladder rung is this CLASS of bug currently at? What would climb it one rung?"

Don't fix the specific bug at ✅ then move on. Fix the CLASS at ✅✅✅ when possible, ✅✅ when not.

### Anti-pattern — comment-only discipline that future-readers will violate

> *"This function returns null when X but throws when Y — see comment"*

The comment lies because a future reader will pass input that triggers Y and not check the docs. Use sum types / Result / Either at the return type to make the two outcomes structurally distinguishable. Climb the ladder.

---

## Substrate as teacher — let the diagnostic cascade drive

When a structural change ripples through many sites (delete an API, rename a type, change a method signature), the diagnostic stream IS your migration brief.

### The pattern

1. **Make the substrate change** (e.g., delete the old API)
2. **Run the build / test suite** with `--no-fail-fast` if available
3. **Read the FIRST error** in the diagnostic stream
4. **Migrate that site** to the new API per the conversion pattern in your ADR
5. **Re-run**; the fail-count drops by 1+
6. Repeat until 0 failures

The fail-count is the progress meter. Each iteration drops it.

### Why this beats pre-planning

Pre-planning the migration (spreadsheet of all sites; estimate each; track in a project board) is wasted effort. The diagnostic stream finds them all WITHOUT you. The stream is exhaustive: if a site is broken, the build/test will say so.

Pre-planning costs hours. Following the stream costs the same total time but produces correct work; pre-planning often misses sites that the diagnostics would have found.

### When this discipline applies

- Renames, type changes, API removals, interface changes
- Large mechanical migrations (30+ sites)
- Anytime the change is structurally one act with many caller impacts

When it doesn't apply:
- Behavior changes (where the test passes but the BEHAVIOR is wrong — needs careful per-site review, not mechanical follow-the-stream)
- Cross-cutting concerns (logging, metrics) where the change must thread through every site coherently

### Avoiding the panic reflex

Initial fail-count after a substrate change can be hundreds. The panic reflex says "stash + revert; re-plan." Don't. The hundreds of failures are the work, not a disaster. Iterate. Watch the count drop. Each round teaches you the next category of fix.

---

## HARD CUT vs deprecation cycle — choose at design time

When you replace an old form with a new form, decide upfront:

- **HARD CUT**: delete the legacy AT THE SAME COMMIT that migrates all callers. No alias. No shim. Legacy doesn't survive past this point.
- **Deprecation cycle**: legacy stays callable, emits warnings, callers migrate over N releases, eventual removal.

### The four questions on HARD CUT

| Axis | HARD CUT verdict |
|---|---|
| Obvious? | YES — one commit retires the legacy; reader sees the atomic transition |
| Simple? | YES — the structural act is one thing; the substrate change and caller migration are TWO FACES of the same act |
| Honest? | YES — no half-finished deprecation state on disk; no "we'll delete the alias eventually" |
| Good UX? | DEPENDS ON CALLER OWNERSHIP — see below |

### When HARD CUT applies

- You own all callers (single repo, small team)
- No external customers depend on the legacy API
- Migration is mechanical (you can mass-rewrite per pattern A → pattern B)
- The cleanup value > the disruption cost

### When deprecation cycle is mandatory

- External SDK / API customers depend on the legacy
- The legacy is in a contract you can't break
- Migration requires per-caller judgment (not mechanical)
- N-release timeline is mandated by stakeholder agreement

If deprecation cycle is mandatory, write the migration ADR that pins the N-release timeline AND the "delete the alias at release X" follow-up.

### Anti-pattern — "deprecation cycle by default"

Teams default to deprecation cycle "to be safe." This is rarely necessary internally. The cost of deprecation cycle:
- Two API surfaces to maintain
- Confusion about which to use in new code
- Warnings that get ignored
- The "delete the alias" follow-up that never lands
- Dead code that future readers spend cycles understanding

When the four questions clear HARD CUT, use it. Deprecation cycle is the EXCEPTION, not the default.

---

## Trap-door doctrine — build the missing dependency forward

When a current change reveals that a prior decision blocks a new need, **build the missing piece**.

### What this rejects

- "This new need is incoherent given the prior architecture" — REJECTED; the substrate has structure; it can be extended
- "We'd have to undo the prior decision to support this" — REJECTED if the undo-cost > extend-cost; investigate the extend-path first
- "Let's build around the constraint with a hack" — REJECTED; the hack persists; the architecture decays

### What this accepts

- "The prior architecture didn't anticipate this; we extend it to support the new need; the extension is generic enough that future similar needs are also supported"
- "We add a type parameter / generic constraint / new method that makes the operation expressible"
- "We split a monolithic abstraction into two pieces so the new need can compose them differently than the existing callers do"

### Worked example

> **Constraint:** Vector type is parameterized over T; arithmetic methods are defined for `T = i64` only.
> **New need:** generic sum function that works for any Vector<T : Numeric>.
>
> **Anti-pattern:** declare the new need incoherent because Vector arithmetic is fixed.
> **Trap-door doctrine:** introduce a `Numeric` trait; bound the existing arithmetic methods on it; the generic sum function takes `<T : Numeric>`. Other operations that depended on `T = i64` also benefit.

The dig that found the constraint is good. The verdict that it's immovable is the failure.

### STOP phrases that signal the anti-pattern

If these phrases want to leave your fingers, STOP and re-examine:

- "This contradicts the design"
- "Can't without un-doing prior work"
- "Incoherent given the existing architecture"
- "We'd need a major refactor to support this"
- "Future arc when this surfaces" (with no named follow-up)

Each phrase is the anti-pattern in voice. The pattern: ask "what's the minimal extension that supports this need without undoing anything?" Build that.

---

## No broken commits — atomic-act ships green

Every commit on the main branch must compile and pass tests. Broken intermediates are not allowed, even if "we'll fix it in the next commit."

### What this rules out

- Substrate change in commit A; caller migration in commit B; intermediate A is broken
- API change in commit A; test update in commit B; intermediate A has failing tests
- Refactor that splits across N commits with broken intermediates at each step

### What this admits

- **One atomic commit** containing substrate + callers; passes tests; ships
- **N sequential commits** EACH PASSING tests; the work is decomposable into N green steps
- **Stacked PRs** that land sequentially, each green at land time, that produce the larger change

### How to decide

If a change is STRUCTURALLY ONE ACT (substrate + callers can't be separated cleanly), it's ONE commit. Bundling matches structural shape. Per the four questions on splitting: if Simple is NO at axis 2 (the parts aren't independently meaningful), bundle.

If a change is STRUCTURALLY N ACTS (e.g., refactor that introduces a new abstraction in step 1, migrates some callers in step 2, migrates more callers in step 3), it's N commits each passing tests.

The discipline question: would the intermediate be MEANINGFUL to a reader landing there? If yes → separate commits. If no → one atomic commit.

---

## Inscription immutability — past records are read-only

Past design docs, release notes, retrospectives, post-mortems, ADRs that already shipped — these are READ-ONLY. Do not amend them in place.

### When you discover a past doc was wrong

DO:
- Write a NEW doc that cites the old one and explains what changed
- The new doc carries the corrected understanding
- The old doc stays as-is (historical record)
- The team handbook / current onboarding doc references the NEW doc as the current truth

DO NOT:
- Edit the old doc to "fix" the error
- Add `EDIT: this was wrong, see X` annotations to the old doc
- Quietly rewrite the old doc to match current understanding

### Why

The historical record is data. Failures-and-corrections preserve the LEARNING. Editing the historical record erases the learning and pretends the discipline is more linear than it is.

Same shape as `git log`: you don't rewrite history; you commit forward. The reader who lands on the old doc gets the historical truth + a pointer to the current truth via cross-references.

### Mutable vs immutable artifacts

**Mutable** (you edit these in place):
- Current ADR / design doc for in-flight work
- Team handbook / engineering wiki
- Onboarding doc / runbook
- Index docs / table-of-contents docs

**Immutable** (you write new ones, never edit):
- Past release notes
- Past retros / post-mortems
- ADRs that have shipped and closed
- Inscriptions (closure paperwork)

### The discipline in practice

When you find a past mistake:

1. Open a NEW doc (next-numbered ADR / retro / etc.)
2. The new doc starts by citing the past doc
3. Explain what you discovered, why the past doc was wrong
4. Document the corrected understanding
5. Update the current handbook / onboarding doc to reference the new doc

The past doc + the new doc together form the corrected record. Neither is the "single source of truth"; the chain is the truth.

---

## Cross-references

- `README.md` — entry point + four questions + translation table
- `TEMPLATES.md` — copy-paste templates for ADR / PR checklist / failing-test-first / named-follow-up
- For the original methodology with flavor preserved: `wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`, `wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
