# src/collection/ — updated-guard re-earn (249.N ward-close)

> The home's 2026-06-04 stamp (8-spell, old vigilia) drifted: anchor `fc402545`,
> diff since = eval.rs +76 (the eval_vec_rest arm), transform.rs −49 (the 249.4b
> `for`-comprehension cut), mod.rs ±1. Re-earned against the COMPLETE updated
> vigilia. Cast mechanic: spells fetched fresh from the signed channel this
> session, embedded verbatim in workers.

## Muster determinations (conditional wards weighed)

- **perspicere FIRED** — real 2-level type in code (eval.rs turbofish)
- **secare NOT mustered** — the Arc/Atomic grep hits are comments about Value's
  contents; no parallel primitives in-home
- **mora NOT fired** — no duration waits
- **excusare NOT mustered** — zero runes in-home at cast time (nothing to audit)
- **test-kind NOT mustered** — zero in-home test surface (the corpus + probes
  exercise the home from outside)

Guard = universal-7 + exigere + perspicere + circumspicere-last = **10-spell**.

## R1 — the 9 inward casts (2026-06-06, on `c0c6b230`)

| spell | verdict |
|---|---|
| sequi | **CONVERGED 0+0** — HM inference state threaded visibly (&mut Subst/InferCtx); CALL_STACK = dedicated diagnostics pipeline (exempt); sort_err = local error-parking (host idiom) |
| intueri | 2 L1 + 5 L2 + 3 L3 — `list_span` lies in a collection home (38 sites → `call_span`); eval_vec_rest doc omits its third (WatAST) arm; MIXED-VERBS banner; infer_conj bool fallback; map-with-index arg-order undeclared; canonical-key ghost |
| solvere | 2 L2 + 1 L3 — runtime.rs eval_length/eval_empty inline-duplicate the _inner encodings (siblings delegate); eval_assoc Record arm double-evaluates via raw-AST re-pass |
| conformare | 6 L2 + 3 L3 — 8 Span::unknown() sites with a real span in scope (args[0].span / call-span / rust_caller_span!); stale arc-138 comments on arity paths that already thread the span |
| purgare | 3 L2 + 1 L3 — mod.rs layout lies (eval_vec_rest filed under transform.rs; ~16 count wrong; contains_key_q shorthand trap). NO dead code: every pub(crate) fn has live consumers; the 249.4b cut left zero orphans |
| struere | 1 L1 + 3 L2 + 1 L3 — eval_list_{zip,window,remove_at,map_with_index} enforce Vec (require_vec) under List names; infer_conj bool fallback (convergent w/ intueri); eval_vec_rest name (convergent ×3) |
| temperare | 1 L2 + 1 L3 — 4 infer error-arms recompute apply_subst while the richer `reduced` is in scope; sort_by two-sided predicate documented-but-unruned |
| exigere | 2 L2 — the two "would require touching the entire dispatcher arm chain" comments (arc-138-cited, arc verified on disk) — DIE with the span threading |
| perspicere | 1 L3 — the lone turbofish; no substrate-wide alias exists to reuse → rune |

**Totals: 3 L1 + 22 L2 + ~11 L3.** Convergences: eval_vec_rest hit by 3 lenses;
infer_conj by 2; the span-threading fix closes conformare AND exigere together.

## Orchestrator weighing

- `list_span`→`call_span` FOUGHT in-home only (param names are home-local;
  macros/ set the per-home precedent with call_site_span; runtime.rs untouched).
- eval_list_* renames fight the RUST names only — the `:wat::std::list::`
  op-string namespace is wat-surface contract, unchanged.
- Span threading > comment rewording: the arc-138 stale comments die because
  their reason dies.
- **Three runes EARNED through combat**: conformare(spanless-by-domain) on the
  _inner family (pre-evaluated &Value API — no AST reachable on any call path);
  temperare(simplicity-win) on sort_by (fix = a new wat predicate protocol —
  out of home scope; cost ceiling cited); perspicere(mumble-alias) on the
  turbofish (no existing alias to reuse; single-home alias would diverge).
- **Declined**: 6-param infer fns (the check-side calling convention is the
  constraint, not this home's design choice) — L3, recorded not fought.
- runtime.rs touches sanctioned as the braid's other half: eval_length/empty
  delegation, record_assoc value-level inner, eval_rest call-site rename.

## Fight sweep dispatched (items A–R)

A list_span→call_span · B eval_rest + layout truth · C the four Vec-enforcing
renames · D 8-site span threading · E stale-comment purge · F the _inner rune ·
G infer_conj fresh-var fallback · H length/empty delegation · I record-assoc
double-eval kill (STOP-if-not-clean) · J _inner doc trio · K contains banner ·
L arg-order NB · M singleton banner · N canonical-key ghost · O use `reduced` ×4 ·
P sort_by rune · Q turbofish rune · R wrapper/inner reorder.

Gates: test-build + lib 920-baseline + the wat corpus (217/0/53) + 6 probes +
collection-clippy-empty. On green: convergence judgment → **circumspicere LAST**
→ re-stamp (ISO8601-UTC-seconds per docs/VIGILATUM.md — compute at convergence,
never date-only).
