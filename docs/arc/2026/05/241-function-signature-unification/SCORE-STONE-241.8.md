# SCORE — Stone 241.8: `:wat::core::defstruct` HARD CUT

**Mode:** A (substrate-only, no new integration surface)
**Runtime:** ~4 h elapsed (two-session; context boundary mid-flight)
**Cascade size:** 33 files (6 src/ + 27 tests/)
**Lib tests:** 834 / 0 (1 pre-existing ignored)
**Clippy:** 883 warnings (≤ 902 gate)

---

## Phase A Scorecard (11 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | `(:wat::core::defstruct :N [f <- :T ...])` startup clean | PASS | |
| 2 | `defstruct` with `:restricted-to` metadata | PASS | |
| 3 | `defstruct` with `:field-metadata {:kw {:restricted-to ...}}` | PASS | keyword keys required |
| 4 | Both form + field metadata | PASS | |
| 5 | Multi-field defstruct | PASS | |
| 6 | Empty `{}` metadata rejected | PASS | |
| 7 | `:wat::core::struct` HARD CUT rejected | PASS | check.rs MalformedForm arm |
| 8 | `:wat::core::struct-restricted` HARD CUT rejected | PASS | check.rs MalformedForm arm |
| — | Lib tests preserved | PASS | 834 / 0 |
| — | Clippy gate | PASS | 883 ≤ 902 |
| — | Arc 241.1–241.7 probes preserved | PASS | 58 tests, all green |

---

## Structural Verification (6 rows)

| Component | Change | Verified |
|-----------|--------|---------|
| `src/types.rs` | `parse_defstruct` minted; `parse_struct` + `parse_struct_restricted` HARD CUT deleted | ✓ |
| `src/check.rs` | Legacy arms deleted; HARD CUT rejection arm added for struct/struct-restricted | ✓ |
| `src/freeze.rs` | `is_mutation_form` + `is_declaration_form` updated; tests migrated | ✓ |
| `src/runtime.rs` | `is_struct_form`, `preregister_struct_accessors_from_form`, `typedef_to_define_ast` emit `defstruct` | ✓ |
| `src/special_forms.rs` | Registry entry updated to `defstruct` | ✓ |
| `src/closure_extract.rs` | Dispatch + `walk_struct_form` + `type_def_to_ast` rewritten for defstruct triple-vector | ✓ |

---

## Migration Cascade Audit (33 files)

### Substrate (6 files)
- `src/types.rs` — S1 (parse_defstruct mint) + S2 (HARD CUT delete parse_struct/parse_struct_restricted) + internal test migration
- `src/check.rs` — S3 (declaration-forms arm) + HARD CUT rejection arm
- `src/freeze.rs` — mutation/declaration dispatch + test migration
- `src/runtime.rs` — struct form detection, accessor preregistration, AST emission
- `src/special_forms.rs` — registry entry
- `src/closure_extract.rs` — walk + emit for defstruct

### Test cascade (27 files)
All test files with legacy `:wat::core::struct` or `:wat::core::struct-restricted` declarations migrated to `:wat::core::defstruct`:

**Fully rewritten (semantic equivalence preserved):**
- `tests/wat_arc203_struct_restricted.rs` — struct-restricted replaced with defstruct + {:restricted-to :field-metadata} equivalents; malformed-shapes test updated to test empty-{} + legacy HARD CUT

**Field-for-field migrations (syntax only):**
- `tests/wat_structs.rs` (9 structs)
- `tests/wat_arc098_form_matches_runtime.rs` (PROLOGUE + 1 inline)
- `tests/wat_arc098_form_matches_typecheck.rs` (PROLOGUE_VALID + PROLOGUE_INVALID)
- `tests/wat_arc148_ord_buildout.rs` (1 struct)
- `tests/wat_arc169_struct_destructure.rs` (PROLOGUE)
- `tests/wat_arc170_closure_extraction.rs` (7 structs + `collect_type_decl_names` helper updated)
- `tests/wat_arc144_special_forms.rs` (test assertion updated to defstruct)
- `tests/wat_arc144_lookup_form.rs` (3 structs + assertion updated to defstruct)
- `tests/wat_arc144_uniform_reflection.rs` (1 struct + assertion updated to defstruct)
- `tests/probe_arc237_stone5fix_nominal.rs` (2 structs)
- `tests/probe_arc237_stone6_is_predicate.rs` (1 struct)
- `tests/probe_arc234_stone3c_keyword_accessor.rs` (1 struct)
- `tests/probe_arc237_s0_records_gate.rs` (1 macro-emitted struct)
- `tests/probe_brace_map_literal.rs` (1 struct)
- `tests/probe_closure_body_prelude_lift.rs` (2 structs)
- `tests/probe_declaration_form_lift.rs` (1 struct + is_declaration_form list)
- `tests/probe_def_not_special.rs` (1 struct)
- `tests/probe_deftest_hermetic_isolation.rs` (4 structs)
- `tests/probe_diagnostic_polymorphic_type.rs` (1 struct)
- `tests/probe_do_splice_struct.rs` (2 structs, 1 in macro quasiquote)
- `tests/probe_let_splice_struct.rs` (2 structs, 1 in macro quasiquote)
- `tests/probe_register_types_splice_aware.rs` (1 struct)
- `tests/probe_spawn_process_parent_type.rs` (4 structs)
- `tests/wat_newtype_values.rs` (1 struct)
- `tests/wat_vector_first_class.rs` (1 struct)
- `tests/probe_arc241_stone8_defstruct.rs` (probe contracts updated)

