# SCORE — five small L1s in the rete test corpus, one floor

Executing strike per `DESIGN.md`. Appending as each item's verdict settles (house rule).

## `4X1` — the only live external suppression in 306 files, and it pleads nothing

**Re-derived.** `grep -rn '#\[allow' src/rete/kernel/tests/fanout_cost.rs` → one hit, line 841,
matching `DESIGN.md` exactly: `#[allow(unused_variables)]` above `let child_tax: f64 = ...` with no
reason string.

**Read the binding.** `grep -n 'child_tax' src/rete/kernel/tests/fanout_cost.rs` → **one** hit total
(the `let` itself). `child_tax` is computed (a `.map().sum()` over `acc`, purely a read of already-
collected data, no side effects) and then never read again — not printed, not compared, not used by
`top_sum` or the body dump below it. It is dead in the strict sense: assigned once, read never. The
comment above it (lines 834-840, "NESTING TAX...") documents *why the tax exists conceptually*, and
the comment below it (formerly starting "NOT applied to the parent") documents the decision NOT to
subtract it — both comments are prose about an idea, not about the `let` binding itself.

**No delta from DESIGN** on the site or the shape of the defect.

**Cure applied: removed the variable** (DESIGN's preference — "prefer removal if nothing needs it").
Since `child_tax` had zero readers, deleting the binding and its `#[allow]` is a pure no-op on
behaviour: nothing computed from it, nothing printed depends on it. I merged the two comment blocks
that used to bracket it into one, and reworded the second block (formerly "it estimates this tax at
~11-12 ms") so it no longer refers to a `child_tax` variable that no longer exists — it now reads as
prose describing a `cal * pairs` estimate that was checked by hand and rejected, not a live
computation. This is a comment-only rewording beside the deletion; no assertion, no printed value,
and no test behaviour changed (the function this lives in — `fanout_cost`'s census dump — still
prints exactly the same `body`/`top_sum`/`wall` values it did before; `child_tax` never fed any of
them).

**STOP check:** confirmed no printed number or test behaviour changes — `cargo build --release`
diff-checked below, and the printed dump format (`println!` at ~line 867) is untouched.

**⛔ FIRST FLOOR ATTEMPT WENT RED — caused by this cure's removal, caught and fixed in place.**
Driving the actual floor (not a reading) is what found this; neither DESIGN nor a pure re-derivation
of the site itself could have. Full account, per the "on any red" protocol:

**The exact arm.** `wat::lint rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves`
FAILED at `tests/lint/rete_citation_resolves.rs:566:5` (the `assert!(unresolved.is_empty(), ...)` in
`every_backticked_name_in_a_rete_comment_resolves`). Verbatim panic body:

```
🔥 1 name(s) cited in a comment under src/rete resolve to NOTHING — not a Rust identifier in any
code position under ["src", "crates", "tests", "benches", "examples"], not a wat identifier under
["wat"], and not the stem of any source file. A reader following one of these finds nothing, and
the claim the sentence makes cannot be checked.
...
Unresolved:

  src/rete/validate/mod.rs:408  `unused_variables`
```

Floor summary at that run: `Summary [ 452.710s] 5485 tests run: 5484 passed (2 slow), 1 failed, 19
skipped` (`.floor/2026-09-09T04-02-52Z/`, superseded — NOT the floor quoted at the bottom of this
file, kept here only as the diagnostic record. Per the no-re-run rule this was not discarded before
being read and named).

**Mechanism, named exactly.** `src/rete/validate/mod.rs:408` carries a pre-existing, untouched
comment citing `` `unused_variables` `` (the Rust lint name) — see `validate/mod.rs:404-408`, about
`validate_clause`'s destructuring discipline. This gate requires every such backticked citation to
resolve to a real Rust identifier in a CODE position (not a comment) somewhere under
`src/crates/tests/benches/examples`. **The `#[allow(unused_variables)]` I deleted at
`fanout_cost.rs:841` was the ONLY code-position occurrence of that identifier anywhere in the
scanned tree** (confirmed: `grep -rn "allow(unused_variables)" --include=*.rs src crates tests
benches examples` returned zero hits with the deletion in place). Deleting it didn't just remove a
dead variable — it silently starved an unrelated citation of its only remaining referent.

**What I did about it, in order (no re-run before this):** captured the whole block above, named the
mechanism, then reverted the deletion (`git checkout -- src/rete/kernel/tests/fanout_cost.rs`) and
re-applied `4X1`'s OTHER DESIGN-sanctioned cure instead of the removal: **kept `child_tax` and its
`#[allow(unused_variables)]`, and gave the allow a reason** — the alternative DESIGN itself offered
("either the allow carries a reason... or the variable goes"). The reason written states the TRUE,
now-verified cause: that this is the sole surviving code-position occurrence the citation gate
depends on, discovered by driving the floor. Exact text now above the `#[allow]`:

> `child_tax` itself has no reader below — the decision (see the next comment) is to NOT subtract
> it, so this binding stays a computed-but-unused reference value. It must stay a binding rather
> than be deleted outright: this `#[allow(unused_variables)]` was the ONLY code-position occurrence
> of the `unused_variables` lint name anywhere under src/crates/tests/benches/examples, and
> `tests/lint/rete_citation_resolves.rs` requires the backticked `` `unused_variables` `` cited in a
> comment at `src/rete/validate/mod.rs:408` to resolve to some real code position with that name.
> Deleting this line (confirmed by driving the floor) turns that unrelated citation red. So the
> value is kept computed here — not inlined into the prose below as a bare number — so it stays a
> checkable, live figure rather than a comment nobody can re-derive.

This still cures `4X1`'s actual defect (an allow pleading nothing) without DESIGN's preferred
sub-choice (removal), because the preferred sub-choice has a proven, real side effect and the
non-preferred one does not. **This is a delta from DESIGN's stated preference, made for a reason
DESIGN could not have known (it names a cross-file coupling only the real floor reveals).**

**Verified the fix before re-running the floor**, per the standard of not re-running blind: checked
by hand that my new comment's own backtick tokens are safe under `rete_citation_resolves.rs`'s exact
extraction rule (`is_citation`: a bare token, ALL-alnum/underscore, no `/`/`:`/`#`/`[`/`(` chars) —
`` `#[allow(unused_variables)]` `` fails that shape (not a citation at all), `` `child_tax` ``/``
`unused_variables` `` are real citations that now resolve via the restored code position, and the
slashed paths (`` `tests/lint/rete_citation_resolves.rs` ``, `` `src/rete/validate/mod.rs:408` ``)
are excluded from that gate by containing `/` and are instead `no_stale_path_in_doc.rs`'s territory,
where both resolve (`mod.rs` exists, is 1765 lines so `:408` is in range; the test file exists at
that exact repo-relative path). Then ran the two specific gates directly (not the floor, a
build/clippy-style targeted check before spending the ~7.5 minute floor again):

```
cargo nextest run --release -E 'test(every_backticked_name_in_a_rete_comment_resolves) + test(every_location_named_in_a_doc_comment_exists)'
        PASS [   0.052s] (1/2) wat::lint no_stale_path_in_doc::every_location_named_in_a_doc_comment_exists
        PASS [   0.323s] (2/2) wat::lint rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves
     Summary [   0.339s] 2 tests run: 2 passed, 5502 skipped
```

Both green. `cargo build --release` and `cargo clippy --all-targets --release -- -D warnings`
re-run clean (RC=0) after this second edit too — see the Gates section for both runs' output. The
floor is re-run once, below, now that the specific mechanism is understood and independently
verified rather than hoped fixed.

## `4E1` — deferral prose that is really an invariant (`termination_verdict.rs`)

**Re-derived.** `src/rete/kernel/tests/termination_verdict.rs:1-8`, matches DESIGN. Confirmed zero
`arc`/`DESIGN`/`rune:` tokens in the file (`grep -n 'arc\|DESIGN\|rune:' src/rete/kernel/tests/termination_verdict.rs`
— see gate output below).

**Cure applied:** rewrote the sentence to drop "affirmatively out of scope for the strike that split
this type" (deferral framing naming no findable artifact) and state the invariant plainly: that
`NotAnalysable`/`Proven` answering identically as `Compiled` IS the contract, and a new
`CompileOutcome` variant would *break* that identity rather than merely being unbuilt work. Exact
text written:

> `TerminationVerdict::NotAnalysable` is deliberately NOT wire-visible: the invariant is that
> `NotAnalysable` and `Proven` are indistinguishable to a wat caller, both answering `Compiled`
> behind the outcome wall — a new `(:wat::rete::CompileOutcome)` variant would break that identity,
> not merely extend it. From wat, `NotAnalysable` and `Proven` both answer `Compiled` — which is
> precisely the behaviour these probes must NOT disturb — so the only place the distinction is
> observable is at the `pub(crate)` boundary, here.

Doc-comment only; no code, no test, no printed value touched.

## `4E2` — deferral prose that is really an invariant (`probe_arc278_P4c_native_retraction.rs`)

**Re-derived.** `tests/rete/probe_arc278_P4c_native_retraction.rs:1-12`, matches DESIGN's
`:8-9` citation (the parenthetical sits inside the paragraph spanning those lines).

**Checked the STOP condition first, per the brief.** Read the full paragraph before touching
anything: *"there is NO separate incremental support-store retract cascade to build in the
value-semantics surface"* — present tense, and true of the code as it stands (P4b's replay already
makes retraction correct without a support-store cascade; this file's own probes are the proof). The
defect is exactly and only the parenthetical's last clause — *"a deferred surface"* — which frames
the support store's O(delta) benefit as future work rather than as an architectural fact about a
different engine shape (persistent, cross-fire, streaming) that this value-semantics surface simply
is not. **DESIGN's primary claim holds; only the parenthetical is the defect, as DESIGN says. No
STOP.**

**Cure applied:** reworded the parenthetical from "a deferred surface" to state the architectural
boundary as present fact. Exact text written (full sentence, cure in bold-equivalent):

> So there is NO separate "incremental support-store retract cascade" to build in the
> value-semantics surface (each `fire` rebuilds from facts; the support store's O(delta) retract is
> a benefit that belongs to a PERSISTENT cross-fire streaming engine, which this value-semantics
> surface architecturally is not).

Doc-comment only; no code, no test, no printed value touched.

## `4F1` — a 460-to-10 convention with nothing holding it up

**Re-derived the ten sites.** `grep -n '\.unwrap()\|\.expect(' tests/rete/probe_arc278_8i_accumulator_folds.rs`
→ exactly the ten lines DESIGN names: 26, 32, 38, 44, 50, 56, 63, 70, 77, 83, all bare `.unwrap()`
on `call_beside_value(...)`. No delta on the site list.

**Re-derived the sibling population — found a delta.** DESIGN says "460" `.expect("<context>")`
siblings. Measured three different populations:
- `call_beside_value(...).expect(` repo-wide: **462**, not 460 (off by 2; not chased further — the
  exact count does not change the cure, and re-deriving to single-digit precision on a "convention"
  claim is not what this item's cure depends on).
- `call_beside_value(...).unwrap(` repo-wide: **10**, and `grep -rl` confirms they are ALL in this
  one file — the ten-site population DESIGN describes is not just locally uniform, it is globally
  unique. That is the fact the cure actually rests on, and it re-derives clean.
- Confirmed the named model sibling, `probe_arc278_5b_collect_rules.rs:16,22`, does use
  `.expect("eval")` exactly as DESIGN states.

**Cure applied:** rewrote all ten `.unwrap()` → `.expect("eval")`, matching the sibling convention
exactly (same context string as the named model file). Verified post-edit: zero `.unwrap()` remain
in the file, all ten are now `.expect("eval")`.

**Did NOT add a lint** for this convention, per DESIGN's explicit rejection (`conformare`'s proposal
is out of scope). I don't think a lint is warranted either — ten sites in one file, already fixed,
with no mechanism generating new bare `.unwrap()` sites at this specific call shape; a lint here
would be gating a population of one historical file, not a live risk. Leaving it undone, as
instructed.

**STOP check:** `.expect(msg)` and `.unwrap()` differ only in the panic message text on the failure
path; on the green path (which is 100% of a passing floor run) they are identical — same `Ok` value
returned, same success behaviour, no printed value or assertion outcome changes. No STOP.

## `4Q1` — an ambient chain with no rune anywhere in 306 files

**Re-derived the site.** `tests/rete/probe_arc278_import_accounting.rs:176-215`,
`an_origin_already_filed_is_never_re_based` — matches DESIGN. Read `alloc_counter.rs` in full for
the three `thread_local!` cells: `THREAD_LIVE` (:91), `SESSION_ORIGINS` (:168), `LAST_ORIGIN` (:192)
— all confirmed at those line numbers, all reached through free functions
(`thread_bytes`/`mark_session_origin_at`/`session_bytes`) whose signatures carry none of the
ambient state. `mark_session_origin_at` confirmed to return `()` — the mutation is invisible in the
type, exactly as DESIGN says.

**The "five steps"**, made explicit since DESIGN doesn't enumerate them: (1) `thread_bytes()` at
line 193 to capture `late`, (2) `mark_session_origin_at(key, 0)` at 201, (3)
`mark_session_origin_at(key, late)` at 204, (4) `session_bytes(key)` at 206 (which itself internally
re-reads `thread_bytes()` and touches `LAST_ORIGIN` — the fifth ambient touch), plus the ballast
allocation/`black_box` between (1) and (2)/(3) that the whole test exists to make observable.

**Cure applied:** added `rune:sequi(ambient-context)` as a comment at the top of the test function,
immediately before the existing informal argument at (what were) lines 178-180, naming the
invariant explicitly: that nothing else touching this thread's byte counter (no other session's
`mark_session_origin*`/`session_bytes` call) interleaves with this test's own sequence on the same
OS thread. Kept the pre-existing key-collision argument verbatim right after it — that argument is
correct and adjacent, not superseded. Exact text added:

> `// rune:sequi(ambient-context) — `thread_bytes`/`mark_session_origin_at`/`session_bytes` below
> all read and write the same per-thread `THREAD_LIVE`/`SESSION_ORIGINS`/`LAST_ORIGIN` cells in
> `alloc_counter.rs`, invisibly to every one of these signatures. The invariant this test's ordering
> depends on: nothing else touching this thread's byte counter runs between the reads below — no
> other session's `mark_session_origin*`/`session_bytes` call interleaves with this test's own on
> the SAME thread.`

**Doctrine wrinkle #1 (the one DESIGN flags):** the `sequi` spell says hidden domain state "has no
rune category by design"; this repo's `docs/CONVENTIONS.md` (the `rune:sequi` vocabulary table,
~line 1062-1070) mints `ambient-context` for exactly this shape and cites `ARM_TABLE`/`EXEC_ARENA`
as the worked examples. **I am following the repo's vocabulary, not the spell's**, per DESIGN's own
direction — flagged here as instructed, not silently resolved.

