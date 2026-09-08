# SCORE — STONE N RELAND 9: the last seventeen

No commit. Floor not re-run. Lands on DRAW N RELAND-9 (`c82780149`) + RELAND-8 local save
(`424269965`).

17 → **4**. The family of generated positional heads is closed. The remaining 4 are
`sift_rules` counts-exact — diagnosed separately, not folded into the template class.

## 1. Expand, then the site (STOP-1)

Specimen: `probe_arc278_call_context` `defservice` (the freeze named
`:probe::callctx3svc::Op::-Mark` on the **whole** defservice span, lines 65–99).
`macroexpand-1` of that form, written to `/tmp/r9/callctx-expand.wat`.

Three things named `-Mark`, only one is a ctor:

| in the emission | what it is |
|---|---|
| `defenum … :-Mark []` | declaration of the internal-op variant. Leading hyphen is kebab→pascal of `-mark` with the dash kept. RELAND 8's position rule leaves this slot alone. |
| `[:probe::callctx3svc::Op::-Mark {} body]` | match arm, already map-form. |
| **`(:wat::service::Alarm … :op (:probe::callctx3svc::Op::-Mark))`** | **the construction.** List, zero args, no `{}`. Retired positional unit ctor. |

The user's body said `:op :-mark`. The template rewrites that keyword via a
source round-trip. `internal-op-repl-strs` (`wat/service.wat` ~1415) concatenated:

```
"(:" + service-op-str + "::" + variant-pascal + ")"
```

i.e. `(:<fqdn>::Op::-Tick)` — the comment even documented that spelling. That is M2
residue 1's last unquote-head: a **string-concatenated** ctor the wrap can never see.

Toy-journal expansion (the PutResponse specimen's positive sibling): generated
`Reply::WriteMetrics {:resp …}`, `Success {}`, `RequestTooLarge {:bytes :cap}` are
already maps. The remaining PutResponse/Reply positional heads were **user/fixture**
sites of generated types, not a second template hole.

## 2. Fix — map form, wherever it lives

Template (stdlib `include_str`, rebuilt):

```
"(:" … variant-pascal ")"    →    "(:" … variant-pascal " {})"
```

Re-expand after rebuild: `:op (:probe::callctx3svc::Op::-Mark {})`. Bare
`(Op::-Mark)` gone. `call_context` ×5 **PASS** (the two-param `.wat.bad` was already
the intended compile-error and stayed green).

The wrap cannot take a string-concat emitter (STOP-5). This is the authorized
template edit, sibling of RELAND 1's Path B `src/` change.

User/fixture sites of generated types the wrap also cannot see (hidden `defenum`,
`.wat.bad`, jsonl-inside-strings). Map-form, field names from the declaration:

| site | why not the wrap | edit |
|---|---|---|
| mcp `wat_mcp__{,thread_}counter_across_turns.jsonl` | EDN-inside-strings | `Outcome::Reply {:state :reply}`, `Get/IncrementResponse::Ok {:value …}` |
| `journal_surface_swap.wat.bad` | `.wat.bad` | incidental positional wrapped so the **claimed** TypeMismatch WriteMetrics vs PutResponse fires |
| `probe_diagnostic_c3_*.wat` | defenum inside `defmacro` quasiquote (RELAND 5 class) | `(~go-var {:req (~req-name :n n)})` |
| `probe_arc272_rs1_*.wat` | already a map, **wrong key** | `IncrementResponse::Ok {:value c}` → `{:count c}` (declared field) |
| `probe-m1-phantom-d.wat` | already a map, wrong key | `PoolMsg::Work {:s …}` → `{:pair …}` |
| hashmap p6 `.wat.bad` | `.wat.bad` match arms | `[:wat::core::Some/None]` → `[:wat::core::Option::Some/None]` so the intended TypeMismatch is the only error |

## 3. The one positional `(:wat::core::Option::None)` (STOP ordered)

`wat-scripts/fixes/positional-to-kwargs.wat:27` was
`(:wat::core::Option::None :wat::WatAST)` — a type argument stuffed into a unit
ctor (now `:None []`, so the arg is a positional field). →
`(:wat::core::Option::None {})`. Wrap skip-path (`wat-scripts/fixes/`); the loader
still type-checks it. Lint `every_wat_scripts_file_loads`: **PASS** (2 of 694 → 0).

## 4. `sift_rules` ×4 — separately (STOP-2)

`macroexpand-1` of `sift-rules-defsvc`: generated `Outcome::Reply {:state :reply}`,
response ctors already maps. **Not the template class.**

| arm | result |
|---|---|
| fail-closed unknown message (thread + process, defsvc + arena) | **PASS** — never enters fire-rules |
| counts-exact deductions (thread + process, defsvc + arena) | still **`disconnected`** |

The client's `RecvOutcome::Lost` of the **sift service** (assertion
`"disconnected"`). Fail-closed would also fail if init/`query-logs` were the
cause; it does not. Isolated `compile-all` + `edn/read` + `insert` + `fire-rules`
of the same two rules and one hot Temp: **EXIT=0**, production-memory has Hot and
Warn. So the rete fragment works **outside** the service. Counts-exact dies
**inside** the generated handler's 240-log (arena: 800, paged) fire-rules loop.
Not mem-store scan. Not Op::-Mark. Named remainder; not closed this strike.

