# DESIGN — the died-error cluster is FOUR vocabularies, and three of them have homes

> Map item 4 of `[[NOTE-partire-RECAST-on-the-current-runtime]]`, the one the recast **deliberately
> refused to assign**:
>
> > *"~55 items. ⬜ HOME DELIBERATELY UNASSIGNED — consumed by kernel, process, distribution AND
> > host. Calling it 'kernel' repeats the `peer_protocol` mistake."*
>
> **That refusal was correct, and it is now partly obsolete.** Both halves are measured below.

## ★★ Why the verdict moved: the campaign changed its own subject

The recast measured the cluster as ONE thing with a union consumer set, and a union across 55 items
is unassignable almost by construction. Measured **per item**, with calls separated from doc
mentions, it is four vocabularies with four different confinements — and one of them was confined
**by stone B, after the recast was written**:

```
loci_died_error_from_reason · loci_died_from_send_error · loci_died_disconnected
   recast (2026-09-01, pre-B):  kernel, process, distribution, host   -> unassignable
   measured now (post-B):       src/kernel/{outcome,message}.rs ONLY  -> confined
```

Every other reference is a doc comment (`process/verbs.rs:71`, `kernel/spawn.rs:786`, a probe's
header). Stones A and B pulled the callers into `src/kernel/`, so the vocabulary they call is now
kernel's by measurement rather than by assertion. ★ `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`
— the note was true when written; re-measuring it is the work, not second-guessing the recast.

## ⛔ And the largest family's home was already decided — by the rule stone B shipped

Stone B pinned: **one `src/intrinsic/kernel/<x>.rs` edge file, one `src/kernel/<x>.rs` impl module.**
`src/intrinsic/kernel/error.rs` **is an edge file** — it delegates four verbs into `runtime.rs` and
its header says so. I skipped it when drawing the kernel family because the *map* said item 4's home
was unassigned, and I took that as covering the edge too.

★ It does not. The map's refusal was about a 55-item union; the edge names four verbs, and the rule
answers them. **`src/kernel/error.rs` is the eighth module of a seven-module stone** — and the reason
I missed it is worth stating plainly: I let a map's verdict override a rule I had derived, measured,
and shipped two commits earlier. `[[feedback_i_cited_a_rule_instead_of_measuring_whether_it_applied]]`

## The decomposition — measured per item, calls separated from mentions

| # | destination | items | lines | basis |
|---|---|---:|---:|---|
| **4a** | `src/kernel/error.rs` | 14 (+5 chain helpers) | ~581 | the edge file; one-edge-one-module |
| **4b** | `src/process/died.rs` | **10** | ~130 | callers `process/verbs.rs` ×9, `distribution/mod.rs` ×5 |
| **4c** | `src/freeze/stop.rs` | **8** | ~105 | callers `freeze.rs` ×5, `distribution/mod.rs` ×3 |
| **4d** | ⬜ **STILL UNASSIGNED** | 12 | ~316 | the genuine residue — see below |

**4a** `eval_died_error_message` · `eval_died_error_to_failure` · `eval_failure_message` ·
`eval_failure_location` · `loci_died_error_from_reason` · `loci_died_from_send_error` ·
`loci_died_disconnected` · `thread_died_error_{panic,runtime,shutdown}` ·
`died_error_payload_message` · `edn_is_loci_died_chain` · `failure_error_field` · `eval_error_names`
— plus the chain helpers `single_died_chain` · `conj_died_chain` · `conj_died_chain_value` ·
`thread_crash_panic_edn` · `thread_crash_runtime_edn`.

**4b** `process_died_error_{bad_return,main_signature,panic,runtime}` and their four `*_value`
siblings.  **4c** `stop_failure_{value,from_panic,names}` · `stop_failed_names` ·
`publish_stop_failures` · `take_stop_failures`, plus `stop_failed_value` (686).

⚠ **The five chain helpers and the two `*_value` stragglers are listed because a short list is the
one mistake this campaign keeps paying for.** They surfaced only from a cross-family caller scan,
exactly as `reply_failed_reason` and the seven `*_OUTCOME_TYPE` consts did. Each is measured:
`single_died_chain` and `conj_died_chain` are runtime-internal; `conj_died_chain_value` → `process`;
`thread_crash_{panic,runtime}_edn` → `kernel`; `stop_failed_value` → `distribution`.

## ⚠ AMENDED before briefing — the both-halves scan changed the lists

Running the visibility scan this DESIGN commits to, and then **ruling on each hit**, converted three
"stays-side bumps" into two members and one bump:

- **`STOP_FAILURES_PTR` (703) is a MEMBER of 4c, not a reach-back.** Only `publish_stop_failures` and
  `take_stop_failures` touch it — it is their state. ★ The instrument can find what the movers depend
  on; it cannot tell a *missed member* from a *legitimate reach-back*. **That is the practitioner's
  call, and it is the exact call the numeric stone got wrong.**