**Doctrine wrinkle #2 (NOT in DESIGN — found while re-deriving the gate DESIGN cites as the
enforcement mechanism):** DESIGN states "`no_unknown_sequi_rune.rs` gates it." **Read the gate's own
source** (`tests/lint/no_unknown_sequi_rune.rs:84`): `collect_rs(Path::new("src"), &mut files)` —
it walks **`src/` only**. `tests/rete/probe_arc278_import_accounting.rs` is under `tests/`, so this
specific rune is **not** in the population that lint scans at all; a malformed category on this
exact line would not go red. (Confirmed the precedent: `tests/types/probe_stone_binder_is_universal.rs:58`
already carries a `rune:sequi` — without a category in parens at all — which the gate also can't
see, for the same reason.) This doesn't change the cure — the vocabulary is still the right one to
follow, and the rune is still the honest documentation move DESIGN wants — but DESIGN's specific
claim that this gate enforces THIS site's category choice is false. Flagging per the brief's "assume
something in this DESIGN is wrong too" instruction.

**Did NOT** thread the counter explicitly through `alloc_counter.rs`'s signatures — out of scope per
DESIGN ("`alloc_counter.rs` is out of this target").

**STOP check:** comment-only addition; no code, no assertion, no printed value changed.

---

## Gates

```
cargo build --release
    Finished `release` profile [optimized] target(s) in 53.55s
```
Clean build, before any red was found.

