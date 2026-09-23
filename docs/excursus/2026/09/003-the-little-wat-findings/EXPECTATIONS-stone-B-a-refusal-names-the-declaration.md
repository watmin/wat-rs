# EXPECTATIONS — STONE B

Written before the strike. Scored against my own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | F-006 names the user's file | `wat` on the f006 fixture | `:file` is the `.wat`, not a `.rs` |
| 2 | F-114 names the user's file | `wat` on a `defrecord` holding a fn field | `:file` is the `.wat` |
| 3 | the banked probe flips | `-E 'test(/diagnostic_locates_the_user/)'` | f006 asserts the CURED shape; **control still green** |
| 4 | ⭐ the gate can fail | restore the sentinel lookup | **both** flipped tests RED |
| 5 | …and recovers | restore the cure | green |
| 6 | no registration path is missed | STOP-1 not tripped | every user-declared type has a decl span |
| 7 | stdlib/builtin types unharmed | floor | no new failures from types registered out of Rust |
| 8 | the floor | `scripts/floor.sh` → `.floor/latest/clean.log` | **0 failed** |
| 9 | clippy | `cargo clippy --release --all-targets` | clean |
| 10 | the gates, **staged first** | the six named in the BRIEF | all pass |

⛔ Every verdict from the `Summary` line, never a piped exit code.

## Runtime prediction

Edit is small — one field, one insert, two call sites, one assertion flip: **30-50 min**.
⚠ **Build dominates:** release harness ~3m 15s, floor **~25 min**. Total 1.5-2.5 h.
⛔ Never poll a build with `pgrep -f 'cargo …'` — it matches its own command line.

## Trap doors, named in advance

1. ⭐ **`register()` (span-less) calls `register_with_span(def, rust_caller_span!())`.** So
   builtins registered from Rust will have a decl span that IS a `.rs` location. For a
   user-declared type that never happens — but a walk that refuses a BUILTIN would now report a
   Rust span through a new route. **That is not a regression** (it reports one today) — but do
   not describe the cure as "no diagnostic can name a .rs file". It cannot be, while builtins
   register from Rust.
2. **The two walks are not symmetrical.** `validate_aggregate_containment` only iterates
   `env.iter()`; `validate_named_type_annotations` iterates that **and** `functions_iter()`.
   F-114 needs only the first; F-006 needs the second. Curing one does not cure the other, and
   the probe has a separate test for each.
3. **`FunctionBody::Native` has no AST.** Keep the sentinel there and say so in a line.
4. **Idempotent re-registration.** `register_validated` no-ops on byte-equivalent duplicates
   (arc 054). Make sure the span insert does not resurrect a rejected duplicate or overwrite a
   first declaration's span with a later one — decide which wins and write it down.
5. ⚠ **A `HashMap` insert on every registration is on the startup path.** The retirement-table
   gate already measures ~147 ms of startup per `wat` invocation; if this measurably moves it,
   report the number rather than absorbing it.
