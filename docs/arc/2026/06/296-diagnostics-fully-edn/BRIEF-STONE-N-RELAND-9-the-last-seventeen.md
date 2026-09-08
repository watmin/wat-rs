# BRIEF — STONE N RELAND 9: the last seventeen

> **45 → 17 from a ONE-TOKEN declaration repair.** That is the measure of how far upstream
> `:None` → `:wat::core::Option::None` was, and of how much was compensating for it.

## THE TRAJECTORY

```
M   2773 · 18 timeout   N 473   R1 291   R2 128   R3 69   R4 50   R5 47   R6 45   R8 17
```

RELAND 8 also proved the classifier was RIGHT to refuse: with the declaration repaired, the keyword
keyfn classifies pure again and arc 255 A-2-ii-b-0's property is restored. **The keyfn workaround
was compensating, not fixing** — measured, then reverted.

## THE 17, AND THEY ARE ONE FAMILY PLUS THREE STRAYS

Every positional head still failing is a **GENERATED** variant:

```
:wat::service::Outcome::Reply                       :demo::Op::Go
:wat::query::Store::PutResponse::Success            :usr::Counter::GetResponse::Ok
:wat::telemetry::Journal::QueryLogsResponse::Success    :usr::Counter::IncrementResponse::Ok
:wat::telemetry::Journal::QueryMetricsResponse::Success
```

★ **This is M2's residue 1, named eight relands ago and never closed** — *"unquote constructors whose
gensym was not in the first table (`~reply-variant-kw`, `~admin-allow-peer-kw`, `~status-stopped-kw`,
`~op-variant-kw`, `~admin-init-kw`)"*. RELAND 1 closed the **Rust** half (Path B). The **wat-side
templates** were never finished.

Plus three strays, each its own question:

```
:wat::core::Option::None constructed POSITIONALLY   1 site — a bare (…::None) with no {}
sift_rules ×2 + sift_rules_arena ×2                 still "disconnected" AFTER the repair, so NOT
                                                    the mem-store scan cause. Its own diagnosis.
call_context ×5                                     RELAND 7 saw `positional Op::-Mark` — note the
                                                    LEADING HYPHEN in the variant name. Ask what
                                                    built that name before assuming it is a ctor site.
```

## ⛔ THE INSTRUMENT IS `macroexpand`, NOT grep — TWICE PROVEN THIS SESSION

CLAUDE.md rule 4. The orchestrator burned six commands grepping `service.wat` for the Path B site
(it was in Rust), and just now burned three more on these templates. **A generated ctor is something
the macro EMITTED. Expand a `defservice`/`defsurface` and read the emission.**

`:wat::query::Store::PutResponse::Success` is a good specimen: small surface, one op, and it appears
in both a thread and a process test.

## THE WORK

1. **Expand** one failing surface. Find where the response/op variant is CONSTRUCTED. Report the
   site before changing it — it may be in a wat template or in Rust, and M2/RELAND 1 found one of
   each.
2. **Fix it to the map form**, wherever it lives. STOP-2 of RELAND 1 still stands: a `src/` change
   is authorised if that is where the emission is.
3. **The single bare `(:wat::core::Option::None)`** — one site, needs `{}`.
4. **Diagnose `sift_rules` ×4 separately.** "disconnected" survived the declaration repair, so the
   mem-store scan explanation does not cover it. **Do not assume it shares a root.**
5. **`call_context` ×5** — establish what produced a variant named `-Mark` before treating it as an
   ordinary ctor site.

## STOP TRIGGERS

- **STOP-1 — grep is used to find a macro-emitted construction.** Expand the form.
- **STOP-2 — `sift_rules` is folded into the template class.** It survived the repair; that is
  evidence it is different.
- **STOP-3 — a test expectation is changed to match a wrong value.** Standing since RELAND 7.
- **STOP-4 — the wall is weakened.** Every one of the 17 is a real site or a real question.
- **STOP-5 — a `.wat` corpus site is hand-edited when the codemod could take it.** R21. The guard
  from RELAND 8 means the tool is now safe to re-run.
