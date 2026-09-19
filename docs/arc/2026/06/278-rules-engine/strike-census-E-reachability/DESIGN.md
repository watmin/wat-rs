# DESIGN — is census E's population ever non-empty?

A **measurement strike**. It ships no engine change and its deliverable is a number.

## The question

Census E (`c91121e5e`) moved `census_count("match:calls")` above `alpha_pattern`'s `?` so that an
invocation whose `cond` is **not** an alpha pattern is counted. The E-proof strike then found the
move is invisible in the one world that could see it: every `cond` there is an alpha pattern, so the
`?` never returns early (STOP-1, `../strike-census-E-proof/REVIEW.md`).

So the question underneath is not about a test. It is:

> **Does `alpha_match_inner_opts` ever receive a `cond` that `alpha_pattern` rejects — anywhere?**

If never, census E corrected a counter for a population that is always empty: harmless, and worth
saying at the site. If sometimes, that caller is the real proof and no fixture needs inventing.

`alpha_pattern` (`matcher.rs:429-453`) returns `None` for a non-List `cond`, an empty List, or a
Symbol-headed `cond` that does not classify as `FactBind`. Note most rete forms — including
`(:wat::rete::not …)` and `(:wat::rete::exists …)` — are Keyword-headed Lists and therefore return
`Some` with that keyword as `type_head`, failing later on the head check instead. That is why the
`None` branch may be much narrower than it looks, and why this must be measured rather than argued.

## THE ONE CONTRACT DECISION — a file-append tripwire, not a panic

`census_count` is `#[cfg(test)]` and only visible inside a `with_count_census` window, and nextest
runs a process per test — so a counter cannot aggregate across the suite. A `panic!` would work but
stops at the **first** hit and masks the rest.

**Append one line to a file on the `None` branch, recording the `cond`.** Every test process appends;
nothing aborts; the suite completes; the evidence is the file.

```rust
let Some(pat) = alpha_pattern(cond) else {
    // TRIPWIRE — transient, arc 278 census-E reachability. REVERTED before commit.
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true).append(true).open("/tmp/arc278-alpha-pattern-none.log")
    {
        let _ = writeln!(f, "{cond:?}");
    }
    return None;
};
```

It goes in **`alpha_match_inner_opts` only** — not inside `alpha_pattern`, which has other callers
(`matcher.rs:462`, `:567`) whose `None`s are a different question. The three wrappers
(`alpha_match_inner`, `_local`, `_seeded`) all route through the one body, so one tripwire covers them.

## Procedure

1. `rm -f /tmp/arc278-alpha-pattern-none.log`
2. Apply the tripwire.
3. Run the **full floor** (`scripts/floor.sh`). It must stay GREEN — the tripwire changes no
   behaviour, only appends.
4. `wc -l` the file; `sort | uniq -c` it if non-empty.
5. **Revert the tripwire.** `git diff --quiet src/rete/matcher.rs` must be clean at the end.

## The two outcomes, both useful

- **0 lines** — the branch never fired across 5,465 tests. Census E's population is empty
  everywhere we can see. Ship a **doc note at the site** recording the measurement and its date;
  ship nothing else. ⚠ This is *"never observed"*, **not** *"cannot happen"* — the honest wording
  matters and the note must carry it.
- **N > 0 lines** — the recorded `cond`s name real callers. That is the proof census E lacked, and
  the E-proof's corpus question answers itself: point the test at a real case, not a fixture.

## Out of scope = REJECTED

- Adding a non-alpha `cond` to any corpus. That was disqualified on the **Honest** question — it
  would prove the move against data invented for the purpose and read stronger than it is.
- Deleting or `unreachable!()`-ing the `None` branch on a zero result. A guard that never fires is
  not dead code; it handles a malformed `cond` and a zero measurement does not license removing it.
- Any engine change. The tripwire is transient.

## STOP triggers

1. The floor goes RED with the tripwire in → STOP. The tripwire must be behaviour-neutral; a red
   means it is not (an I/O error path, a sandboxed test, a permissions issue), and the measurement
   is void until that is understood.
2. The file is non-empty **and** the floor is green → do NOT interpret the `cond`s alone. Report
   them verbatim with counts and stop; which caller they come from is the next question.
3. `matcher.rs` is not byte-identical to HEAD at the end → STOP. That is how the last strike left a
   defect in the tree.
