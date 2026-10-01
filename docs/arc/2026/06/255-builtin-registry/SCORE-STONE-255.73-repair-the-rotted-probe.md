# SCORE — STONE 255.73: repair the rotted probe, and make it run

**Executor: a Sonnet subagent.** Branch `main`, drawn against `e84a58b14`, struck at `ee5ae9d0e`.
Not pushed.

## 1. The probe's claim (one sentence)

`wat-scripts/probes/arc-170/probe-s3b-astsplice.wat` proves the 259 S3b Blocker A "derive-and-splice"
mechanism — that a work-fn's concrete argument/return types, reflected off its `fn-forms` output via
an AST walk, can be spliced into a shipped process-runner's `self-peer`/`Peer` tuple types at expand
time, so a generic runner ships as a concretely-typed child program that actually spawns and round-trips
(`EXPECT "6 10"`), per `docs/arc/2026/06/259-forced-hand/NOTE-S3b-blockers-and-resolution.md`'s "Blocker
A" resolution.

## 2. What was repaired

Confirmed the 255.72 finding first, on the up-to-date release binary (`cargo build --release --bin wat`,
`Finished … in 22.26s`, HEAD `ee5ae9d0e` with `e84a58b14` as an ancestor):

```
./target/release/wat wat-scripts/probes/arc-170/probe-s3b-astsplice.wat
[#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/MalformedForm {:message \"malformed :wat::core::keyword-node form: angle-bracket type parameters are illegal in a name (arc 109, \"annihilate the angle bracket\") … :wat::kernel::Peer<(wat::core::i64,wat::core::i64),(wat::core::i64,wat::core::i64)> …\" :location … :line 59 :col 16 … :line 63 :col 61 … :head \":wat::core::keyword-node\" …}"}]
rc=1
```

**Site 1 (lines 53–65, old numbering):** `peer-node`/`sp1-node`/`sp2-node` built a `Peer<(i64,ARG),
(i64,RET)>`-shaped keyword by `ast-name` + string-concat + `keyword-node` — the exact angle-bracket
spelling arc 109 ("annihilate the angle bracket") retired. Repaired by minting the reference FORM
`(Head :- [args])` structurally off the raw type-position AST nodes already in scope (`arg-ty`/`ret-ty`,
bound a few lines above) — no `ast-name`/string round-trip at all, the same precedent as
`wat/bracket.wat:422/538` (`sp-out`/`sp-in`) and 255.72's own typed site 1:

