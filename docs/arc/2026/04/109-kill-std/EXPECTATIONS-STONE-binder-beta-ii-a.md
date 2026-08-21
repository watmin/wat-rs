# EXPECTATIONS — arc 109, β-ii-a

Written BEFORE the strike, against `9741507da`.

| # | what | expected |
|---|---|---|
| 1 | ★★ **inertness** | floor **4855/4855**, **zero goldens touched** |
| 2 | `wat/cache.wat`'s `lru-svc<K,V>` still declares | floor green |
| 3 | `wat-tests/service-cache-lru.wat` still starts, dials, put/gets | floor green |
| 4 | every monomorphic `defservice` in the corpus unchanged | floor green |
| 5 | clippy `-D warnings` | 0 |
| 6 | the derivation is written ONCE | read the diff — two call sites, one helper |

Row 1 is the whole stone. **An inert change that turns something red has done something.** There is
no acceptance row for "the list has the right contents", because nothing consumes it yet — its
correctness is established by β-ii-b, and this stone's job is to exist without disturbing anything.

## Independent prediction

**10–20 minutes.** Two bindings and a helper. The risk is not difficulty.

## Trap-doors

1. ★ **A leading empty string from the split.** `"<K,V>"` stripped of brackets is `"K,V"`, but a
   naive `split "<"` on the SUFFIX yields `["" "K,V>"]`. An off-by-one here produces a symbol named
   `""`, which will not surface until β-ii-b tries to splice it.
2. ★ **Whitespace.** `"<K, V>"` yields `" V"` unless trimmed. Same invisibility as above.
3. **`nil` vs `[]` for the monomorphic case.** Empty vector, always. A nil would make β-ii-b's
   splice site need a special case, which is the thing the arity-ladder rule exists to prevent.
4. **Two derivations.** The file already carries a note regretting the duplicated `-base`/`-tp`
   shape; adding a second duplicated pair beside it doubles the regret.
5. **A golden moving.** Would mean a downstream consumer picked up the new binding — the stone is
   supposed to be unreachable from every emission.

## Mode B

Any of: an existing binding's value changes · a golden is adjusted rather than reported · a file
other than `wat/service.wat` is edited · `keyword/from-string` used where a symbol node was asked
for · cargo run by the rider.
