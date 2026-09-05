# REFUTE — the floor is RED, 7 tests. And the DESIGN was backwards: Part 2 is the fix, Part 1 defeats it.

> Orchestrator, central floor. **Do not re-run** — log kept at `.floor/latest/`, arms below verbatim.
> ⚠ I first reported "three failures" from a `tail -3`. It is **seven**. Corrected here.

```
     Summary [ 119.683s] 5170 tests run: 5163 passed, 7 failed, 17 skipped
```

## THE SEVEN

```
        FAIL [   0.016s] (3173/5170) wat::program probe_arc213_program_edn_roundtrip::stop_trigger_slash_in_name_keyword
        FAIL [   0.371s] (3431/5170) wat::resolve probe_arc258_stone3_fix_source::contract_06_fix_source_preserves_option_expect
        FAIL [   0.950s] (4500/5170) wat::wat_lang wat_arc144_special_forms::lookup_form_let_returns_special_form
        FAIL [   0.958s] (4499/5170) wat::wat_lang wat_arc144_special_forms::lookup_form_fn_returns_special_form
        FAIL [   0.966s] (4502/5170) wat::wat_lang wat_arc144_special_forms::lookup_form_quasiquote_returns_special_form
        FAIL [   1.007s] (4503/5170) wat::wat_lang wat_arc144_special_forms::lookup_form_if_returns_special_form
        FAIL [   1.022s] (4506/5170) wat::wat_lang wat_arc144_special_forms::lookup_form_match_returns_special_form
```

Five are one shape (`wat_arc144_special_forms::lookup_form_*`) — inline assertions on the old
spelling. The other two are the finding.

## ⛔ ARM 1 — arc 213 WROTE A WITNESS FOR THIS EXACT CHANGE, AND IT FIRED

```
    STOP TRIGGER: keyword :wat::core::HashMap/length encodes to EDN ':wat.core.HashMap/length' — two '/' in body

    thread 'probe_arc213_program_edn_roundtrip::stop_trigger_slash_in_name_keyword' (2596397) panicked at /home/john/work/holon/wat-rs/tests/program/probe_arc213_program_edn_roundtrip.rs:197:17:
    STOP TRIGGER CONFIRMED: :wat::core::HashMap/length encoded to ':wat.core.HashMap/length', decoded to ':wat::core::HashMap::length' (CORRUPTED — wrong keyword)
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

        PASS [   0.370s] (3174/5170) wat::macros vector_splice_symmetry::anaphoric_splice_capture_refused_by_hygiene
```

Read `tests/program/probe_arc213_program_edn_roundtrip.rs:175-220`. It encodes
`:wat::core::HashMap/length`, and then:

- decode gives back the SAME keyword → `eprintln!("STOP trigger may be resolved")` — **passes**
- decode gives back a DIFFERENT keyword → `panic!("CORRUPTED — wrong keyword")` — **what fired**

**Arc 213 anticipated precisely this: "if you ever make the encode valid, prove the decode is
correct too, or you have made it worse."** Before this stone the write emitted two slashes and
`read` REFUSED — loud. After Part 1 the write is readable and decodes to
`:wat::core::HashMap::length` — **the wrong name, silently.** We traded a loud failure for a quiet
one. That is the single trade this campaign exists to refuse.

## ARM 2 — the same class, in `fix-source`

```

    thread 'probe_arc258_stone3_fix_source::contract_06_fix_source_preserves_option_expect' (2597723) panicked at /home/john/work/holon/wat-rs/tests/resolve/probe_arc258_stone3_fix_source.rs:103:5:
    assertion `left == right` failed: fix-source must preserve Option/expect's -> :T annotation through the walk
      left: "(:wat.core/do (:wat.core.Option/expect -> :wat.core/i64 x \"m\"))"
     right: "(:wat.core/do (:wat.core/Option/expect -> :wat.core/i64 x \"m\"))"
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## ★★★ THE DESIGN WAS BACKWARDS — AND THE SUBSTRATE ALREADY HELD THE RIGHT ANSWER

I wrote Part 1 (the fold) as the fix and Part 2 (the `try_ns` refusal) as hardening. **It is the
other way round, and the evidence is on disk:**

```
src/edn/bridge.rs:109   verbatim_keyword(path) -> Tagged(#…/Keyword, {:path "<the exact wat name>"})
src/edn/bridge.rs:727   the DECODER for that tag — reads the path back EXACTLY
src/edn/render.rs       keyword_from_wat_path's existing Err arm already calls verbatim_keyword
```

So **without the fold**, Part 2 does the whole job:

```
:wat::core::HashMap/length
  -> try_ns("wat.core", "HashMap/length")  -> Err        (Part 2's refusal)
  -> the EXISTING Err arm                  -> verbatim_keyword
  -> #…/Keyword {:path ":wat::core::HashMap/length"}     valid EDN, exact
  -> bridge.rs:727 decodes it              -> the SAME keyword
  -> arc 213's test                        -> "STOP trigger may be resolved"
```

**Part 1 is what breaks this.** Folding makes `try_ns` SUCCEED, so the verbatim path is never
taken, and the decode is left guessing — which is where the corruption comes from. The fold is not
merely unnecessary; it is the defect.

★ And note what that means about the original bug: the two-slash keyword was never the disease. It
was `try_ns` accepting a name it should have refused, which routed AROUND machinery that was
already correct. The wall does not harden the fix — **the wall IS the fix.**

## WHAT TO DO

1. **REVERT Part 1.** `keyword_from_wat_path` goes back to what it was; it must NOT share
   `wat_keyword_to_clojure_symbol`'s fold. Leave that function alone — it has its own callers and
   is correct for them.
2. **KEEP Part 2** (`try_ns`/`ns` refuse a name containing `/`, with `/` itself still legal) and
   **KEEP Part 3** (the `fqdn_of` comment). Both are right.
3. **Re-measure the five goldens.** Without the fold they will NOT return to the two-slash form —
   they take the verbatim tagged carriage instead, which is a THIRD shape. Capture what each
   actually becomes; do not assume. If a golden's shape changes in more than the one keyword, STOP.
4. **The five `wat_arc144_special_forms` inline assertions** move with whatever shape step 3
   produces. They assert a sentinel; the sentinel's spelling changed for a good reason.
5. **Arc 213's test must PASS by round-tripping correctly**, not by weakening. If it cannot, STOP —
   that is a finding and the stone is not shippable.

⛔ **Do not touch the reverse discriminator, do not rename anything, and do not weaken arc 213's
witness.** Its success condition (`decoded == original`) is the bar; changing it is the builder's
ruling, not this stone's.

## WHAT IS ALREADY VERIFIED — do not redo

Census **571 · 85 · 52**. `fqdn_of` diff is comment-only (8 insertions, 3 deletions, zero code
lines). The `one_name_grammar` rune on `split_clojure_symbol_ns_name` earns its standing — it
splits an EDN clojure symbol's `ns/name` including `ns//`, where `identifier::method` yields empty;
that is genuinely not the lint's subject. `try_ns`'s three-case behaviour is right, including `/`
itself staying legal.
