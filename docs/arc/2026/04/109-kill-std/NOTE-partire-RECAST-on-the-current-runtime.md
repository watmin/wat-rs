# NOTE — `partire` RE-CAST on the current `runtime.rs` (25,997). **This supersedes the 2026-08-31 map.**

> Cast 2026-09-01 at the builder's direction — *"re-cast partire on the current runtime.rs first"* —
> after four stones changed the file by 8,155 lines and made every range in the original cast stale.
> Ward re-fetched from the signed MCP and embedded verbatim. **No stone drawn, nothing moved.**
>
> ⛔ `[[NOTE-partire-on-the-two-megafiles-runtime-and-check]]`'s `runtime.rs` half is **STALE**. Its
> `check.rs` half stands. This file is the live map.

## What changed in the CAST, not just the file

The original returned line RANGES, and `[[NOTE-every-partire-range-contains-a-neighbour]]` measured
the consequence: six modules, six intruders. This cast was instructed to **enumerate every item by
name with its own line, and to name EXCLUDED items with caller evidence.** It did, and the difference
shows immediately — it caught intruders the first cast's ranges hid:

```
holon::outcome   EXCLUDES no_field_names · builtin_enum_variant_names
                 → consumed by io, services, intrinsic/mod, stream, rete/purity, host, edn, declare
record           EXCLUDES eval_retag_op (7782)
                 → sits between eval_variant and eval_struct_field; sole caller is intrinsic/kernel/serve.rs
numeric          EXCLUDES dispatch_rete_op (4396)
                 → between i64_mod_op and bigint_div. The SAME intruder the first cast flagged,
                   re-confirmed on current lines.
```

★★ **And it read the `src/intrinsic/*` edge headers as ground truth for home names** rather than
inventing them — several say outright which `runtime.rs` function they delegate to and that it is
"pre-existing" or "in place, unmoved."

## ⛔ ITS SHARPEST FINDING INDICTS A SHIPPED STONE — the numeric home is HALF-MIGRATED

`src/numeric/` **reaches back into `runtime.rs` for its own domain**, verified:

```
src/numeric/arith.rs:20   use crate::runtime::{collapse_bigrational, to_bigrational, I64ArithErr, …}
src/numeric/ops.rs:17     use crate::runtime::{bigint_component_to_value, …}
```

Eighteen numeric-tower items were left behind: `i64_{add,sub,mul,div,quot,rem,mod}_op` (4287–4366),
`bigint_div` (4482), `to_bigrational` (4506), `collapse_bigrational` (4521), `rational_div` (4534),
`bigint_component_to_value` (4555), `f64_{add,sub,mul,div,max,min}_op` (4582–4617).

⚠ **The numeric stone's own rider reported this at the time** — *"four sibling helpers … were not in
the brief's 24-function list, so they were left in place"* and *"`I64ArithErr` is numeric-tower
vocabulary that did NOT move … flagging as a candidate."* It was recorded as an honest delta and not
acted on. **My function list was the defect, and an independent cast has now found the same gap
from the other direction.** `[[feedback_a_lesson_learned_and_then_dropped]]`

## The map — modules proposed, each enumerated by item in the cast's own report

| # | module | home | items | severity |
|---|---|---|---|---|
| 1 | numeric completion | `src/numeric/` (exists) | 18 | L1 — a visibly half-finished extraction |
| 2 | `record` | `src/record/` (new; edge `intrinsic/record.rs` exists and says "no body moves… yet") | 17 | L1 |
| 3 | kernel family | `src/kernel/` (exists) — **7 sub-modules mirroring the 7 `intrinsic/kernel/*.rs` edge files** | ~30 | L1 |
| 4 | died-error / outcome vocabulary | ⬜ **practitioner's-call** | ~55 | L1 |
| 5 | `holon::outcome` completion | `src/holon/outcome.rs` (exists) | 12 | L1 |
| 6 | `option` / `result` | `src/option/`, `src/result/` (edges exist) | 7 | L2 |
| 7 | purity classifier | `src/rete/purity.rs` (exists) | 2 | L1 |

★ **Item 3 is the shape worth noticing:** the cast refuses to dump the kernel verbs into one blob,
because `src/intrinsic/kernel/` is *already split by decision* at the edge — message · resource ·
identity · ambient · abort · source · serve. The impl should mirror the edge, and the cast matched
each impl fn to the edge file that delegates to it. It also found the scatter: `eval_kernel_here` and
`eval_kernel_call_site` are **~4,000 lines apart** despite serving the same three-delegate edge file.

## ⬜ Item 4 is deliberately NOT assigned a home

The died-error/outcome cluster (12801–14190) is consumed by **kernel, process, distribution AND
host** — `src/kernel/spawn.rs`, `src/process/verbs.rs`, `src/distribution/{mod,mcp}.rs`,
`src/host/test_runner.rs`, plus `src/intrinsic/kernel/error.rs`'s four entry points. The cast says
plainly that calling it "kernel" would **repeat the `peer_protocol` mistake** — fusing a shared
vocabulary into one caller's home. Candidate homes: `kernel::error`, a standalone `error_outcome`, or
an extension of `src/edn/contract.rs`. **Named, not picked.**

## ★★ THE LEAVE THAT MATTERS — the eval spine is one concern at length

The cast returns a **defensible LEAVE** for the residual core: `eval_tail` · `eval_inner` · `eval` ·
`eval_list` · `dispatch_keyword_head{,_value}` · `eval_let`/`eval_do`/`eval_if` · `eval_apply` ·
equality/comparison · quote/quasiquote · `apply_function` — *"the interpreter's central special-form
dispatch… the load-bearing evaluator, not several concerns wearing one name."* It proposes a
`rune:partire(historical-shape)` rather than a cut, and refuses `eval_conforms`/`eval_subtype`
(**zero external callers** — no independent test surface distinct from the spine).

⚠ **So the megafile campaign has a floor, and this names it.** After items 1–7 the residue is the
evaluator itself plus ~6,536 lines of in-file `mod tests`, and no further honest cut is on offer.

## Refused cuts — each on DOCUMENTED grounds the cast went and read

1. **math/stat two-layer split** — `src/intrinsic/math.rs`'s header records Stone HOME-10 measuring
   and *declining* it: shim-only, "arity-check → unwrap → `f(x)` → rewrap".
2. **`eval_edn_validate`** — `src/intrinsic/edn.rs`'s header states its algorithm "stays there."
3. **`program_dim` / `require_encoding_ctx` into `src/holon/`** — blocked by `src/holon/codec.rs`'s
   stricter two-layer contract, and disqualified anyway by a non-holon consumer (`intrinsic/config.rs`).
4. **`eval_conforms` / `eval_subtype`** — zero external callers.

★★★ **A cast that goes and reads the prior rulings before proposing is the difference between a seam
map and a wish list.** Three of these four are refusals it could only make by opening the edge files
and finding a stone that already ruled.