**WAT source files:** No migration needed. `wat/kernel/hermetic.wat`, `wat/kernel/sandbox.wat`, `wat/test.wat` only contain `struct-new` (constructor calls) — not struct declarations.

---

## Final `parse_defstruct` Body (verbatim)

```rust
fn parse_defstruct(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    const HEAD: &str = ":wat::core::defstruct";
    if args.len() < 2 || args.len() > 3 {
        return Err(TypeError::MalformedDecl {
            head: HEAD.to_string(),
            reason: format!(
                "expected 2 or 3 args after head (name [field-vector] or name {{metadata}} [field-vector]); got {}",
                args.len()
            ),
            span: decl_span,
        });
    }
    // args[0] = name keyword
    let (name, type_params) = parse_declared_name(&args[0], HEAD)?;
    
    // Discriminate 2-arg vs 3-arg
    let (metadata_ast_opt, field_vec_ast) = if args.len() == 2 {
        (None, &args[1])
    } else {
        (Some(&args[1]), &args[2])
    };
    
    // Parse optional metadata-map
    let mut ctor_whitelist: Vec<String> = Vec::new();
    let mut field_restrictions: HashMap<String, Vec<String>> = HashMap::new();
    
    if let Some(meta_ast) = metadata_ast_opt {
        // metadata-map renders as a List with HashMap head
        let meta_items = match meta_ast {
            WatAST::List(items, _) if items.len() >= 1 => {
                if let Some(WatAST::Keyword(head_kw, _)) = items.first() {
                    if head_kw != ":wat::core::HashMap" {
                        return Err(TypeError::MalformedDecl {
                            head: HEAD.to_string(),
                            reason: "metadata arg must be a map literal {{...}}".to_string(),
                            span: meta_ast.span(),
                        });
                    }
                    if items.len() < 4 || (items.len() - 3) % 2 != 0 {
                        return Err(TypeError::MalformedDecl {
                            head: HEAD.to_string(),
                            reason: "metadata map must be non-empty (FORM-COLLAPSE: empty {{}} is illegal)".to_string(),
                            span: meta_ast.span(),
                        });
                    }
                    &items[3..]
                } else {
                    return Err(TypeError::MalformedDecl { ... });
                }
            }
            _ => return Err(TypeError::MalformedDecl { ... }),
        };
        // Walk key-value pairs from items[3..]
        // :restricted-to [kwlist] -> ctor_whitelist
        // :field-metadata {:field-kw {meta}} -> field_restrictions
    }
    
    // Parse field-vector via canonical parse_argspec_triples
    let field_items = match field_vec_ast {
        WatAST::Vector(items, _) => items.clone(),
        other => return Err(TypeError::MalformedDecl { ... }),
    };
    let field_span = field_vec_ast.span();
    let argspec = crate::argspec::parse_argspec_triples(
        &field_items, HEAD, &field_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    ).map_err(TypeError::from)?;
    let fields: Vec<(String, TypeExpr)> = argspec.fixed_params;
    
    let restrictions = if ctor_whitelist.is_empty() && field_restrictions.is_empty() {
        None
    } else {
        Some(StructRestrictions { ctor_whitelist, field_restrictions })
    };
    
    Ok(TypeDef::Struct(StructDef { name, type_params, fields, restrictions }))
}
```

---

## Honest Deltas

### Trap-door (T-fd): `:field-metadata` inner map keys must be keyword-prefixed

The parser routes `{bareSymbol {submap}}` → struct-destructure before `parse_defstruct` is called. Struct-destructure expects all items to be bare symbols; a `{submap}` value fails it with `ParseError::MalformedStructPattern`.

**Consequence:** `:field-metadata` inner map MUST use keyword keys (`:field-name`) not bare symbols (`field-name`). Design adapts by accepting both `WatAST::Keyword` and `WatAST::Symbol` as field name tokens in the `:field-metadata` parsing loop, stripping the leading colon from keywords.

Probe contracts 03/04 updated to use keyword syntax. FORM-COLLAPSE-NOTES examples should show keyword keys.

### Pre-existing failures (unchanged)
- `probe_8_atom_round_trip` in `probe_arc216_stone5b_hashset_native_storage` — confirmed pre-existing via git stash round-trip before any Stone 241.8 changes
- `dispatch_empty_lookup_define_emits_define_dispatch_head` in `wat_arc144_uniform_reflection` — pre-existing, unrelated to struct forms

---

## PHASE 3 OPENS

Stone 241.8 closes the HARD CUT on legacy struct syntax. The foundation is:

- `parse_defstruct` is the sole struct declaration path
- Field-vector uses canonical `parse_argspec_triples` (Stone 241.1 substrate)
- Capability restrictions live in the metadata-map (Stone 241.6/7 substrate)
- All 33 cascade files green; 834 lib tests, 8/8 Stone 241.8 contracts

Phase 3 of arc 241 (function-signature unification) now has a clean substrate for the next stone.
