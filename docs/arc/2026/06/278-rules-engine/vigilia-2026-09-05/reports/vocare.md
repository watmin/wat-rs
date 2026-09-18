# ward `vocare` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I have what I need. Report follows.

---

## L1 — defects

### L1‑1. The D2 cure's acceptance test is a tautology, and the caller‑visible symptom has no observer at all

**Sites read this session:**
- `src/rete/kernel/session.rs:243-247` — `JoinRightIndex { buckets, indexed_n }`, both private.
- `src/rete/kernel/session.rs:261-264` — `RightIndexWriter::push`: `self.buckets.entry(key).or_default().push(el); *self.indexed_n += 1;`
- `src/rete/kernel/session.rs:279-284` — `writer()`, the only `&mut` door; `get()` at `:288` is shared.
- `src/rete/kernel/tests/right_index_counter_invariant.rs:433` `right_index_counter_tracks_its_bucket_population`, assertion at `:455` (`if mark != elements { violations.push(…) }`).

**Mechanism.** Every element in `buckets` arrived through `push`, which advances `indexed_n` in the same statement; the fields are private to `session.rs`, nothing else in that file mutates them, and no other module can. Therefore `indexed_n[J] == Σ|buckets[J]|` holds **by construction**. The assertion has one possible outcome. The commit message concedes the shape without drawing the conclusion: *"The rider had to ADD an escape hatch to run mutation 2 at all."* The committed test has no mutation that reddens it — `[[derive what your own measurement implies]]`.

**Why that is not merely redundant.** The proposition correctness actually needs is a *different* one, stated as an assumption at `session.rs:238-240`: *"the mark is also the length of the alpha prefix already indexed — which is the reading `already` needs."* `keyed_join_persistent` (`src/rete/kernel/fire/mod.rs:809-827`) reads `already` and slices `right_elements[already..]`; the two bypass sites push one element per alpha element (`fire/pass/hash_join.rs:186-202`) and one per `dr` slot (`:314-326`). If those three ever stop exactly tiling the alpha memory in order, `already` becomes a wrong offset — re‑pushing (doubled buckets, doubled join output) or skipping (dropped rows) — and in **both** cases mark and population move together, so the invariant stays green. The acceptance test is blind to the only hazard that remains.

**And the public surface is unobserved.** D2's caller‑visible symptom was a chain‑mirroring query returning **18 rows where the spec says 12**. The only artefact that observes it is `wat-scripts/scratch-pad/d2-derived-fact-axis.wat:69-76` (`:d2p::q-chain`), and `tests/lint/wat_scripts_fixes_load.rs:34` only *parses and type‑checks* every `wat-scripts/` file — its own header, `:12`: *"without running `main`"*. Grepped `tests/`, `src/`, `wat/` for `q-chain`, `chain-rows`, `d2p::`: **zero hits.** The port check cannot see it either (`tests/rete/wat_scripts_grid_port_check.rs:45-52` compares `:derived` vs `:oracle-derived`, both deduped sets), which the commit itself records as a mis‑scoped follow‑up.

**Remedy (idiom already in the tree).** `tests/rete/probe_arc278_leading_filter_multiplicity.rs` is the precedent for a multiplicity defect caught through a query, and `tests/rete/probe_arc278_query_harvest_protocol.wat:22` shows the observable: `(:wat::core::length (:wat::rete::query fired (…)))`. Port the two‑wave stagger world plus `q-chain` into a `tests/rete/probe_*.rs` + `.wat` pair and assert 12. That test reddens on a doubled bucket regardless of which writer produced it; the counter test cannot.

### L1‑2. Nine tests that run on the floor document themselves as *not* on the floor

Six files, **zero `#[ignore]` attributes among them**, nine plain `#[test]`s, none named in `.config/nextest.toml`'s `default-filter` (which names exactly five tests, all unrelated):

| file | claim lines | `#[test]` lines |
|---|---|---|
| `tests/kernel/spawn_program_prime_process.rs` | 25, 28, 163, 220, 273 | 169, 221, 274 |
| `tests/kernel/peer_process_round_trip.rs` | 53, 57 | 58 |
| `tests/kernel/peer_select_prime_process.rs` | 18, 19 | 33 |
| `tests/kernel/peer_verb_round_trip_process.rs` | 17, 18 | 31 |
| `tests/kernel/probe_arc214_beta_forms_server.rs` | 32, 40 | 41 |
| `tests/kernel/probe_arc278_close_outcome_wall.rs` | 107‑111 | 89, 112 |

The sharpest is `probe_arc278_close_outcome_wall.rs:109`: **"Not part of the default floor."** It is.

**Consequences, three:**
1. The printed instruction — e.g. `spawn_program_prime_process.rs:28`, `cargo test --test kernel spawn_program_prime_process -- --ignored` — selects **zero** tests and prints a pass. `[[a check can report success without running]]`.
2. A standing dismissal licence over exactly the fork/pdeathsig/lifeline family `wat-rs/CLAUDE.md` struck ("A RED IS A RED"). A red at `peer_process_round_trip.rs:58` reads, from its own doc, as an on‑demand integration probe.
3. `.config/nextest.toml` says exclusions live in `default-filter` — *"the structural exclusion mechanism `#[ignore]` used to fake with a greppable … string"*. These six files are that fake, still standing.

---

## L2 — weaknesses

### L2‑1. "EXPECTED RED" banners standing over green tests — six files, three of them each claiming to be *the* one