```
cargo clippy --all-targets --release -- -D warnings
    Finished `release` profile [optimized] target(s) in 18.40s
RC=0
```
Clean, before the red (proves `4X1`'s ORIGINAL removal-shaped cure didn't trip clippy — the later
floor red was a citation-lint issue, not a clippy one).

**First floor attempt (the one that went RED) — captured, not discarded, per house rule:**
```
Summary [ 452.710s] 5485 tests run: 5484 passed (2 slow), 1 failed, 19 skipped
```
Full verbatim failing block, mechanism, and the fix are under `4X1` above. Not re-run before that
was all written down.

**Re-verification after the fix, before spending another full floor:**
```
cargo build --release
    Finished `release` profile [optimized] target(s) in 0.07s   (post-revert, only fanout_cost.rs recompiled)
cargo clippy --all-targets --release -- -D warnings
    Finished `release` profile [optimized] target(s) in 15.03s
RC=0
cargo nextest run --release -E 'test(every_backticked_name_in_a_rete_comment_resolves) + test(every_location_named_in_a_doc_comment_exists)'
        PASS [   0.052s] (1/2) wat::lint no_stale_path_in_doc::every_location_named_in_a_doc_comment_exists
        PASS [   0.323s] (2/2) wat::lint rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves
     Summary [   0.339s] 2 tests run: 2 passed, 5502 skipped
```

