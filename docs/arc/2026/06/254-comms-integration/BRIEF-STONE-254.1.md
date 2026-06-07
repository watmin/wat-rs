# BRIEF — Stone 254.1: the channel-payload portability gate

## The work (one paragraph)

Channels carry messages, not resources. Add a type-level portability predicate
`is_portable_type` — factored from the SAME classification the value-level
encoder already uses — and gate `make-bounded-channel`'s payload type with it, so
a non-portable payload (e.g. a struct carrying a `Sender`) is rejected at check
time with a clear diagnostic. This closes the composite portability gap proven by
the RED-at-HEAD probe; un-ignore that probe and it goes green. Record types are
portable by construction; struct types are portable iff every field is portable.

## Read in order (the rooms)

1. `src/closure_extract.rs:1476` `encode_value_to_ast` + the **portable arms**
   (`:1492-1544`: bool/i64/f64/u8/String/keyword/Uuid/Char/List + recursion) and
   the **non-portable arms** (`:1729-1745`: `Sender`/`Receiver`/handles/closures →
   `NonPortableCapture`). This is the value-level source-of-truth for *what is
   portable*; the new type-level predicate mirrors EXACTLY this set.
2. `src/check.rs:10506-10518` — the `make-bounded-channel` payload extraction:
   `WatAST::Keyword(k) => parse_type_expr(k)` yields `t` (the payload `TypeExpr`).
   The gate inserts on the `Ok(t)` branch.
3. `src/types.rs:1987` `TypeDef::Record(RecordDef)` and the `defstruct` path
   (`:5`, `parse_defstruct`) — how Record vs Struct are represented in `TypeEnv`,
   and how to read a struct's field types (for the recurse).
4. `src/check.rs:11819` `is_holon_or_record` — existing Record-recognition shape to mirror.
5. `tests/nursery/probe_arc254_channel_payload_portable.rs` — the probe. The
   `#[ignore]`'d `channel_of_struct_with_opaque_field_must_be_rejected` is the
   load-bearing row; un-ignore it. The other two tests stay green.

## Implementation sketch

```rust
/// A type is portable (wire-serializable / universe-crossable) iff it can be
/// reconstructed in a fresh world. Mirrors closure_extract's value-level split.
fn is_portable_type(ty: &TypeExpr, types: &TypeEnv) -> bool {
    let ty = reduce(ty, &Subst::new(), types); // canonicalize aliases (check.rs:12837)
    match &ty {
        // atoms + reconstructible scalars
        TypeExpr::Path(p) => match p.as_str() {
            // portable scalars: i64/f64/bool/u8/String/keyword/Uuid/Char/nil ...
            // a user TYPE name → look up in TypeEnv:
            //   TypeDef::Record  => true (holon-representable by construction)
            //   TypeDef::Struct  => every field type is_portable_type(..) (recurse)
            //   TypeDef::Enum    => every variant payload portable (recurse)
            // NON-portable paths: Sender/Receiver/ProgramHandle/HandlePool/
            //   ChildHandle/IOReader/IOWriter/... → false
            _ => /* classify per above */,
        },
        // portable containers iff element(s) portable
        TypeExpr::Parametric { head, args } => /* Vector/List/Option/HashMap/Tuple-like:
            head is a portable container AND all args portable; Sender<_>/Receiver<_> → false */,
        TypeExpr::Tuple(elems) => elems.iter().all(|e| is_portable_type(e, types)),
        TypeExpr::Fn { .. } => false, // closures never cross
        TypeExpr::Var(_) => false,    // unresolved → conservatively non-portable
    }
}
```

Gate (check.rs:10507, the `Ok(t)` arm):
```rust
WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
    Ok(t) => {
        if !is_portable_type(&t, types) {
            local_errors.push(CheckError { span: args[0].span().clone(), kind:
              CheckErrorKind::MalformedForm { head: form.into(),
                reason: format!("channel payload type {} is not portable — channels carry \
                  messages, not resources; a payload must be wire-serializable (records, \
                  scalars, and portable containers; not Sender/Receiver/handles/closures)", k),
                remedies: vec![] } });
        }
        t
    }
    Err(_) => { /* existing not-a-valid-type-keyword arm, unchanged */ }
}
```

## Blast radius

`src/check.rs` (the gate + `is_portable_type`, or a sibling fn near it) +
whatever `TypeEnv` accessor reading struct fields requires. NO new types, NO
runtime change, NO change to `closure_extract.rs` (read-only reference). One
test un-ignored. `is_portable_type` stays in `check.rs` for now — it lifts to a
warded `src/portability/` home in a later stone when the concern is unified
(per DESIGN §contract; not this stone).

## STOP triggers (rejection criteria — surface, do not work around)

1. **STOP** if the Record-vs-Struct-vs-Enum distinction cannot be read cleanly
   from `TypeEnv`/`TypeDef` — report the actual `TypeDef` shape; do not guess a
   classification.
2. **STOP** if gating `make-bounded-channel` reddens EXISTING tests whose channel
   payload is a legitimately-portable struct/record — that means `is_portable_type`
   is wrongly rejecting an all-portable composite; FIX the predicate to accept it.
   Do NOT loosen the gate to a blanket allow.
3. A channel payload that is a struct/record carrying a portable field MUST still
   be accepted (the gate rejects only genuinely non-portable payloads).

## Cite

The disconfirming probe `tests/nursery/probe_arc254_channel_payload_portable.rs`
is the proven RED-at-HEAD evidence: `channel_of_struct_with_opaque_field_must_be_rejected`
type-checks clean today; this stone makes it reject + un-ignores it.