## 5. `-Mark` — what built the name, then the ctor

The name is the internal-op convention: op `-mark` → variant `-Mark` (dash kept,
rest kebab→pascal). It **is** a ctor site — the Alarm `:op` rewrite above. Not a
false positive on the hyphen.

## The 17

| cluster | after |
|---|---|
| call_context ×5 | **PASS** |
| journal_surface `wrong_response_type` | **PASS** (claimed TypeMismatch now fires) |
| mcp counter ×2 | **PASS** |
| c3 macro-emits-record-def | **PASS** |
| rs1 durable-field-vector | **PASS** |
| hashmap p6 | **PASS** |
| lint `every_wat_scripts_file_loads` | **PASS** |
| metadata_of step-payload row | **PASS** — golden `:wat.core/None` → `:wat.core.Option/None` in the example's `:rule`. FQDN, not a wrong value. |
| `probe_arc296_enum_map_ctor` | **5 passed** |
| sift_rules counts-exact ×4 | still `disconnected` |

## STOP rows

| STOP | result |
|---|---|
| STOP-1 grep to find a macro-emitted construction | **held.** Expand first. Template read only after the emission named the string-concat. |
| STOP-2 sift_rules folded into the template class | **held.** Expansion already map-form; isolated fire-rules green; remainder named. |
| STOP-3 expectation changed to match a wrong value | **held.** journal_surface not recaptured to the wall. metadata golden recaptured to `Option/None` (the campaign spelling). |
| STOP-4 wall weakened | **held.** |
| STOP-5 `.wat` corpus hand-edited when the wrap could take it | **held.** Template is string-concat. jsonl / `.wat.bad` / defmacro-hidden / already-map wrong keys are not wrap work. |

## Working tree (this strike)

```
wat/service.wat                                            internal-op-repl ` {}`
tests/cli/wat_mcp__{,thread_}counter_across_turns.jsonl    Reply/Ok maps
tests/services/probe_arc278_journal_surface_swap.wat.bad   incidental wrap
tests/diagnostics/probe_diagnostic_c3_*.wat                Go {:req …}
tests/services/probe_arc272_rs1_*.wat                      {:count}
wat-scripts/fixes/positional-to-kwargs.wat                 None {}
wat-scripts/probes/arc-170/probe-m1-phantom-d.wat          {:pair}
tests/collection/probe_hashmap_ctor_vector_symmetric_p6.wat.bad
tests/reflection/probe_stone_metadata_of_whole_row__step_payload_row.edn
wat-scripts/scratch-pad/probe-reland9-*.wat                measurement
```

Do not commit unless a later brief says to.
