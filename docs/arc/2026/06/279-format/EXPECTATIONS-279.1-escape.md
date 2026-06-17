# EXPECTATIONS — Arc 279.1 (weigh on the orchestrator's own build)

Written before the strike, so the result cannot move the goalposts.

| what | command | expected |
|---|---|---|
| feature gate green | `cargo test --release -p wat --test probe_arc279b_format_escape` | 3 passed / 0 failed / 0 ignored |
| arc-279 base unbroken | `cargo test --release -p wat --test probe_arc279_format` | 3 passed / 0 failed |
| foundation unbroken | `cargo test --release -p wat --test probe_arc279b_subs_tuple_macro_eval` | 1 passed / 0 failed |
| deftest binary floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 257 passed / 1 failed (run_string_entry_direct, pre-existing) |
| deporder gate | `cargo test --release --test test_stdlib_load_order 2>&1 \| grep result` | 1 passed / 0 failed (0 violations) |
| lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 929 passed / 36 failed (pre-existing 251-rot class) |

Runtime prediction: 15–25 min (one wat function rewrite; the foundation + spec are pinned).

## Trap-doors named

- **`Tuple` field read off-by-one** — a 4-field `Tuple(mode,pending,buf,segments)`: `first`/`second`/`third`
  read 1-3; the 4th needs `last` (or `get acc 3`). If `last` misbehaves on a 4-tuple, that's STOP-2.
- **String-node helper + literal braces** — the flushed `buf` now legitimately contains `{`/`}`; the
  read-string re-wrap (`"\"" + text + "\""`) must still produce a clean String node. The `"`-guard keeps
  `"` out, so this should hold; verify case 1 (`{{literal}}` → `{literal}`) which exercises a brace-bearing
  text segment.
- **Empty template / all-text / all-doubles** — `""` → `""`; `"{{}}"` → `"{}"`; confirm the emit tail's
  empty/single/concat branches still fire correctly with the new `pieces`.
- **Used-set shape** — the kept unused-kwarg check (`core.wat:707-722`) reads `used-set` as a
  `HashMap<String,bool>`; Pass 2 must produce that exact shape or the kept check breaks (STOP-3).

## Definition of done

All six rows match. The `format` doc comment reflects `{{`/`}}` (no stale `\{` note). The three feature
tests are un-ignored. `wat/core.wat` + the probe file are the only changes. No deferral language in the
final macro (exigere): the escape ships, it is not "TODO"'d.