- **`conj_died_chain` (10214) is a MEMBER of 4b.** Its only caller is `conj_died_chain_value`. My
  cross-family scan put them on opposite sides because it **did not strip comments** and read a
  doc-comment mention as a call — the identical defect stone B's rider reported to me about
  `try_match_pattern`, which I recorded and did not fix.
  `[[feedback_a_lesson_learned_and_then_dropped]]`
- **`failure_value_from_assertion_payload` (9861) is the one genuine bump** — private today, 4d
  residue, called by 4b's `process_died_error_panic`. Its three "external consumers" are all doc
  mentions.

**Half 2 (imports orphaned by departure): none.** Derived, not assumed.

## ⬜ 4d — what stays unassigned, and why that is an answer

`fault_value` · `fault_names` · `fault_with_cause` · `fault_from_runtime_error` ·
`fault_from_panic_payload` · `failure_names` · `location_names` ·
`failure_value_from_assertion_payload` · `check_failed_cause` · `frame_names` ·
`format_panic_payload` · `value_from_frame_info`

Consumers: `edn`, `host`, `types`, `resolve`, `assertion`, `comms`, `kernel`, `distribution`. **This
is the `:wat::core::Fault`/`Failure` diagnostic vocabulary — the substrate's error language, not any
one home's.** Its union is irreducible because the thing genuinely is shared, which is precisely what
the recast's refusal was protecting. Naming it `kernel` would repeat `peer_protocol`; naming it
`edn` would confuse a wire format with a value language.

**Out of scope = REJECTED for this design.** 4d gets a home when something forces the question — a
crate boundary is the likeliest forcing function, and the crate migration is the builder's next
sequenced phase. Recorded here so the next self does not re-derive the same four families to reach
the same undecided twelve.

## THE ONE CONTRACT DECISION — pinned

**A vocabulary is admitted to a home when its CALL sites — not its mentions — are confined to that
home or its importers. Confinement is re-measured per stone, because the campaign moves callers.**

This is the rule stone A used (the outcome vocabulary looked unassignable and had exactly one home)
and it is what separates 4a/4b/4c from 4d. It also states the thing that made item 4 look harder
than it is: **confinement is a property of the tree at a moment, and this campaign changes the tree.**

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **4a → `kernel/error`, 4b → `process`, 4c → `freeze`, 4d stays** | YES | YES | YES | YES | ✅ **ADMITTED** |
| all 40 into one `src/error/` home | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| all 40 into `src/kernel/` (the map's named trap) | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| 4a only; leave 4b/4c with item 4's old verdict | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| assign 4d to `src/edn/` for completeness | **NO** | YES | **NO** | — | ⛔ DISQUALIFIED |

- **one-`src/error/` Honest? NO** — it fuses four vocabularies with four consumer sets on the
  strength of a shared name-prefix. That is the `peer_protocol` mistake with a neutral label.
- **all-into-kernel Honest? NO** — measured: `process_died_error_*` has zero kernel callers and
  `stop_failure*` has zero. The recast named this trap by name.
- **4a-only Honest? NO** — the same measurement that homes 4a homes 4b and 4c; running it and then
  shipping a third of it is `[[feedback_a_lesson_learned_and_then_dropped]]`.
- **4d-to-edn Obvious? NO, Honest? NO** — a Fault is a value, not a wire encoding; `edn` is one of
  eight consumers, not the owner. An assignment made for tidiness is the unearned kind.

## Sequence — three stones, smallest blast radius first

**4c** (6+1 items, two callers) → **4b** (8+1 items, two callers) → **4a** (14+5 items, the edge).
4a is last because it is the largest and because 4b/4c prove the confinement rule on cheap cases
first. None of the three depends on another; the order is by risk, not by need.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| 4c: the home holds it | `grep -c "^pub(crate) fn " src/freeze/stop.rs` | **7** |
| 4b: the home holds it | `grep -c "^pub(crate) fn " src/process/died.rs` | **9** |
| 4a: one edge, one module | `ls src/kernel/error.rs` + `grep -c "crate::runtime::eval_" src/intrinsic/kernel/error.rs` | exists, **0** |
| ⛔ the residue was NOT swept in | `grep -c "fn fault_value\|fn fault_with_cause\|fn check_failed_cause\|fn failure_names" src/runtime.rs` | **4** |
| ⛔ the intruder fence | `grep -c "fn no_field_names\|fn builtin_enum_variant_names" src/runtime.rs` | **2** |
| ⛔ the spine | `grep -c "^pub(crate) fn eval_tail\|^pub(crate) fn eval_inner\|^pub fn eval\b" src/runtime.rs` | **3** |
| bodies verbatim | diff each moved item vs `git show HEAD:src/runtime.rs` | byte-identical |
| runtime.rs | `wc -l` | 19,914 → **~19,000** |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

⚠ **Every brief in this sequence carries BOTH halves of the visibility class**, derived per stone:
the stays-side items needing `pub(crate)`, *and* the imports left orphaned by departing items. Stone
B proved deriving one half eliminates one half. `[[feedback_a_patch_fixes_one_copy_of_a_claim]]`