- `tests/lint/no_inlined_wat_in_tests.rs:23` — "Until then this is the ONE expected-red test; nextest isolates it, so a SECOND red is a real regression."
- `tests/lint/no_loose_string_assert.rs:14-15` — same sentence, same claim.
- `tests/lint/no_inlined_edn.rs:80-82` — "This is an EXPECTED-RED test until a follow-up fleet drives it to zero."
- `tests/rete/probe_arc278_import_fold_key.rs:26` — "⚠ EXPECTED RED before Class A2 lands" (test `:33`).
- `tests/rete/probe_arc278_export.rs:640` — "expected RED" (test `:651`).
- `tests/rete/probe_arc278_enum_variant_typo.rs:49` (test `:51`) and `:67` (test `:69`) — "EXPECTED RED until Class D1 lands."

All are plain `#[test]`, zero `#[ignore]`, none in `default-filter`; at 5420/5420 with 0 FAIL every one of them is green. Three simultaneous claims of being "the ONE" falsify each other on their face. The sign is inverted: these are now the *acceptance* tests for the classes that landed, and their headers tell the next reader a red is expected.

**Remedy — the rung the repo already built.** `tests/lint/rete_header_claims_are_asserted.rs` exists precisely because "an assertion no gate can check rots undetected by construction." The same rung applies here: a walk over `tests/**/*.rs` requiring that any `#[test]` whose doc or module header carries `#[ignore]` / `--ignored` / `EXPECTED RED` / "not part of the default floor" either carries the attribute or is named in `default-filter`. That single gate closes L1‑2 and L2‑1 together.

### L2‑2. The destroyed `None`‑means‑maintainer inference, corrected in one place and left standing in two

The D2 commit records striking the inference "`indexed_n[J].is_some()` names the maintainer" and re‑based `maintained_joins` (`right_index_counter_invariant.rs:283-295`) on the census row. Two copies of the struck inference survive in code files:

- `src/rete/kernel/census.rs:64-66` — "`None` in the middle column means `indexed_n` has NO entry for that join: **the maintainer has never run on it**." False since the cure: `writer()` (`session.rs:280-283`) creates the `indexed_n` entry alongside the bucket entry for *every* writer, so the two key sets `per_join_marks` (`session.rs:311`) unions can never differ — the `Option` is always `Some`.
- `src/rete/kernel/tests/right_index_counter_invariant.rs:450-453` — "No mark: the maintainer has never visited this index," directly above `let Some(mark) = mark else { continue };`. An unreachable skip arm carrying the exact inference the same file corrected 170 lines above.

**Remedy:** either collapse the `Option` to `usize` (the honest type post‑cure) or restate both comments as "absent iff nothing opened the writer, which no path can now produce." Leaving it as an `Option` with a false explanation is `[[an accurate comment can be a defect's alibi]]` one rung down.

---

## L3 — judgement

- **The `tests/` → engine boundary is genuinely clean.** Complete grep over every `use wat::` line in `tests/`: **no** test file imports `wat::rete::*`. The prior vocare cast's four flagged join tests now carry `rune:vocare(vantage-bypass-test)` (`src/rete/kernel/tests/pass_semantics.rs:233, 334, 451, 524`) and the coverage hole they left was closed at the caller's vantage by `tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs.rs`. This ward's earlier rows are closed, and the closure was done properly.
- **The `src/`‑location exemption is wider than the argument that earns it.** `tests/lint/no_inlined_wat_in_tests.rs:383` scans `tests/` only, so `src/rete/kernel/tests/*.rs` is exempt by scan scope — and those files are not unit tests of a Rust module: they freeze and fire whole wat worlds hand‑built as Rust string literals (`D2_WORLD`, ~90 lines at `right_index_counter_invariant.rs:88-178`). vocare's "unit tests in the host source tree" exemption is being claimed by file location while the content is a consumer‑vantage program. `census.rs:20-24` gives a real reason the census cannot be read from wat — but that reason licenses reading the census, not building the *world* by hand where the `.wat`‑fixture gate cannot reach it.
- **The counter test's own reasoning is the best thing in the file and stops one step short.** Its three ascending reach assertions (`:332-400`) refuse the vacuous partition, and the mutation‑3 control (`:487`) drives the guard rather than describing it. What none of that reaches is whether the *asserted equality itself* can still be false. That question is one rung above non‑vacuity and this file never asks it.

---

## What I could not check, and why

- **I ran nothing.** Read‑only, no builds, no floor. Every "this test is green" claim rests on the brief's stated 5420/5420 · 0 FAIL, plus two things I did verify by reading: no `#[ignore]` attribute in the file, and no matching entry in `.config/nextest.toml`'s `default-filter`. If any of the nine kernel tests or the six "expected red" tests is in fact red, L1‑2 and L2‑1 invert — the docs would be right and the floor count wrong. Either way something on that list is false; I cannot tell you which half from here.
- **I did not trace `dr`'s construction** in `fire/pass/hash_join.rs`, so I make no claim that the `already`‑as‑alpha‑prefix assumption is *currently* violable. L1‑1 is a claim about what the acceptance test can detect, not that a live bug exists behind it.
- **I did not read all 743 test files.** The "no test imports `wat::rete`" result is a complete grep over `use wat::` lines and I stand behind it; a test reaching engine internals through a re‑export under a different path would not appear in it.
- **Out of scope by the brief:** `benches/`, `crates/`, `examples/`, `tools/`, `wat-migrate/`, and all of `docs/`.
- **One thread I opened and dropped rather than over‑claim:** `:wat::rete::with-overlay`, `release-session`, `make-rule`, `make-query`, `node-kind-label`, `children-ids-text` have no direct mention in `tests/` or `wat-tests/`, but each is called from `wat/` stdlib (`query.wat`, `grep.wat`, `rete/oracle/*.wat`), so they are exercised indirectly. Establishing whether that indirect coverage actually discriminates would take driving each verb, which I could not do read‑only. I am flagging it as unmeasured, not as a finding.
