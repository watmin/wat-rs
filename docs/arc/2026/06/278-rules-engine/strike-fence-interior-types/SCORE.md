# SCORE — the `where` fence's interior gets typed, and it shipped

Executed per `DESIGN.md`/`BRIEF.md`/`EXPECTATIONS.md`, with one directed amendment from the
coordinator mid-strike (quoted in full below). **Bottom line: SHIPPED, one commit, checker +
corpus fix together.** Floor green (5493/5493, 19 skipped), clippy rc=0.

## The arc this file records

1. Built the cure (fill `mod.rs:282`'s empty arm). Verified against every `EXPECTATIONS.md` row,
   including both mutation proofs.
2. Found a genuine census: `wat-scripts/fixes/to-faithful-clojure-net.wat`, 2 rules, 3 sites — one
   `DESIGN.md` already named (`g3-genuine`), one it didn't (`g7-post-arrow`).
3. Hit a real tension: STOP-1 ("if the census names more than a handful of files, stop" — implying
   proceed when small) vs. STOP-3 ("the corpus is not yours to touch" — no size qualifier) don't
   resolve each other for a small nonzero census, and "commit only on green" cannot be satisfied
   without touching the corpus. **Reverted, reported, stopped** — handed the decision up rather
   than picking a side.
4. **Coordinator's ruling** (quoted below, in full, for the record): fix the three sites by hand
   (not a codemod — the reasoning for why is itself part of the ruling), land both halves in one
   commit. Plus two concrete strengthening requests: a positive control in the clause-level-where
   probe, and a `let`/`match` boundary line in the probe file's own header.
5. Did all four, independently re-verified the coordinator's own evidence rather than trusting it
   on citation, found one more thing along the way (the corpus fix's actual codemod entry point,
   `:fix::convert`, cannot currently be driven end-to-end — a PRE-EXISTING, unrelated bug, not
   fixed, reported below), floor green, clippy clean, shipped.

## The coordinator's ruling, verbatim (for the record — this is what authorized touching the corpus)

> SHIP IT. Your stop was correct and the deadlock was my brief's fault, not your judgment — fix
> the 3 corpus sites and land both halves in one commit.
>
> STOP-1 said "if the census names more than a handful of .wat FILES, stop" — implying proceed
> when it is small. STOP-3 said "the corpus is not yours to touch" — at any size. Those
> contradict each other for any nonempty census, and you hit exactly that. You read them the
> safe way and handed the decision up, which is what I would want. My brief is what needs the
> fix.
>
> 1. Fix the three sites to `:wat::rete::core::i64::=`. All three operands are declared i64 and
>    both rules state numeric intent in their own comments — g3-genuine "span source-len ==
>    name-len", g7-post-arrow "the node one child-index after an arrow". There is no ambiguity
>    about what the author meant.
> 2. It is behaviour-preserving, and I drove that rather than assuming it. `string::=` aliases
>    the GENERIC `:wat::core::=` while `i64::=` aliases the TYPED `:wat::core::i64::=` —
>    genuinely different runtime ops, so agreement is not structural. I drove both over 8 i64
>    pairs including 7/7, 7/8, 0/0, -1/-1, -1/1, i64::MAX/MAX, i64::MIN/MIN, MAX/MIN: 8/8
>    agreement. That is evidence, not proof. Sanity-check the codemod still behaves after the
>    swap; if anything moves, that is a finding worth more than the fix.
> 3. Why NOT a wat-fix codemod — record this reasoning in the commit, because it is the first
>    question a future reader will ask given CLAUDE.md's doctrine. A general codemod for this
>    class would need per-operand type inference the fix framework does not have; a blind
>    `string::=` -> `i64::=` rewrite would corrupt every legitimate string comparison in the
>    corpus. The type checker IS that instrument, and it names the correct comparator per site
>    in its own diagnostic. Three sites, one file, a leaf keyword only — this is not "a
>    structural rewrite across many .wat files", and the checker's per-site diagnosis is
>    stricter than a codemod, not looser.

(The two strengthening requests — the positive control and the let/match boundary line — are
addressed in their own sections below, not reproduced here verbatim.)

## Why NOT a wat-fix codemod — recorded per the ruling's own instruction

This is a leaf-keyword swap (`string::=` → `i64::=`) at three call sites the type checker itself
located and diagnosed by name, in one file, not a structural rewrite across many files —
`wat-rs/CLAUDE.md`'s codemod doctrine targets renames, record→enum migrations, and form flips
applied blindly across a corpus by shape-matching, which is exactly the failure mode that would
make a *general* version of this fix dangerous: a codemod that rewrites every `string::=` it finds
to `i64::=` has no way to tell a genuine string comparison from a mistyped numeric one — it would
corrupt every legitimate string equality in the corpus. The type checker built in this same strike
already IS the correctly-scoped instrument for exactly this class of mistake: it resolves each
operand's real declared type and names the right comparator per site, which is strictly more
precise than a codemod's shape-matching could be. Fixing the three sites it named, by hand, using
its own diagnostic as the specification, is narrower and safer than writing a general-purpose
codemod would be — not a shortcut around the doctrine, the doctrine's own reasoning applied to a
case it was never meant to cover.

## Behaviour-preservation — independently re-derived, not just cited

Re-drove the coordinator's claim myself, live, against the actual rete engine (not just trusting
the citation): built two rules over the same fixture-shaped fact (`:cc::Pair {a b}`), one gated by
`(:wat::rete::where (:wat::rete::core::string::= ?a ?b))`, one by `i64::=`, fired both against 14
i64 pairs (the coordinator's 8 — `7/7 7/8 0/0 -1/-1 -1/1 MAX/MAX MIN/MIN MAX/MIN` — plus 6 more of
my own: `10/100 100/10 1/10 -10/10 0/-0 42/42`):