```
sp1-node  `(wat.type/Tuple :- [wat.type/i64 ~ret-ty])   ;; S — send type
sp2-node  `(wat.type/Tuple :- [wat.type/i64 ~arg-ty])   ;; R — recv type
peer-node `(:wat::kernel::Peer :- [~sp1-node ~sp2-node])
```

`sp1-node`/`sp2-node` keep their original send-then-recv argument order into `self-peer`
(`wat/bracket.wat:425`'s `runner-self-kw = (Peer :- [~sp-out ~sp-in])` convention: slot 1 = the type
`prn` *sends* — `out`'s `(i64, ret-ty)` — slot 2 = the type `prn` *receives* — `pair`'s `(i64, arg-ty)`).
The ORIGINAL string-built `peer-node` had ARG first / RET second — the opposite order from its own
`sp1-node`/`sp2-node` pair passed to `self-peer` two lines below it (RET first / ARG second) — an
internal inconsistency that could never have been caught, because execution always died inside
`peer-node`'s own construction before either form was ever type-checked. The repair makes `peer-node`
structurally equal to `(Peer :- [sp1-node sp2-node])`, so it is self-consistent with the `self-peer`
call by construction, not by a second hand-matched order.

**Site 2 — a second, previously-unreached defect the repair exposed.** Once site 1 no longer raised
at expand time, the shipped `:probe::__runner` form reached the child's own startup type-checker for
the first time ever, which found 3 type-check errors:

```
#wat.check/TypeMismatch {:message ":wat::core::first: parameter #1 expects tuple, (Vector :- [T]), (List :- [T]), (PersistentVector :- [T]), or WatAST; got (:wat::kernel::RecvOutcome :- [:(wat::core::i64,wat::core::i64)])" … :line 77 :col 89 …}
#wat.check/TypeMismatch {:message ":wat::core::second: parameter #1 expects tuple, … got (:wat::kernel::RecvOutcome :- [:(wat::core::i64,wat::core::i64)])" … :line 78 :col 83 …}
#wat.check/MalformedForm {:message "malformed :wat::core::match form: non-exhaustive: enum :wat::kernel::SendOutcome missing arm(s) for variant(s): Stopped (or include `_` wildcard)" … :line 79 :col 29 …}
```

`__runner`'s body bare-bound `(:wat::kernel::recv prn)` as if it returned the payload directly; `recv`
actually returns `(RecvOutcome :- [T])`, an enum that must be matched (exactly as `:probe::drain`,
three lines above in the same file, and `wat/bracket.wat`'s `dial-runner` already do). The `send` match
was also missing the `SendOutcome.Stopped` arm. Repaired by restructuring `__runner`'s body as a
top-level `match` on the `RecvOutcome` (Message processes-and-recurses; Lost raises; Stopped/Closed
exit) rather than a `let`-bound unwrap — a `let`-bound unwrap would force every match arm to the same
result type (the Message arm's tuple vs. Stopped/Closed's `nil`), which is itself a different type
error; the restructured shape mirrors `wat/bracket.wat`'s `dial-runner` (lines 482–506) exactly — and
added the missing `SendOutcome.Stopped` arm.

Also updated the file's header comment (previously still said "via keyword-node + quasiquote") to
name today's mechanism and the arc 109/255.73 history, so the probe's own prose matches its code.

**Result** — the probe now runs clean and prints its documented claim:

```
./target/release/wat wat-scripts/probes/arc-170/probe-s3b-astsplice.wat
"6 10"
rc=0
```

No STOP fired: the mechanism the probe proves (derive-and-splice off `fn-forms`) is unchanged and
still demonstrable in today's language; only the now-illegal name-building spelling, and the
previously-unreached `recv`/`send` outcome handling, needed repair.

## 3. The new test

`tests/process/probe_arc170_s3b_astsplice.rs` (new file; `tests/process/` auto-registers via
`build.rs`'s `process_mods.rs` generation, same convention as every other file in that directory).
Runs the compiled binary against the probe's real path
(`wat-scripts/probes/arc-170/probe-s3b-astsplice.wat`, relative to `CARGO_MANIFEST_DIR`, the same
`Command::new(env!("CARGO_BIN_EXE_wat"))` shape as `tests/kernel/probe_arc255_29_thread_address.rs`),
asserts exit code 0, and asserts stdout is exactly `"6 10"\n` (the probe's own documented `EXPECT`).
A future rot of this exact file — the loader gate's blind spot 255.72 found — is now a floor RED
instead of invisible.

```
cargo test --release --test process probe_s3b_astsplice
running 1 test
test probe_arc170_s3b_astsplice::probe_s3b_astsplice_runs_and_prints_6_10 ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out; finished in 0.74s
```

## 4. Sibling-probe census (report only — nothing else changed)

`git ls-files 'wat-scripts/probes/**/*.wat'` → **83** files tracked.

**Never run by any test** (basename not referenced by any `tests/**/*.rs` in a way that executes the
file's *current* content): **82 of 83** — every file except this stone's own
`probe-s3b-astsplice.wat` (now run by the new test above). One near-miss, noted for precision:
`probe-m1-ann-erase.wat`'s *basename* appears in `tests/lint/nested_program_starts.rs`, but only as
`git show f2e0ac26b^:wat-scripts/probes/arc-170/probe-m1-ann-erase.wat` — a **retired historical
blob**, parsed in-memory as a negative-control fixture (asserted to go RED). That test never executes
the file at its current path/content, so it is counted among the 82 never-run.

**Of those 82, run individually with `./target/release/wat <probe>` (10s timeout, `</dev/null`,
2026-10-01, same binary as above): 21 crash (non-zero exit); 61 exit 0.**

| probe | rc | first error |
|---|---|---|
| `arc-170/probe-bracket-cause.wat` | 2 | `bracket collect-loop: runner … crashed: … #wat.runtime/DivisionByZero {:message "division by zero" …}` at probe line 5 col 19 |
| `arc-170/probe-c1-ast-shape.wat` | 1 | `malformed :wat::core::ast-name form: ast-name requires a Symbol, Keyword, or StringLit node` at line 37 col 15 |
| `arc-170/probe-c1-plain-fnforms-shape.wat` | 1 | same `ast-name` MalformedForm, at line 30 col 16 |
| `arc-170/probe-cap2-peer-pid.wat` | 1 | prints `"process-peer peer-pid:"`, then `:wat::kernel::peer-pid: expected peer ((Thread :- [I O]) \| (Process :- [I O])), got :wat::kernel::Peer` at line 29 col 33 |
| `arc-170/probe-child-inherits-defns.wat` | 2 | StartupError `UnresolvedReferences`: `:probe::dbl` — "call head — not a builtin, not a registered function", line 24 col 67 |
| `arc-170/probe-compound-upcast.wat` | 2 | **Rust-level panic**, not a wat-level error: `internal error: entered unreachable code: Value::RustOpaque is not atomizable; is_atomizable predicate in src/check.rs should have rejected this. If you see this panic, the predicate has drifted.` (`src/value/value.rs:913:37`) |
| `arc-170/probe-defclause-discriminate.wat` | 1 | `NoMatchingClause`: `no clause of :probe::describe matched (1 args); called with (wat::core::Record <probe::MongoReason{...}>)` at line 36 col 9 |
| `arc-170/probe-fnforms-keyword-err.wat` | 1 | `:wat::kernel::fn-forms: expected a keyword naming a registered fn … got :no::such::fn` at line 3 col 36 — the `-err` suffix and the deliberately-bogus keyword suggest this is a negative-control fixture, not rot (unverified beyond the name/content) |
| `arc-170/probe-generic-shipped.wat` | 2 | StartupError, 5 type-check errors — **same defect class as this stone's site 2**: `:wat::core::first` given `(RecvOutcome :- […])` instead of the unwrapped tuple, plus non-exhaustive `SendOutcome` match (missing `Stopped`), line 32 |
| `arc-170/probe-m1-addr-roundtrip.wat` | 1 | prints an `Address` wire line, then `:wat::edn::read … unsupported substrate tag #wat.kernel/Address (capability tags reconstruct only off the trusted peer wire, never from parsed data)` at line 45 col 35 |
| `arc-170/probe-m1-argcount.wat` | 1 | prints `3`, then the same `ast-name` MalformedForm as above, at line 22 col 15 |
| `arc-170/probe-m1-cf-norevoke.wat` | 2 | `AssertionFailure`: `recv': prober closed unexpectedly` |
| `arc-170/probe-m1-fix-revoke.wat` | 2 | `AssertionFailure`: `recv': peer closed` |
| `arc-170/probe-s1-fn-forms.wat` | 2 | StartupError, 2 type-check errors — **same defect class as site 2**: non-exhaustive `SendOutcome.Stopped` + `:probe::work` given a bare `(RecvOutcome :- […])`, line 26 |
| `arc-170/probe-s1-impure-gate.wat` | 1 | `:wat::kernel::fn-forms form: … closure-extract internal: encoding for captured Value of kind :wat::kernel::Process not implemented in slice 1` at line 7 col 9 |
| `arc-170/probe-s1-named.wat` | 2 | StartupError, 1 type-check error — **same defect class as site 2**: `:probe::work` given a bare `(RecvOutcome :- […])`, line 9 col 123 |
| `arc-170/probe-s3-process-runner.wat` | 2 | StartupError, 3 type-check errors — **same defect class as site 2** (`:wat::core::first`/`second` on a bare `RecvOutcome` + non-exhaustive `SendOutcome.Stopped`), line 53 |
| `arc-170/probe-s3b-crux-fnforms-closure.wat` | 1 | `:wat::kernel::fn-forms form: … encoding for captured Value of kind wat::core::fn not implemented in slice 1` at line 50 col 12 — `docs/arc/2026/06/259-forced-hand/NOTE-S3b-blockers-and-resolution.md` names this exact probe as the **documented disconfirming probe, "RED, proved it"** — this crash is almost certainly BY DESIGN, not rot (unverified beyond that doc's text) |
| `arc-170/probe-s3c-rendezvous.wat` | 2 | StartupError, 3 type-check errors — **same defect class as site 2**, line 65 |
| `arc-170/w3-n-dial-runner.wat` | 4 | `MainSignatureError`: `:user::main not defined — a wat program needs an entry point` — this file is likely a library/template fragment never meant to run standalone, a different failure mode than the others in this table (unverified beyond the error itself) |
| `arc-278/s2s-revoke-probe.wat` | 2 | prints `"echo:hi"` then `"revoke-midlife-ok"`, then `Panic`: `disconnected` (`src/freeze.rs:1542`) |

**Measurement for the builder's follow-up:** 5 of these 21 (`probe-generic-shipped.wat`,
`probe-s1-fn-forms.wat`, `probe-s1-named.wat`, `probe-s3-process-runner.wat`,
`probe-s3c-rendezvous.wat`) share the **identical** defect class this stone's site 2 repaired — a
bare-bound `(RecvOutcome :- […])` where a tuple was expected, plus a non-exhaustive `SendOutcome`
match missing `Stopped` — strongly suggesting they were copied from the same pre-fix template as
`probe-s3b-astsplice.wat`. `probe-compound-upcast.wat`'s crash is a **Rust-level panic** naming its own
drifted invariant, not a retired-syntax issue, and is likely the most urgent of the 21 to triage next.

