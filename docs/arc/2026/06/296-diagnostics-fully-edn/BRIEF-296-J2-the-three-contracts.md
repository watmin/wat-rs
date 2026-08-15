# BRIEF — 296 J-2: the three failing contracts

> Stone J's span carriage is **built and working** in the working tree. This brief closes the three
> tests it left red. It changes **no** `src/` behaviour.

Current: `4531 run / 4528 passed / 3 failed / 154 skipped`, clippy 0. The three reds are the whole job.

## WHAT J ACTUALLY DELIVERS — measured, and narrower than the stone reads

Measured live this session, a 3-deep child (`main → middle → inner`):

```clojure
;; REMOTE, post-J
:location #wat.kernel/Location {:file "<spawn-process-program>" :line 3 :col 34}
:frames   [#wat.kernel/Frame {:file "<spawn-process-program>" :line 5 :symbol ":user::inner"}]

;; IN-PROCESS, identical program, via ./target/release/wat
:location #wat.kernel/Location {:file "/tmp/nested_crash.wat" :line 2 :col 30}
:frames   [#wat.kernel/Frame {:file "/tmp/nested_crash.wat" :line 4 :symbol ":user::inner"}]
```

**J delivers PARITY WITH IN-PROCESS — same fields, same fidelity, same single frame.** It does not
deliver a multi-frame backtrace, because **wat does not have one anywhere**: in-process reports one
frame for the same 3-deep stack, so `:user::middle` is missing locally too.

**Frame depth is a KNOWN ISSUE and explicitly NOT this stone** (builder's ruling, 2026-08-15). Do not
try to fix it. Do not let a test's wording imply J provides a stack. Record parity as the claim.

*(An earlier version of this brief's parent showed an invented 3-frame backtrace. It was fabricated
from reading `apply_function(…, list_span)` call sites and never run. The numbers above are captured.)*

## 1 — `c02_control_ordinary_forms_stay_plain_edn`

**Keep the intent; exempt the carriage.**

It walks the whole frame collecting tags and asserts the list is empty. Its intent — its own comment —
is *"without this, 'wrap everything' would satisfy C01 while making every frame unreadable."* That
intent is about **not blanket-wrapping lexemes to dodge C01**, and span carriage does not touch it.

Change: collect tags exactly as now, then **filter out precisely the two carriage tags**
(`wat.ast/Spanned`, `wat.ast/Program`) and assert the remainder is empty.

It must still fire if anyone wraps a lexeme gratuitously — that is the whole reason it exists, and the
filter must be by exact tag name, never a prefix or a namespace-wide skip.

Update its doc: verbatim carriage is for what EDN cannot spell; span carriage is a separate declared
vocabulary; **neither licenses the other**. Say why the exemption is exactly two tags.

## 2 — `c05_the_wire_form_is_a_record_not_a_tuple`

**Same subject, one level down.**

Its subject is *a scoped symbol crosses as a `Tagged` whose body is a record, not a tuple* — untouched
by this stone. Only its reach changed: the frame is now
`#wat.ast/Program {:origins […] :forms […]}` rather than a bare `Vector`.

Change: destructure the `#wat.ast/Program` record and take `:forms`, then apply its existing
assertions to that vector. Its name stays accurate; its claim does not weaken.

**Do not** relax it to "the frame is any Tagged". It should still fail if the frame stops being the
Program record.

## 3 — `select_prime_yields_lost_when_process_child_crashes` — ARM THE CLAIM, THEN RECAPTURE

This golden was the **oracle**: it held the correct `:location` from before the regression, and J's fix
now matches it byte-for-byte. It has done its job.

It is also stale on **four axes that predate this stone**: the `ProcessDiedError`→`LociDiedError`
rename, arc-278's `Fault` nesting, Option-unwrapping of `:location`/`Frame` fields, and a `freeze.rs`
line drift (1014→1441). Those are landed history, not regressions.

⛔ **Recapturing it wholesale would dissolve the property it was holding.** Nothing would then notice
if the span regressed again — the golden would simply re-capture the lie.

So, in this order:

1. **Add a standing assertion** to the test, independent of the golden: `:location`'s `:file` is the
   child program's own file and is **never** a `src/*.rs` path. Assert structurally on the parsed EDN,
   not with `.contains()` — `no_loose_string_assert` is armed and has fired on this arc twice.
2. **Then** recapture the golden with `UPDATE_EDN=1`.
3. **Then prove the assertion can fail**: temporarily break the span carriage (the rider before you
   used `if false && carriage == Carriage::Transport`), confirm the new assertion goes red, revert,
   confirm green. Report how you verified it.

Same move as G-2's `#usr/Point` golden: the subject survives, only the expected value moves — except
here the subject gets promoted out of the golden into an assertion that a recapture cannot silently
erase.

## STOP TRIGGERS

- **STOP-1 — the c02 exemption needs to be broader than two exact tag names.** If some third tag
  appears in an ordinary frame, that is a finding: something else is wrapping. Report it; do not widen
  the filter to make it pass.
- **STOP-2 — the recaptured oracle differs on an axis you cannot attribute** to one of the four stale
  causes named above. An unexplained delta is a second effect. Capture it and report; do not accept it.
- **STOP-3 — the new assertion cannot be made to fail.** Then it proves nothing and the strike has not
  landed (`[[feedback_a_green_test_can_prove_nothing]]`).
- **STOP-4 — any `src/` change looks necessary.** J's carriage is done. This brief is tests only. If a
  test cannot be satisfied without touching `src/`, that is a finding about the carriage, not a licence.

## BLAST RADIUS

`tests/program/probe_arc170_edn_bridge_unspellable.rs`, `tests/program/probe_arc170_edn_bridge_hygiene.rs`,
`tests/process/probe_supervisor_select_lost.rs` and its golden. **No `src/` changes. No `.wat` corpus
changes.** Do not touch Wave A's 105 recaptured goldens or its lifted ignores, also uncommitted here.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.

Expect **`3 failed` → `0 failed`** at `4531` run. Report the arithmetic.

**On any red you did not intend: do NOT re-run.** Copy the failing test's whole stdout+stderr block
verbatim — never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the FOREGROUND and block on it; a rider on this arc already lost a flight to
exactly that. Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. Leave the work uncommitted.

Report: each contract's change and why it preserves the original subject, how you verified the new
oracle assertion can fail, the floor Summary line verbatim with the arithmetic, every STOP, and the
honest deltas — especially anywhere this brief did not match the disk.