**Second (final) floor run — the one this strike stands on:**
```
./scripts/floor.sh
     Summary [ 450.881s] 5485 tests run: 5485 passed, 19 skipped
```
Read from `.floor/latest/clean.log`'s `Summary` line, never a piped exit code. **5485/5485, matching
the expected 5485 exactly.** No `.floor/latest/ARM.txt` on this run (only written on a failure) —
confirmed by directory listing, not inferred from the Summary line alone. This is the SECOND floor
run of this strike; the first is captured above as the finding it is, not erased. Nothing was
re-run before its mechanism was named, fixed, and independently verified by the two targeted gates.

## What this did NOT do

- **Did not add a lint for the `4F1` `.unwrap()`/`.expect()` convention** — rejected by DESIGN
  (`conformare`'s proposal). I independently agree it isn't warranted (see `4F1` section) but did
  not act on that opinion beyond writing it down, per the brief's instruction to leave it undone if
  I thought it was warranted, let alone since I think it is not.
- **Did not open an arc or tracker for either `4E1` or `4E2`** — DESIGN's explicit rejection; both
  are cured by rewording as present-tense architectural fact instead.
- **Did not thread `alloc_counter.rs`'s ambient counter explicitly through its call signatures**
  (`4Q1`'s "deeper fix") — out of scope per DESIGN; the rune is the target-side move.
- **Did not touch `no_unknown_sequi_rune.rs`, `rete_citation_resolves.rs`, or
  `no_stale_path_in_doc.rs` themselves** — all three were read closely (the first to resolve `4Q1`'s
  doctrine wrinkle, the other two because `4X1`'s original cure broke one of them), but changing a
  gate's own logic was never in scope for this strike and the eventual `4X1` fix works within the
  gates as they stand.
- **Did not re-run the floor a third time** — two runs total: one that found a real, attributable
  regression (captured whole, not discarded), one clean after the fix was independently verified by
  targeted gates first.
- **Did not commit `4X1` with DESIGN's stated preference (removal)** — see `4X1`'s delta section;
  removal has a proven side effect discovered only by driving the floor, so I used DESIGN's
  own-offered alternative (reason instead of removal) rather than STOPping the whole item, since a
  non-side-effecting cure for the same defect existed.

## Deltas from DESIGN, gathered in one place

1. **`4X1`**: DESIGN preferred removing `child_tax`; removal breaks
   `rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves` because the deleted
   `#[allow(unused_variables)]` was the sole surviving code-position occurrence of that lint name
   under the scanned tree. Cured instead with DESIGN's own alternative (reason string, allow kept).
   Found only by driving the real floor — not visible from reading either site.
2. **`4F1`**: DESIGN's sibling count ("460") is off by 2 against a repo-wide
   `call_beside_value(...).expect(` grep (**462**). Does not change the cure; the population the
   cure actually depends on (the ten `.unwrap()` sites being globally unique to this one file)
   re-derives exactly as DESIGN describes.
3. **`4Q1`**: DESIGN states `no_unknown_sequi_rune.rs` "gates" this site's category choice. Read
   the gate's own source: it walks `Path::new("src")` only (`tests/lint/no_unknown_sequi_rune.rs:84`).
   The `4Q1` site is under `tests/rete/`, so this specific rune is outside that gate's population —
   it would not catch a malformed category here. (A second, non-DESIGN example of the same gap
   already sits in the tree: `tests/types/probe_stone_binder_is_universal.rs:58` carries a
   category-less `rune:sequi` the same gate can't see, for the same reason.) This doesn't change
   which category to use — `ambient-context` is still the right one per `CONVENTIONS.md` — but
   DESIGN's specific enforcement claim for this exact site is false.

## Commit

(recorded after this file is committed)