## Gates

| what | how | result |
|---|---|---|
| the probe runs | `./target/release/wat wat-scripts/probes/arc-170/probe-s3b-astsplice.wat` | `"6 10"` / rc 0 (quoted above) |
| the probe runs | the new test, `cargo test --release --test process probe_s3b_astsplice` | `test probe_arc170_s3b_astsplice::probe_s3b_astsplice_runs_and_prints_6_10 ... ok` (quoted above) |
| release floor | `scripts/floor.sh`, sole run, foreground, nothing else running | `.floor/2026-10-01T01-21-53Z/clean.log`: `Summary [ 385.524s] 6236 tests run: 6236 passed (27 slow), 24 skipped` — 6235 (the 4969e1907 baseline the brief names) + this stone's 1 new test; exit=0 |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | `Finished `release` profile [optimized] target(s) in 11.87s`, rc 0 |

No reds. The floor ran once, green, with nothing else running concurrently; no re-run was performed or
needed.

## Files touched

- `wat-scripts/probes/arc-170/probe-s3b-astsplice.wat` — repaired in place (hand-edit; this is one
  file's repair, not a corpus-wide migration, per the brief's own doctrine note).
- `tests/process/probe_arc170_s3b_astsplice.rs` — new, wires the probe into the floor.
- `docs/arc/2026/06/255-builtin-registry/SCORE-STONE-255.73-repair-the-rotted-probe.md` — this file.

## STOPs triggered

None. STOP-1 did not fire: the probe's claim (the derive-and-splice mechanism, `fn-forms` → AST-walk
→ type-form splice → spawn + drain) still exists and still means the same thing in today's language;
only the angle-bracket name-building spelling and a previously-unreached `recv`/`send` outcome-handling
gap needed repair, and the repaired probe reproduces the exact documented output.
