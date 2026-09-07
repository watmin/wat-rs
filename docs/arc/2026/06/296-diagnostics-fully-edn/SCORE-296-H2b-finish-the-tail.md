# SCORE — 296 H-2b: finish H-2's tail

No commit. Floor and clippy left to the orchestrator. H-2's writer was not touched.

Contract held: **in a golden, ONLY the variant tag and its body shape may change. LAYOUT MUST NOT MOVE.**

---

## Class 1 — layout restored

Every tracked `.edn` was rebuilt from `git show HEAD:<path>` with an in-place tag+body rewrite (vector `[…]` → map `{…}`, inner whitespace kept). No parse+`write_pretty`.

- **0 files gained lines vs HEAD.** The 40 that H-2's rewriter pretty-printed are back to HEAD's line count.
- tests goldens with leftover old `#ns.Type/Variant [` : **0**.
- Exemplar `special_form_lookup_define_smoke` is one line again:

```
#wat.core/Option.Some {:value (#wat.ast/Keyword {:path ":wat::core::__internal/special-form"} :wat.core/if)}
```

- `pprintln_doc_row__step_payload.edn`: 7/7, tag+body only (`#wat.runtime.Purity/Pure []` → `#wat.runtime/Purity.Pure {}`, etc.).
- `wat_repl__bad_then_good_fault.edn`: Option Some kept its multi-line slot; `[`/` ]` became `{` / `:value` / `}`.

A handful of H-2 pretty-only files (no variant tag) restored **identical to HEAD** and dropped out of the diff. Tracked `.edn` vs HEAD is now **307** (was 311) plus two demo session files (below). Untracked `tests/value/probe_arc278_read_foreign__keys_survive.edn` left alone.

**Keyword-stone replacements were NOT re-applied.** Measured: the special_form binary still emits `#wat.ast/Keyword {:path ":wat::core::__internal/special-form"}`, not `:wat.core.__internal/special-form`. Applying the stone's spelling made the verbatim assert red. STOP-2: the golden follows the binary. Named — the keyword stone's intended spelling is still not what this path prints.

## Class 2 — EDN inside strings / comments, by hand

Derived (not copied): old `#ns.Type/Variant` hits in `.wat` were **comments**, except one string that is a **record** (`#dos.Bag/PutRequest {:items [1 2 3]}` — left).

No `.wat` FORM still carries an old tag (STOP-3 did not fire). The wat-fix remains identity; R21 not violated.

Comments updated in tests/, wat-scripts/scratch-pad/, wat-scripts/demos/, wat-scripts/probes/, wat-scripts/grep/, wat-scripts/perf/, crates/wat-edn/demo/, docs/arc/278 probes, wat-tests/edn/roundtrip.wat.

**Left on purpose:**
- `wat-scripts/fixes/variant-vector-to-tagged-map.wat` — documents the FROM form.
- `wat/spawn.wat:219` `#wat.bracket.PoolMsg/Setup` — frozen stdlib comment (`include_str`).
- `wat/service.wat` `#dos.Bag/PutRequest {:items …}` — record.

Demo **session.edn** (rewriter never walked them; they are live stdin) converted with declared field names:

```
#repl/Cmd.Bump {:by 5}     #repl/Cmd.Show {}     #repl/Cmd.Quit {}
#proto/Frame.Chunk {:text "…"}    #proto/Frame.SectionDone {:count 2}
```

Scratch-pad comments whose payload keys were unknown kept a vector body after a tag-only flip (`#g/End.Known [7 26]`, `#wat.eval/FormOutcome.Evaluated [7]`). Named, not invented.

## Class 3 — prose, @example, needles

Docs and assertion messages moved off the dead wire in `src/edn/render.rs` (module table + coerce table + test comments only), `src/types.rs` (Option wire comment), `src/intrinsic/edn.rs` (@example `Kind.Click {:n 42}`), `src/intrinsic/record.rs`, `src/edn/derive_tests.rs`, `src/distribution/mod.rs`, and the probe/test files that taught `#wat.core.Option/… []`.

H-2 probe header now shows the landed discriminator:

```
record  :usr::Shape::Circle  →  #usr.Shape/Circle {:r 2}
enum    :usr::Shape Circle   →  #usr/Shape.Circle {:r 2}
```

`examples/console-demo/tests/smoke.rs` needle updated to the writer; **console-demo smoke PASSES**.

**`src/comms/mod.rs:522-525` LEFT.** Opaque `to_wire`/`from_wire` STRING PAYLOAD — the claim is passthrough, not tag shape. Comment now says so. Updating the string would still prove the same thing and would look like a writer change.

## Named — a live bypass still mints the old LociDiedError tag

The orchestrator's "no live substrate site still mints an old-form tag" missed a hand-built EDN path that does **not** go through `variant_tag`:

```
src/process/verbs.rs:80
  Tag::ns("wat.kernel.LociDiedError", "StartupError") + Vector [cause]
```

`edn_is_loci_died_chain` / `loci_died_error_from_reason` still key on `tag.namespace() == "wat.kernel.LociDiedError"`. Writer-emitted `Value::Enum` LociDiedError is `#wat.kernel/LociDiedError.StartupError {:error …}` and would **not** match that decoder.

- `wat_cli::startup_error_bubbles_up_as_exit_3` needle left as `#wat.kernel.LociDiedError/StartupError` — it matches this bypass.
- `wat_cli::freeze_time_panic` needle is already `#wat.kernel/LociDiedError.Panic` — that path goes through the writer.

Not moved this strike (writer-adjacent mechanism, not corpus). The two CLI needles are now a split, honestly.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 layout | **held.** 0 line-gain vs HEAD. Sample diffs are tag+body. |
| STOP-2 verbatim→structural | **not taken.** special_form still `assert_eq!` + `include_str`. |
| STOP-3 hand-edit of a `.wat` FORM | **not taken.** |
| STOP-4 H-2 probe | **3/3 green**, `#[ignore]` count **0**. |

## Targeted checks (executor)

```
cargo test --release --test types -- probe_arc296_h2
  3 passed  (tag / body / unit-vs-record)

cargo test --release --test reflection -- special_form_lookup_define_smoke primitive_empty type_lookup_define_smoke
  3 passed  (layout exemplar + keyword-stone siblings, Keyword record kept)

cargo test --release --test value -- option_result_tagged / foreign_variant_keys_survive / a0 option+user_unit
  13 + 1 passed

cargo test --release --lib -- unit_variant_empty_vector_is_refused arc170_1fi_coerce_option arc170_1fi_coerce_result
  5 passed  ([] refused; new map accepted)

cargo test --release --test cli -- freeze_time_panic
  passed

cargo test --release -p console-demo --test smoke
  passed
```

Floor not run. Clippy not run.

## Files (this stone)

```
tests/**/*.edn                                              layout restore from HEAD
wat-scripts/demos/{stdio-service,stream-protocol}/session.edn   known-key demo input
*.wat comments (not wat/*.wat, not the wat-fix FROM form)
src/edn/render.rs                                           prose/table only
src/{types,intrinsic/edn,intrinsic/record,distribution,edn/derive_tests,comms}.rs
tests/**/*.rs probes/needles
examples/console-demo/tests/smoke.rs
```

Writer (`variant_tag`, Enum/Option/Result write/read, `ForeignVariantValue.names`) untouched this strike.