```
"string::= hits [... {:a 7 :b 7} {:a 0 :b 0} {:a -1 :b -1} {:a 42 :b 42}]"
"i64::=    hits [... {:a 7 :b 7} {:a 0 :b 0} {:a -1 :b -1} {:a 42 :b 42}]"
```

**Identical hit sets, both runs, all 14 pairs.** Confirms the swap is behaviour-preserving over a
wider set than the one handed to me, driven independently rather than re-asserted.

## A new finding from the sanity check — `:fix::convert` cannot be driven end-to-end today, and it is NOT this fix's doing

The coordinator asked to "sanity-check the codemod still behaves after the swap." I tried to drive
`:fix::convert` (the actual entry point `wat-scripts/fixes/to-faithful-clojure-net.wat` exposes,
via its `:user::main` reading a path list from stdin) against a real `.wat` file end-to-end, and it
fails — but **before** reaching either `g3-genuine` or `g7-post-arrow`, and **identically on a
clean, fully-unmodified HEAD** (verified by `git stash`-ing every change, checker included, and
re-running):

```
#wat.kernel/AssertionFailure {... :message "compile-condition: where expr is not total —
':wat::core::string::contains?' is not total" ...
:frames [... :symbol ":wat::rete::compile-condition" ... :symbol ":fix::convert" ...]}
```

Root cause, read directly: `g4-namespaced`'s fence calls `:fix::has-ns?`
(`wat-scripts/fixes/to-faithful-clojure-net.wat:45-46`), which calls
`:wat::core::string::contains?` — a PARTIAL core function used inside a `where` fence, refused by
the NATIVE RETE COMPILER's totality check (`wat/rete/compile.wat`, a completely different, later
stage than this strike's freeze-time validator). This is a real, pre-existing defect,
**unrelated to g3/g7 and unrelated to anything this strike touches** — confirmed pre-existing on
unmodified HEAD, not introduced by the checker or the corpus fix. **Not fixed here** — same
STOP-3 boundary as everything else in `wat-scripts/`, and it is a different bug in a different
rule, not part of the ruling's three named sites. Reporting it because the ruling asked for a
live sanity check and "if anything moves, that is a finding worth more than the fix" — this is
that finding, just not the shape I expected: the codemod's end-to-end path is currently blocked by
something else entirely, so a full before/after diff of `:fix::convert`'s OUTPUT could not be
produced by either state. The isolated comparator-level drive above is the strongest evidence
available under that constraint, and it is independent of, not a substitute for, this finding.

