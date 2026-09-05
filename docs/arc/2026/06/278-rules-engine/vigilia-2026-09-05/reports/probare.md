# ward `probare` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

## L1 — assertions whose negation is unreachable

**1. `src/rete/reachability.rs:1659–1665` — the discrimination row never executes, for either case.**

```rust
for (name, src, other) in [("keyword", KW, ":beta"), ("enum", EN, ":probe::E::B")] {
    ...
    let never = src.replacen(&format!("::= :v {other}"), "::= :v :zeta", 1);
    if never != *src {
        assert_eq!(raw_count(&never), Ok(0), "`{name}`: a constant equal to no fact must select NOTHING");
    }
}
```

`other` names the **miss fact's** value, not the **rule's** constant. `KW`'s rule reads `(:wat::rete::core::keyword::= :v :alpha)` (`reachability.rs:1603`) and `EN`'s reads `(:wat::rete::core::enum::= :v :probe::E::A)` (`reachability.rs:1625`). I grepped both consts this session: `::= :v :beta` → 0 hits in `KW`; `::= :v :probe::E::B` → 0 hits in `EN`; the rule-constant forms → 1 hit each. So `replacen` no-ops, `never == *src`, the `if` is never entered, and the `assert_eq!` inside it **has never run**. The comment two lines above states what it is vouching for: *"otherwise the operand is being evaluated but not compared, and the rows above prove nothing."* That claim is currently unproven.

The tell is local: the two *sibling* rewrites in the same loop body (`reachability.rs:1653`) and at `:1398` and `:1442` each carry `assert_ne!(src, oracle, "the rewrite must actually select the oracle")`. This one carries an `if` instead — a guard that converts "the rewrite did not happen" into silent success.

*Mutation that proves it hollow:* replace the `if never != *src {}` with `assert_ne!(never, *src, ...)`, matching the sibling at :1653. It goes RED immediately at HEAD, with no change to the engine.

---

**2. `tests/wat_lang/probe_assert_true_false.rs:32` and `:46`, and `tests/collection/probe_nth.rs:28` — bare `#[should_panic]` over a body with two setup panic sites.**

`call_fn` (`probe_assert_true_false.rs:14–21`) panics on `world.symbols().get(name)` returning `None` ("no {name:?} in fixture"), and the test body panics on `startup_beside(file!()).expect("startup")`. With no `expected =`, `assert_true_panics_on_false` and `assert_false_panics_on_true` are satisfied by **any** panic — a missing fixture, a rename, a freeze failure — not by the assertion firing. Identical shape at `probe_nth.rs:28`, where `call_beside_value` itself panics if the fixture fails to freeze (`src/freeze.rs:1086`).

This is the C18 shape exactly: green for a reason unrelated to the wall it exists to prove. The fixtures do exist today (`tests/wat_lang/probe_assert_true_false.wat:7,13`; `tests/collection/probe_nth.wat:9`), so these pass for the right reason *right now* — the assertion simply cannot tell the difference.

*Mutation:* rename `:t::assert-true-on-false` to `:t::assert-true-on-flase` in the `.rs` call. The test stays GREEN — the lookup panic satisfies `should_panic`.

*Remedy:* `#[should_panic(expected = "assert-true")]` at minimum; better, drop `should_panic` and match the `Err`/payload the way `tests/kernel/probe_deftest_verdict_wall.rs:51` does.

---

**3. `tests/kernel/probe_arc275_verify_stdlib.rs:30` — `assert!(n >= 0)` on a violation count.**

```rust
Value::i64(n) => {
    println!("verify-stdlib violation count = {n}");
    // Just assert it returns a non-negative count (the actual enforcement is 275.2).
    assert!(n >= 0);
}
```

`n` is `(:wat::core::length (:wat::deporder::verify-stdlib))`. A length is never negative; the negation is unreachable. No message, so nothing even describes what a failure would mean.

The same file's `probe_verify_stdlib_violations_detail` (`:36–:52`) carries **no assertion at all** — it prints and returns. Both are unattributed `#[test]`s on the green floor.

*Mutation:* make `:user::compute-violation-count` in the co-located `.wat` return the constant `9999`. Both tests stay GREEN.

*Remedy:* these are `--nocapture` reporting harnesses, not gates — `tests/kernel/test_stdlib_load_order.rs:17` is the real enforcement (`assert_eq!(n, 0)`). Either delete them or mark them `#[ignore]` so they stop counting as floor coverage.

---

**4. `wat-tests/deporder.wat:71` — `(:wat::test::assert-true (:wat::core::i64::>= (:wat::core::length viols) 0))`.**

