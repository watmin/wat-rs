# EXPECTATIONS — arc 109, β-ii-b

Written BEFORE the strike, against `a39eb99aa`.

| # | what | expected |
|---|---|---|
| 1 | ★★ `lru-svc<K,V>` still works END TO END | `wat-tests/service-cache-lru.wat` — starts the service, dials two clients, put/get, eviction |
| 2 | the new spelling still works | an adapted `:- [K V]` copy runs |
| 3 | ★ the binder's contents still load-bearing | `:- [X Y]` still FAILS |
| 4 | every monomorphic service unchanged | floor green |
| 5 | floor | **0 FAIL** |
| 6 | clippy | 0 |
| 7 | the 5 TYPE names still carry `{p}` | read the diff — β-ii-c's territory, untouched |
| 8 | the substring cluster untouched | `:829`–`:851`, `:1795` unchanged |

**Row 1 is the acceptance instrument, not row 5.** The floor exercises `lru-svc` only as far as
declaring it; `wat-tests/service-cache-lru.wat` is what actually STARTS the service and calls
through the generated `start`/`put`/`get` — which are exactly the names this stone renames. A green
floor with a broken generated `start` is entirely possible.

⚠ Confirm `wat-tests/` is in the floor's scope. If it is not, run it by hand and say so — an
acceptance instrument nobody runs is not an instrument.

## Independent prediction

**15–25 minutes.** Twenty near-identical deletions. The work is not the edit; it is the per-site
check that the dropped params appear in each function's signature.

## Trap-doors

1. ★ **A param that lives only in a BODY.** The licence rests on 251.7 unioning name-params with
   signature free-vars. A generated function using `K` only inside its body would silently lose it.
   STOP-1 exists for this; row 1 is what would catch it if the check is skipped.
2. **A macro-local comparison against a suffixed spelling.** The RUNTIME registers under the base —
   but a comparison inside this macro could still expect `"…::init<K,V>"`. Grep the file for each
   renamed string being compared, not just constructed.
3. **Dropping `{p}` from a TYPE name by accident.** `{b}::Admin{p}` and `{b}::Handle{p}` sit among
   the function names and read the same. Row 7 is the check.
4. **`$impl` variants.** Six of the twenty are `start$impl`, `start$impl-thread`, `resume$impl-process`
   and friends — easy to miss one, and a missed one is invisible until something calls it.

## Mode B

Any of: a TYPE name loses `{p}` · the substring cluster is touched · a site is dropped without its
signature checked · a file other than `wat/service.wat` edited · cargo run by the rider.
