# SCORE — `defservice` compares and destructures types as DATA

Brief: `BRIEF-STONE-defservice-compares-types-as-data.md`. Rider returned; scored against my own
re-run of every load-bearing row, not against its report.

## The verdict up front

**Shipped.** `wat/service.wat` only. Floor green, clippy 0. The rider fired **STOP-3 twice and was
right both times** — my classification table was wrong at two of the nine rows I wrote, exactly as
the brief's own warning anticipated (*"it is my reading, not a census, and this file has made my
counts wrong four times"*).

## Rows — my own instruments, not the rider's

| row | my check | result |
|---|---|---|
| floor | `scripts/floor.sh` | **4854/4854, 0 FAIL, 19 skipped**, 75.5s. No `ARM.txt`. |
| clippy | `--workspace --all-targets --release -- -D warnings` | **0** |
| baseline | `--check` the unperturbed `probe_arc278_s2s_peer_on_thread.wat` | exit 0 |
| ★ p1 | angle spelling + `:peers` names a surface with no ephemeral field | exit 1 — "missing" arm |
| ★ p2 | angle spelling + `:peers` deleted, ephemeral peer kept | exit 1 — "extra" arm |
| p3 | **`:-` form** spelling + correct `:peers` | exit 0 |
| ★ p4 | **`:-` form** spelling + wrong `:peers` | exit 1 — "missing" arm |
| ★ p5 | **`:-` form** spelling + `:peers` deleted | exit 1, and the message **names `:probe::Echo`** |

Perturbations built by `sed` off the real committed fixture, so the positive control and the negative
controls are the same file differing in one clause.

**p4 and p5 are the cross-cell neither the brief nor the rider covered.** Row 3 tested *new spelling
+ correct declaration*; row 4 tested *old spelling + wrong declaration*. Nothing tested **new
spelling + wrong declaration** — and that is the only cell that can distinguish "the structural
reader works" from "the structural reader silently returns nothing." An extraction that yielded an
empty list would pass row 4 unchanged, because the error it fires is about the *other* list.

p5 is the non-vacuity control proper: with the ephemeral field written `(Peer :- [Echo::Op
Echo::Reply])` and no `:peers` clause at all, the check must fire **and name `probe::Echo`** — a name
it can only have obtained by destructuring the form. It does. `[[feedback_a_green_test_can_prove_nothing]]`

## STOP-3, twice — and my table was the defect both times

**Site 608 — I called it EQUALITY; it is a representation BRIDGE.** `(keyword-node (string::concat
":" (keyword/to-string record-ty-decl)))` at `wat/service.wat:642-644` exists only because
`keyword/from-string` yields a raw `Value::wat__core__keyword` while `keyword/to-type-form-colon`
requires a `:wat::WatAST` node. It compares nothing. Verified by reading the site; the rider cited
`src/runtime.rs:10927-10964` and `src/edn_shim.rs:1373-1384` for the two representations. **Left
alone, correctly.** Site 963 is the same mechanism for `handle-name-decl` — the rider classified the
row I left blank, and its classification holds.

**Site 801 — right disposition, unearned reason.** The rider left `peers-surfaces` on
`keyword/to-string`, justifying it as *"a surface reference here can't carry `<>`"*. I measured
instead of accepting it. Every `:peers` clause in the corpus — `wat-tests/`, `wat-scripts/probes/`,
`tests/services/`, `wat/query.wat` — is a bare monomorphic surface keyword; **zero** are parametric.
But **nothing rejects a parametric entry**; the surrounding machinery merely assumes one cannot
appear — `peer-forms-calls` mints `"{s-str}::surface-forms"` by string interpolation, which would
produce a nonsense callable name for a parametric surface rather than a diagnostic.

So the site is **safe for ②-iii** — the codemod rewrites parametric type references, and a bare
keyword is not one, so it will not be touched — but it is safe *by what the corpus contains*, not by
what the slot forbids. ⚠ **A CENSUS SCOPES WORK IN; IT NEVER SCOPES WORK OUT.** Recorded as a live
gap, not closed.

## What the rider found that I could not have

Three, all from running the real corpus rather than a constructed probe:

1. **`type-equal?` is strict about representation** where `keyword/to-string` was deliberately
   lenient about two. `state-parent`'s default branch was a bare keyword literal evaluating to a raw
   keyword `Value`; it had to become `(keyword-node ":wat::core::Record")`. Caught only by expanding
   the real `lru-svc`, which is what reaches the default `:durable-parent` path. **This is
   `[[feedback_a_slot_with_two_implementations_is_two_slots]]` inverted** — the new door is stricter
   than the old one, so every caller that fed it the loose representation is a site, and only one of
   them was in the brief.