## The two strengthening requests

### Positive control, now IN the file

`wat-scripts/scratch-pad/arc278-fence-interior-types/probe-clause-level-where-fires.wat` now
carries `:clw::r-control` — the identical shape as `:clw::r` (same fact type, same field read,
same `:then`) with the clause-level `where` simply deleted — and both counts print on one line.
Driven:

```
"where-arm 0, control 1"
```

This is the artifact proving the open-question answer on its own: the control fires (the harness —
fact insertion, field read, `:then` — is sound), the where-arm does not (the clause-level `where`
genuinely never evaluates), and a reader a year from now does not need this SCORE.md to see both
halves of that argument.

### The let/match boundary, now beside the claim

`tests/rete/probe_arc278_fence_interior_types.rs`'s header — the file whose existence is the claim
"the fence is type-checked" — now states plainly, beside that claim, that an ill-typed comparator
inside a fence-local `let` body or `match` arm is not checked today, and cites the driven repro
(shadowed `?k`, `i64` → `string`, compiles and fires) rather than only pointing at
`check_fence_interior`'s own doc comment in the checker someone would already have to be editing
to find it.

## EXPECTATIONS — final scorecard, every row

| row | expected | actual |
|---|---|---|
| gap arm unlocks, 3 tests pass | 3 passed | **3 passed**, re-verified on the FINAL tree (checker + corpus fix + both strengthening requests) |
| `legal_fences_still_compile` holds | PASS | **PASS** |
| `inline_twin_is_refused` untouched | PASS | **PASS** |
| bank cleared | `grep -c bad-is-banked` = 0 | **0** |
| `.wat.bad` gate demands failure | 27 passed | **27 passed** |
| contract decision held | no `fact_type` field | **held** — both new variants carry `rule`/`head`/`operand`/`op_type`/`operand_type` only |
| `:282` comment rewritten, not deleted | diff shows rewrite | **confirmed**, in the landed diff |
| ⛔ MUTATION A | no-op ⇒ RED, restore ⇒ green | **confirmed both directions, twice** — once during design, re-confirmed on the exact final pre-commit tree |
| ⛔ MUTATION B | keyword-constant refused; re-suppress ⇒ silent | **confirmed both directions** |
| census reported, then resolved | verbatim list or "zero" | **1 file, 2 rules, 3 sites, all fixed** by coordinator's directed amendment |
| floor | 0 failed, passed ≥5493, skipped 20→19 | **0 failed, 5493 passed, 19 skipped** — matches the coordinator's own prediction exactly |
| clippy | rc=0 | **rc=0** |

Floor, verbatim:
```
Summary [ 457.486s] 5493 tests run: 5493 passed (2 slow), 19 skipped
[floor] exit=0. Log kept at .floor/2026-09-10T08-47-05Z/ regardless — a green run is evidence too.
```

## MUTATION A / B — unchanged from the pre-revert design, re-confirmed on the final tree

Both were driven in full during the initial build (verbatim transcripts below) and MUTATION A was
re-run once more on the exact tree that shipped (after the corpus fix and both strengthening
requests landed) to confirm nothing in that final pass silently broke it.

**MUTATION A** (arm forced to a no-op, live source):
```
$ cargo nextest run --release -E 'test(fence_interior)'
FAIL [   0.338s] (2/3) wat::rete probe_arc278_fence_interior_types::fence_interior_type_error_is_refused
  panicked at tests/rete/probe_arc278_fence_interior_types.rs:83:5:
  `string::=` over two i64-bound join vars inside a `where` fence must be refused at rule-compile
  time — the identical predicate written inline is a ConstraintTypeMismatch.
```
Restored, rebuilt, re-ran: 3/3 pass (see scorecard row above).

**MUTATION B** (driven earlier, on the checker before the corpus/doc changes — the checker code
itself has not moved since):
- Refused, with the real code: `FenceConstraintTypeMismatch {..."defrule \`mub::r\` (where fence):
  \`:wat::rete::core::i64::=\` compares at \`i64\`, but operand \`:not-a-number\` has type
  \`keyword\`..."}`
