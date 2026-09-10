# WORKLIST — class ⑥: a variant name written WHOLE

Generated on the tree at `0c4162bdb` by:
```
grep -rnoE '"[:a-z][a-zA-Z0-9:_-]*::[A-Z][A-Za-z0-9]*::[A-Z][A-Za-z0-9]*"' --include='*.rs' src crates tests
grep -rnoE '"[:a-z][a-zA-Z0-9:_-]*::[A-Z][A-Za-z0-9]*::[A-Z][A-Za-z0-9]*"' --include='*.edn' tests wat-tests wat-scripts
```

## src + crates — 97 literals, 18 files
```
src/runtime.rs:21
src/remedy/mod.rs:16
src/check.rs:12
src/types.rs:8
src/match_arm.rs:8
src/remedy/rank.rs:4
src/io.rs:4
src/closure_extract.rs:4
src/macros/eval.rs:2
src/declare/register.rs:2
crates/wat-reader/src/identifier.rs:2
src/types/surface.rs:1
src/rete/purity.rs:1
src/remedy/distance.rs:1
src/reflect/verbs.rs:1
src/edn/render.rs:1
crates/wat-macros/src/edn_doc.rs:1
crates/wat-doc/src/print.rs:1
```

## tests — 20 literals, 8 files
```
tests/services/probe_arc278_journal_surface.rs:3
tests/macros/probe_let_splice_enum.rs:3
tests/macros/probe_do_splice_enum.rs:3
tests/comms/probe_ioreader_read_frame.rs:3
tests/diagnostics/probe_arc241_stone10_remedy.rs:2
tests/types/typed_if_match.rs:1
tests/rete/probe_constructor_meta_surface_audit.rs:1
tests/function/wat_arc170_closure_extraction.rs:1
```

## .edn goldens — 12 literals, 7 files (RE-CAPTURE, never hand-edit)
```
tests/diagnostics/probe_arc241_stone10_remedy__contract_03_ranked_multi_candidate_variant_typo.edn:3
tests/resolve/probe_arc255_register_variant_is_its_own_door__defn_then_variant.edn:1
tests/resolve/probe_arc255_register_variant_is_its_own_door__variant_then_defn.edn:1
tests/diagnostics/probe_arc241_stone10_remedy__contract_01_typo_remedy_on_variant_constructor.edn:2
tests/diagnostics/probe_arc241_stone10_remedy__contract_06_multi_remedy_multi_line_format.edn:3
tests/types/enums__unit_variant_pattern_on_tagged_variant_rejected.edn:1
tests/types/enums__tagged_variant_arity_mismatch_reported.edn:1
```

## the distinct names
```
     19 ":my::Status::Ok"
     11 ":wat::io::IOReader::ReadFrameOutcome"
     10 ":wat::core::Option::None"
      9 ":wat::core::Option::Some"
      7 ":wat::core::Result::Ok"
      7 ":wat::core::Result::Err"
      5 ":my::Status::Oks"
      3 ":wat::kernel::RecvOutcome::Message"
      2 ":wat::telemetry::Journal::WriteMetricsResponse"
      2 ":wat::runtime::Purity::Pure"
      2 ":wat::kernel::RecvOutcome::Stopped"
      2 ":wat::kernel::RecvOutcome::Lost"
      2 ":wat::kernel::RecvOutcome::Closed"
      2 ":wat::kernel::LociDiedError::Stopped"
      2 ":my::probe::Event::Deleted"
      2 ":my::probe::Event::Created"
      2 ":my::Status::Pending"
      2 ":my::Status::Oke"
      2 ":my::Status::Okay"
      2 ":my::Status::Error"
      2 ":my::Request::Push"
      1 "wat::core::Result::Ok"
      1 "wat::core::Option::None"
      1 "trading::types::PhaseLabel::Valley"
      1 "cg::Status::Active"
      1 ":wat::query::Store::PutResponse::Success"
      1 ":wat::kernel::LociDiedError::Panic"
      1 ":wat::kernel::LociDiedError::Disconnected"
      1 ":wat::core::ReadOutcome::Malformed"
      1 ":wat::core::ReadOutcome::Forms"
      1 ":trading::types::PhaseLabel::Valley"
      1 ":t::Ok1::Reply"
      1 ":t::Ok1::Op"
      1 ":t::Bad::FooResponse"
      1 ":t::Bad5::FooResponse"
      1 ":t::Bad4::FooResponse"
      1 ":t::Bad3::FooResponse"
      1 ":t::Bad2::FooResponse"
      1 ":ns::Cache::GetRequest"
      1 ":my::Status::Err"
      1 ":my::Shape::Rect"
```
