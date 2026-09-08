# BRIEF — STONE N RELAND 10: the last four

```
M    2773 failed + 18 TIMED OUT — no wat program could start
                    …nine relands…
R9   5234 passed ·  4 failed
```

Four tests. One service. One code path.

```
probe_arc278_sift_rules::sift_rules_defsvc_counts_exact_deductions_on_{thread,process}
probe_arc278_sift_rules_arena::sift_rules_arena_counts_exact_deductions_paged_on_{thread,process}
```

## THE SYMPTOM IS NOT THE EVIDENCE

```
#wat.kernel/AssertionFailure {:message "disconnected" …}
```

That is the CLIENT observing `RecvOutcome::Lost`. **The sift service thread died; "disconnected" is
what the client saw afterwards.** The service's own death reason is the evidence and it is not in
this message.

RELAND 9 already narrowed it correctly and that narrowing must be preserved: **the fail-closed arms
PASS** on both thread and process, for both defsvc and arena. Only **counts-exact** dies. If init or
`query-logs` were the cause, fail-closed would fail too. **The failure is inside `fire-rules`.**

## ★ THE NAMED SUSPECT — MEASURE IT, DO NOT ASSUME IT

This is the **same symptom shape** as RELAND 7's journal failure, which was:

```
sort$native classifies the comparator BEFORE any comparison
classify_closure walks the accessor IMPL, hits :wat::core::Option::None
  -> "comparator is not pure"  -> the store thread dies  -> client sees Lost  -> "disconnected"
```

That one was caused by the corrupted declaration and is now fixed. **But the mechanism is general:
this campaign made `Option::None`/`Some` enum heads the purity lattice must classify, and any
closure classified before use can refuse for the same reason.** `fire-rules` runs user predicates.

⛔ **It may be something else entirely.** RELAND 7's first hypothesis (field-order transposition) was
wrong and the measurement said so. Name what you find; do not fit it to this.

## THE WORK — GET THE SERVICE'S OWN ERROR

1. **Drive `fire-rules` DIRECTLY**, outside the service, on the same rules and facts — exactly how
   RELAND 7 found the store's death by calling `Store/put` then `Store/scan` on a fresh mem-store.
   The direct call surfaces the real error instead of `Lost`.
2. If it does not reproduce directly, **print inside the service arm** before and after `fire-rules`
   and find the last statement that runs.
3. Report the service's own error verbatim, with the arm.
4. Fix at the layer the error names — `src/` is authorised if that is where it lives (RELAND 1
   precedent).
5. Floor.

## STOP TRIGGERS

- **STOP-1 — "disconnected" is reported as the cause.** It is the client's view of a corpse.
- **STOP-2 — the fix is a workaround at the call site.** RELAND 7's keyfn rewrite looked like a fix
  and was compensating for a corrupted declaration; RELAND 8 proved it by reverting and measuring.
  If a call-site change makes it pass, **revert it and prove the underlying cause is real.**
- **STOP-3 — a test expectation is changed.** Standing since RELAND 7.
- **STOP-4 — the wall is weakened.**
- **STOP-5 — the four are declared "pre-existing" without a bisect.** The campaign's save points are
  all local commits; if it predates Stone M, that is a finding and the campaign closes green.