- Silenced, with `is_non_field_keyword`'s suppression reinstated: `expected an error, got Ok`.

## The open question — `mod.rs:456`'s clause-level `Where` — unchanged conclusion, now self-proving

**Confirmed: it genuinely cannot carry a predicate that is ever evaluated.** Unchanged from the
initial drive; the artifact is now stronger (positive control embedded, see above). `matcher.rs:804`
(`ReteClauseShape::Where(_) => None`) and `compiled_cond.rs:596` (`Op::Fail`) both refuse the shape
unconditionally at fire time, independent of the interior expr. `:456` is untouched.

## STOP-2 — the `let`/`match` shadowing hazard — unchanged resolution, now documented at both ends

`check_fence_interior` does not descend into a `let`'s BODY or a `match` arm's BODY — only into a
`let`'s bound VALUES and a `match`'s SUBJECT (outer scope, before any binder fires). Documented in
the function's own doc comment (`typing.rs`) AND now in the probe file's header (per the
coordinator's second strengthening request), so the boundary is visible from both the
implementation and the claim it qualifies.

## Files changed (landed as one commit)

```
 M src/rete/validate/error.rs                                     (+44)  two new FenceConstraintType* variants
 M src/rete/validate/mod.rs                                       (+9/-2) the arm fills
 M src/rete/validate/typing.rs                                    (+154) check_fence_interior + check_fence_constraint_types
 M tests/rete/probe_arc278_fence_interior_types.rs                       #[ignore] off, let/match boundary line added
 M tests/rete/probe_arc278_fence_interior_types_fence.wat.bad            bad-is-banked rune cleared
 M wat-scripts/fixes/to-faithful-clojure-net.wat                        3 sites, string::= -> i64::=, both commented
?? wat-scripts/scratch-pad/arc278-fence-interior-types/probe-clause-level-where-fires.wat  (positive control added)
?? docs/arc/2026/06/278-rules-engine/strike-fence-interior-types/SCORE.md  (this file)
```

## What I did NOT do

- Did **not** touch `mod.rs:456`'s clause-level `Where(_) => {}` — confirmed by reading the final
  diff (no change there) and by the driven repro (now with an embedded control).
- Did **not** write a wat-fix codemod for the three corpus sites — see "Why NOT a wat-fix codemod"
  above; this was a directed amendment to the original STOP-3 reading, not a reversal of the
  doctrine.
- Did **not** fix `g4-namespaced`'s unrelated pre-existing totality bug
  (`:fix::has-ns?`/`string::contains?`) — out of scope, a different rule, a different bug class
  (totality, not type mismatch), confirmed pre-existing on unmodified HEAD.
- Did **not** re-run the floor more than once. One red run (pre-amendment), one green run
  (post-amendment, this is the one that shipped) — never re-ran the SAME code state hoping for a
  different answer.
- Did **not** delete the scratch-pad `.wat` file, or the corpus's explanatory comments — all kept
  as durable, checked references.

## Honest deltas — what surprised me, this pass

- The coordinator's read of STOP-1 vs STOP-3 as a genuine, unresolved contradiction in the BRIEF
  itself — not a misjudgment on my part — was not something I had confidently concluded on my own;
  I treated the tension as mine to resolve conservatively. Being told explicitly "your stop was
  correct... my brief is what needs the fix" reframes the earlier revert as a full round of the
  process working as intended (surface a genuine ambiguity, do not resolve it unilaterally),
  rather than as inconclusive work.
- The live sanity-check the ruling asked for surfaced a real, if unrelated, second corpus defect
  (`g4-namespaced`'s totality violation) that neither `DESIGN.md` nor my own earlier census pass
  had found, because the freeze-time wall this strike builds and the native-kernel totality check
  that catches `g4-namespaced` are different subsystems checking different things at different
  times — a corpus file can be simultaneously "loads and type-checks clean" and "cannot actually
  be run," and `every_wat_scripts_file_loads_on_the_current_runtime` only tests the former.
