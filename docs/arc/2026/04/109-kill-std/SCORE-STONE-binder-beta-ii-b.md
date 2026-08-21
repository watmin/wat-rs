# SCORE — arc 109, β-ii-b: the generated FUNCTION names drop `{p}`

Rider: one flight, ~6 min, no STOP fired — but it **declined a site the brief implied**, correctly.

| # | what | result |
|---|---|---|
| 1 | ★★ `lru-svc<K,V>` end to end | ✅ `deftest_wat_tests_service_cache_lru_multi_client_on_process` **and** `_on_thread` PASS — the tests that START the service, dial two clients and call the renamed `start`/`put`/`get` |
| 2 | the `:- [K V]` spelling | ✅ clean |
| 3 | ★ the binder's contents still load-bearing | ✅ `:- [X Y]` still fails |
| 4 | `:- []` monomorphic | ✅ clean |
| 5 | floor | ✅ **4855/4855, 0 FAIL** |
| 6 | clippy | ✅ 0 |
| 7 | the TYPE names still carry `{p}` | ✅ 6 untouched |
| 8 | the substring cluster untouched | ✅ |

18 sites, `18 18 wat/service.wat`, nothing else modified.

## ★ THE RIDER REFUSED A 19TH SITE, AND ITS ARGUMENT IS EXACTLY RIGHT

`:1794` — `method-name`, the per-op client method built inside the `op-methods` fold — is
function-shaped and reads like the eighteen. The rider left it and said why. **Verified by my own
hand:**

```
the NAME carries       fqdn-tp    (the SERVICE's params)
client-peer-ty    ←    proto-op / proto-reply
req-ty            ←    proto-base / proto-tp     (the SURFACE's params)
resp-ty-str       ←    proto-base / proto-tp
```

So the method's signature is typed from `proto-tp` while its name carries `fqdn-tp`, and **nothing
in the macro enforces those are equal** — only a design comment saying a user is expected to write
matching binder names at both sites. STOP-1's proviso ("every param the name carries appears in that
function's signature") is therefore NOT verifiable there, unlike the eighteen where I can point at a
concrete `~state-ty` / `~handle-name` / `~record-ty` carrying the same `fqdn-tp`-derived suffix.

★ It also found a second, independent confirmation I had not noticed: **EXPECTATIONS' own
"substring cluster untouched" range names `:1795`** — which is the closing line of that very
interpolation. My own scorecard already fenced the site my brief's table implied was in scope.

That is the second time a rider on this arc has declined something a brief of mine implied and been
right. `[[feedback_an_instruction_to_delete_needs_more_grounding_than_one_to_add]]`

## ⛔ AND TWO OF MY EIGHTEEN WERE DEAD CODE

`start-name` (old `:824`) and `resume-name` (old `:2511`) are **bound and never referenced** —
measured, 1 occurrence each, their own binding line. Compare `serve-name` at 40 and `init-name` at 6.
The public `/start` and `/resume` macros are built separately, base-only, from
`start-macro-name` / `resume-macro-name`.

So my table listed two dead variables as work sites. The rider edited them per the table, noted they
are no-ops, and flagged them. **The brief presented a census as a worklist without checking that
each entry was live** — the same shape as counting definitions instead of call sites, one level down.

⚠ Follow-up, bounded: `start-name` and `resume-name` should be DELETED, not merely de-suffixed.
`wat/service.wat` only; no behaviour change. Not done here because deleting a binding is out of a
"drop the suffix" stone's blast radius.

## Honest deltas

- **My "~20" was 18.** The table itself listed 18; the "~" was hedging I had not earned. The 19th
  candidate exists and is correctly out of scope, which is where the imprecision came from.
- **My "5 type names" is 6** — `{b}::Op{p}` appears at BOTH `:807` and `:1080`. Row 7 counted them.
- **The perturbation's error count dropped from 2 to 1.** Expected and benign: one of the two errors
  was in a generated FUNCTION whose name carried `<X,Y>`; that name no longer carries params, so the
  remaining error is the type-level one. Still fires ⇒ still load-bearing.

## What β-ii-b did NOT do

- The **6 TYPE names** → β-ii-c, with the 10 `proto-tp` sites.
- The **substring cluster** (`:829`–`:851`, `:1795`) → β-ii-d.
- `method-name` at `:1794` → belongs with β-ii-c/d, and its `fqdn-tp`-vs-`proto-tp` mismatch is a
  question about the macro's model, not a port.