Live on the floor (not `#[ignore]`d; it carries `(:wat::test::time-limit "30s")` at `:69`), driven as a `#[test]` through `wat::test! {}` in `tests/kernel/test.rs:17`. Same tautology: a length is `>= 0` for every Vector. The deftest's own comment concedes it: *"its length may be zero or more — the enforcement test is 275.2."*

So a 30-second floor test asserts nothing that the harness's own "did it raise?" verdict does not already carry.

The identical tautology at `wat-tests/lint.wat:105` is **already documented as tautological in-file (`:90–:94`) and is `#[ignore]`d (`:97`)** — so `deporder.wat:71` is the surviving live instance of a class the tree has already named.

---

**5. `src/rete/kernel/tests/gather_probe_cost.rs:706–712` — the guard names a mechanism it cannot observe.**

`gather_val_id_split` (`:610`) times a `HashMap<Value>` arm against a `HashMap<u32>` interned arm. Its own comment at `:682–:685` states the hazard: *"If they diverge, the two timings above are measuring different work … which would most likely show up as the interned side looking FASTER."* It then asserts:

- `:700` `idx_v.len() == 201`
- `:706` `idx_i.len() == idx_v.len()` — **bucket count only**
- `:714` `idx_v.values().map(Vec::len).sum() == N` — element total, **for `idx_v` only**

`idx_i`'s element total is never asserted. With `g = i/200` over 40,200 elements, each of the 201 buckets holds 200 elements; the interned arm could drop up to 199 per bucket and still report 201 buckets. The stated failure mode — the interned arm indexing fewer elements and therefore reading as a speedup — passes both live assertions.

The sibling test in the same file, `gather_unary_index_split` (`:446`), gets this right: it asserts `sum() == N` for **both** arms (`:561` and `:568`), each with a message naming the lossy-failure risk. The asymmetry is the finding.

*Mutation:* in the rebuild loop at `:693–:698`, change the push to `if i % 2 == 0 { idx_i.entry(*vid).or_default().push(i); }`. Bucket count stays 201, element total halves, and the test stays GREEN.

*Remedy:* add `assert_eq!(idx_i.values().map(Vec::len).sum::<usize>(), N, ...)` beside `:714`.

---

**6. `wat-tests/core/struct-to-form.wat:56–70` — a deftest named for a property it does not check, on a false premise.**

```
(:wat::test::deftest :wat-rs::std::struct-to-form::test-quasiquote-splices-runtime-values
  (:wat::core::let [x 42  y "hello"  form (:wat::core::quasiquote (:my::Foo ~x ~y))]
    ;; … No further structural inspection is available (show renders "<WatAST>" …)
    (:wat::core::do form nil)))
```

The name asserts that quasiquote **splices runtime values**. Nothing observes the spliced values. If `~x`/`~y` spliced the symbol instead of `42`/`"hello"`, or dropped the unquote entirely, the deftest returns `Passed`. The only reachable failure is a panic during construction.

The in-file justification — *"No further structural inspection is available"* — is **false at HEAD**. `:wat::core::ast->source` is a live intercepted primitive (`src/runtime.rs:5262`, implemented at `src/edn_shim.rs:728` as `eval_ast_to_source`) that serializes any `Value::wat__WatAST` to verbatim source; `:wat::core::ast->children` (`src/runtime.rs:5269`) decomposes one for walking. Either would let this test assert what its name claims.

*Mutation:* nothing. The test cannot distinguish a working quasiquote from one that splices the wrong thing — that is the finding.

*Remedy:* `(:wat::test::assert-eq (:wat::core::ast->source form) "(:my::Foo 42 \"hello\")")`.

---

## L2 — weak checks, remedy known

**`tests/lint/gen_doc_surface_matches.rs:161–196`** — `every_gen_name_the_doc_writes_actually_exists` has no liveness guard on its discovered population. `doc_qualified_names(&doc)` extracts every `:wat::gen::NAME` from the design record; if that extractor goes blind (the doc reworded to drop qualified names, the prefix changed), `phantoms` is empty and the test passes over nothing. Its sibling `every_exported_gen_verb_is_documented` (`:145`) *does* guard, with `verbs.len() >= 20`. Note the `types` set is safe by accident — a blind `declared(&lib, TYPE_FORMS)` would red, not green. Remedy: `assert!(doc_qualified_names(&doc).len() >= N)` before the filter.

Note also that this file is what `tests/lint/every_walking_gate_declares_non_vacuity.rs:26–29` names as its one admitted out-of-scope hole — but that header's description of it (*"parses 27 verbs out of a named file and would pass on 0"*) is stale for the first test, which now guards at `>= 20`. It is accurate for the *second*, which is the one above.