2. **A parametric type's args are not all `Keyword` nodes.** A bare type-variable (`K`, `V`, `T`)
   renders as `WatAST::Symbol` and is never colon-spelled, so `keyword/to-string` raises on it.
   `lru-svc<K,V> :satisfies Cache<K,V>` has type-vars in *both* arg slots and tripped this on the
   first real rebuild. The fix branches per-arg on `ast-kind`.
3. **`proto-tp`/`record-tp` are param-consumption-filtered**, not "whatever is between the angle
   brackets" — `type-params-used-in` over `:durable`. The rider's own first hibernate probe assumed
   `lru-svc::Record<K,V>`; the real value is unparameterized because `:durable` never mentions K or
   V, and `type-equal?` correctly rejected the probe's guess. A check refusing the *prober's*
   assumption is the check working.

## Expansion identity

Rider captured BEFORE/AFTER `write-forms` expansions of `wat/cache.wat`'s parametric `lru-svc<K,V>`
and `wat/query/mem.wat`'s monomorphic `mem-store`. `diff` = **0 lines, byte-identical**. It could not
use `git stash` (blocked by the auto-mode classifier) and swapped via `git show HEAD:` file-copy
instead — an honest substitution, and it said so.

## ⛔ THE DEFECT IN MY BRIEF — I ordered the load-bearing evidence destroyed

The brief says, of row 4: *"Row 4 is the one that bites. Rows 1-3 measure that the checks still
accept; only row 4 measures that they still refuse."* And then, under Your own checks:

> *"Delete any scratch `.wat` that must fail; `tests/lint/wat_scripts_fixes_load.rs` type-checks
> everything under `wat-scripts/`."*

Both true. Together they mean **the only rows that mattered were proven with artifacts I instructed
the rider to delete**, so nothing on disk records that the checks still reject. I had to rebuild all
three from scratch to score the stone.

The premise was false, and one `ls` refutes it. A must-fail probe **has an established home in this
repo** — it just is not `wat-scripts/`:

```
tests/macros/probe_arc279_format_missing_kwarg.wat          the fixture that must fail
tests/macros/probe_arc279_format__format_strict_missing_kwarg_is_macro_error.edn   the golden
tests/macros/probe_arc279_format.rs                         startup_from_file + assert_edn_matches_file!
tests/services/*.wat.bad                                    the same pattern under services
```

`tests/**` is not covered by the loader gate — that is the whole reason the `.wat.bad` convention and
these must-fail `.wat` fixtures can exist. **A negative control that CAN be kept MUST be kept**
(`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`), and this is the verbatim recurrence
of `[[feedback_a_brief_that_demands_proof_must_say_where_the_proof_lives]]` — *"my two load-bearing
rows were proven with fixtures the rider then deleted per scope, leaving the only rows that mattered
unverifiable."* Same failure, same arc, written into a brief by the self that recorded it.

★ **The rule, sharpened by this recurrence:** a brief that names a row as load-bearing must name the
**path its artifact lands at**, in the same sentence. "Prove it" and "clean up after yourself" are
one instruction with a destination, or they are a contradiction the executor resolves by deleting the
proof.

**Standing work:** the five perturbations above become permanent tests under `tests/services/`, on
the `probe_arc279_format` pattern. Briefed separately — they are this stone's missing artifact, not
a new capability.

## Also fixed in this commit (orchestrator, trivial)

`wat/service.wat:635-641` carried identity 2c's **STOP-2 as still open**, pointing at *":700 below …
a type-identity string compare … 2c does not resolve it."* That site is precisely what this stone
replaced with `type-equal?`. The comment now records STOP-2 as closed, names the door, and
distinguishes the bridge on the next line from the compare that is gone — a comment asserting a live
gap that is not live is `[[feedback_a_comment_can_ship_a_gap_as_a_law]]`.

## One live cosmetic gap, recorded not closed

Both bijection diagnostics render their *advice* in the angle spelling:

```
:peers declares surface :probe::Bogus but no :ephemeral field is typed
:wat::kernel::Peer<probe::Bogus::Op,…::Reply> — add the dialed peer as a root :ephemeral field
```

After ②-iii that advises a retired spelling. The check is spelling-agnostic; only the sentence it
prints is not.
