# Arc 239 — SCORE

**Stone:** span-arity test-rot sweep

## Files touched

### Class B fix (1× E0026)
- `tests/probe_arc234_stone2b_defrecord_macro.rs` — line 65: removed `holon_form: _` from `wat__Record` destructure; base variant has only `{ class_fqdn, struct_form }`.

### Class A fixes (20× E0061 → E0308 → resolved)

The fix pattern is: `bind(name, Span::unknown(), value.into())`.  
Note: the initial pass inserted `Span::unknown()` correctly but the third arg needed `.into()` to convert `Value` to `TrackedValue` (compiler surfaced E0308 mismatched types; suggestion applied verbatim).

| File | Calls fixed | Span import added? |
|---|---|---|
| `tests/wat_arc208_process_io_result.rs` | 5 | pre-existing (`use wat::span::Span;` at line 39) |
| `tests/wat_arc170_stone_c1_threadpeer.rs` | 6 | added (`use wat::span::Span;`) |
| `tests/wat_process_peer_ipc_round_trip.rs` | 1 | pre-existing |
| `tests/wat_arc170_stone_a_drain_and_join.rs` | 2 | used `wat::span::Span::unknown()` inline (file already used this form) |
| `tests/probe_counter_actor_process_diag.rs` | 1 | pre-existing |
| `tests/wat_arc170_typed_channel_pipes.rs` | 5 | pre-existing |

**Total Class A fixed:** 20 sites across 6 files.  
**Total Class B fixed:** 1 site.  
**Grand total:** 21 errors resolved.

## BRIEF ledger vs. reality

The BRIEF listed 15 affected files. On `--workspace --keep-going` the build reported 21 errors across 7 test targets (not 15). The remaining 8 BRIEF files (`probe_arc214_*`, `probe_arc216_*`, `probe_arc234_stone4_*`, `probe_arc237_*`, `probe_sender_receiver_*`, `crates/wat-cli/tests/wat_cli.rs`) had zero `.bind()` calls — only warnings (unused functions/variables). They compiled clean and were not touched.

## Final build result

```
cargo build --release --tests --workspace --keep-going
→ 0 errors
```

## Final test result

```
cargo test --release --workspace --no-fail-fast
→ 12 targets with runtime failures
```

All runtime failures are **pre-existing** — confirmed by stash-round-trip verification. The same 12 targets fail on the unmodified branch (before any changes in this stone):

- `probe_arc216_stone5c_hashmap_native_storage` — `probe_12_atom_roundtrip`
- `probe_arc234_stone15_namespace_promotion` — `probe_5_class_fqdn_extraction_post_rename`
- `probe_lifeline_pipe_proof` — `lifeline_pipe_zero_orphans_across_100_trials`
- `test` (main) — 5–6 `deftest_wat_rs_*` / `deftest_wat_tests_*` tests
- `wat_arc144_uniform_reflection` — `dispatch_length_lookup_define_emits_define_dispatch_head`
- `wat_arc201_structured_signature_types` — `signature_of_defn_foldl_emits_structured_parametric_and_fn`
- `wat_arc220_list` — `list_first_returns_some`, `list_conj_prepends`
- `wat_bundle_capacity` — `try_propagates_bundle_err_across_function_boundary`
- `wat-cli` — presence/echo/program tests
- `wat-lru`, `wat-telemetry`, `wat-telemetry-sqlite`

No STOP triggered. All 21 errors were Class A or Class B as categorized. No structural errors from in-flight arcs (170/212/213/214) appeared.

## Closure follow-up (tagged)

Add `cargo build --release --tests --workspace` to the green-gate so span-arity drift cannot silently re-rot behind the `--lib` metric. This is the failure-engineering class-fix (not this stone).