**`tests/lint/every_walking_gate_declares_non_vacuity.rs:113–121`** — `discovery()` keys scope on the literal strings `read_dir` and `Command::new` in the file's own text. A gate that walks through a shared helper, `walkdir`, or `glob` is classified `Named` and is never asked for a declaration. I checked all 40 gate files: none currently walks by any other route, so the hole is latent rather than live. Remedy: add the alternate walkers to the recogniser, or make the positive control cover a helper-walked file.

**`tests/rete/probe_arc278_49_one_core_covers_the_surfaces.rs:78–130`** — the single test asserts properties of `SURFACES`, a `const` table declared 8 lines above it in the same file. No assertion reads anything from `src/`. Nothing any engine change can do will turn it red; only editing the table will. The header is honest about why (*"`Op` is `pub(crate)`, so the probe models the shapes locally and cannot be held against the real type from where it stands"*) and correctly hands the real gate to `every_op_variant_lands_in_core_or_driver` in `src/rete/compiled_cond.rs`. It is a description wearing `#[test]` — a floor row that argues rather than proves. Worth an `#[ignore]` or a rename, not a fix.

---

## L3 — judgement

The lint directory is in genuinely good shape. `every_walking_gate_declares_non_vacuity.rs`, `census_name_read_by_a_cost_test_is_emitted.rs`, `diagnostic_output_is_deterministic.rs` and `probe_deftest_verdict_wall.rs` all carry real non-vacuity guards *plus* positive controls, and several state their own limits in the header rather than implying coverage they lack. The `$oracle`/`$native` split is a real dual implementation (`wat/rete/oracle/insert.wat:22`, `:45` are independent wat bodies, not delegates), so the rete differentials are comparing two engines.

The remaining hollowness clusters in **two places the lint discipline does not reach**: (a) `wat-tests/**/*.wat` deftests, which land on the floor via `wat::test! {}` but are subject to no vacuity gate at all — a deftest with zero assertions returns `Passed`, and `deftest_verdict` (`src/freeze.rs:1119`) has no way to know; and (b) old smoke/probe tests written as reporting harnesses (`probe_arc275_verify_stdlib.rs`) that were never retired once the real enforcement landed. Both are the same failure: a row that once meant "I ran this and looked" left behind as if it meant "this is checked."

The `>= 0` tautology has now appeared three times in this tree (`lint.wat:105` — caught and ignored; `deporder.wat:71` — live; `probe_arc275_verify_stdlib.rs:30` — live) and once historically (`probe_arc216_stone5a_value_hash.rs:372` records its removal). That is a class, not four accidents, and it is cheaply gateable: a lint refusing `>= 0` against a `length`/`len()` expression in an assertion position, in both `.rs` and `.wat`.

---

## What I could not check, and why

- **I ran nothing.** Read-only, per the brief. Every "would still pass GREEN" above is a static argument from reading, not a driven mutation. The one exception is finding 1, where I verified the substring absence directly with `grep` against the two `const` blocks — that one is measured, not inferred.
- **The 256 `.is_err()` sites** across `src/` and `tests/` are the C18 class at scale — an `assert!(r.is_err())` that holds because the input has a *second*, unrelated error. Distinguishing a right-reason from a wrong-reason `is_err` needs the error kind driven per site. I read roughly 40 of the call sites' messages and none was obviously wrong; I did not verify one.
- **`src/`'s `#[cfg(test)]` modules are large and I sampled them.** I read `src/rete/reachability.rs`, `src/rete/kernel/tests/gather_probe_cost.rs`, and `src/rete/kernel/tests/accum_alpha_cost.rs` in the regions my scanners flagged. `src/runtime.rs`, `src/check.rs`, `src/types.rs` and `src/edn_shim.rs` all carry inline test modules I did not audit.
- **`benches/`, `crates/`, `examples/`, `wat-scripts/`** — not examined. `wat-scripts/**/*.wat` in particular is gated only for load + rete-name resolution (per `CLAUDE.md`), so it is a plausible hollow reservoir I did not open.
- **I did not verify the floor count attribution.** I did not confirm how many of the 5420 rows are `wat-tests` deftests versus Rust `#[test]`s, so I cannot say what fraction of the floor sits behind the un-gated `deftest`-returns-`Passed` path.
- **I could not check whether any of these six were already known and deliberately deferred.** The brief said C9/C14/C16/C18/D2 and the self-comparing gate were closed today; none of my findings is one of those, but I did not read `docs/` (excluded by the brief), so if any of them is tracked there as an open row I would not know.
