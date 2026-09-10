//! STONE: `metadata-of` answers with the whole row.
//!
//! `from_metadata(metadata-of <fqdn>)` succeeds and equals the entry's own doc.
//! Both branches emit the same shape for every shared key. The six doc-contract
//! keys are populated, not merely present. `:ret` is the pair `[type, description]`.
//!
//! Negative control (row 11): delete `:examples` from the returned map →
//! `from_metadata` goes RED with `MissingExample`. Un-arm any of the six `put`s
//! and the round-trip test below goes red the same way.

use std::collections::HashMap;
use std::sync::Arc;

use wat::freeze::call_beside_value;
use wat::runtime::Value;
use wat::WatAST;
use wat_doc::DocError;

fn metadata_of(fn_name: &str) -> HashMap<Value, Value> {
    match call_beside_value(file!(), fn_name).expect("eval metadata-of") {
        Value::Option(opt) => match &*opt {
            Some(Value::wat__std__HashMap(m)) => (**m).clone(),
            other => panic!("metadata-of must be Some(HashMap); got {other:?}"),
        },
        other => panic!("metadata-of must return Option; got {other:?}"),
    }
}

fn get<'a>(map: &'a HashMap<Value, Value>, key: &str) -> &'a Value {
    map.iter()
        .find_map(|(k, v)| match k {
            Value::wat__core__keyword(s) if s.as_str() == key => Some(v),
            _ => None,
        })
        .unwrap_or_else(|| panic!("metadata map missing key {key}"))
}

fn has_key(map: &HashMap<Value, Value>, key: &str) -> bool {
    map.keys().any(|k| match k {
        Value::wat__core__keyword(s) => s.as_str() == key,
        _ => false,
    })
}

fn value_to_doc_ast(v: &Value) -> WatAST {
    match v {
        Value::String(s) => WatAST::string(s.as_str()),
        Value::i64(n) => WatAST::int(*n),
        Value::wat__core__keyword(k) => WatAST::keyword(k.as_str()),
        Value::Enum(ev) => WatAST::keyword(wat_reader::identifier::compose_variant(
            &ev.type_path,
            &ev.variant_name,
        )),
        Value::Vec(items) => WatAST::vector(items.iter().map(value_to_doc_ast).collect()),
        Value::wat__WatAST(a) => (**a).clone(),
        Value::Unit => WatAST::nil(),
        other => panic!("unexpected metadata-of value for from_metadata: {other:?}"),
    }
}

fn metadata_map_to_watast(map: &HashMap<Value, Value>) -> WatAST {
    let pairs = map
        .iter()
        .map(|(k, v)| {
            let key = match k {
                Value::wat__core__keyword(s) => WatAST::keyword(s.as_str()),
                other => panic!("non-keyword metadata key: {other:?}"),
            };
            (key, value_to_doc_ast(v))
        })
        .collect();
    WatAST::map(pairs)
}

fn rust_doc_before_attr(src: &str, attr: &str) -> String {
    let pos = src.find(attr).unwrap_or_else(|| panic!("attribute `{attr}` not found"));
    let mut docs = Vec::new();
    for line in src[..pos].lines().rev() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("///") {
            docs.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
        } else if trimmed.starts_with("#[")
            || trimmed.starts_with("#!")
            || trimmed.starts_with("//")
        {
            continue;
        } else {
            break;
        }
    }
    docs.reverse();
    docs.join("\n")
}

fn defined_in_variant(map: &HashMap<Value, Value>) -> &str {
    match get(map, ":defined-in") {
        Value::Enum(ev) => ev.variant_name.as_str(),
        other => panic!(":defined-in must be Enum; got {other:?}"),
    }
}

fn assert_shape_vec_of_triples(v: &Value, key: &str) {
    match v {
        Value::Vec(items) => {
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::Vec(fields) if fields.len() == 3 => {}
                    other => panic!("{key}[{i}] must be a 3-vector; got {other:?}"),
                }
            }
        }
        other => panic!("{key} must be a Vec; got {other:?}"),
    }
}

fn assert_shape_examples(v: &Value) {
    match v {
        Value::Vec(items) => {
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::Vec(fields) if fields.len() == 1 || fields.len() == 2 => {
                        assert!(
                            matches!(&fields[0], Value::wat__WatAST(_)),
                            ":examples[{i}] expr must be WatAST; got {:?}",
                            fields[0]
                        );
                    }
                    other => panic!(":examples[{i}] must be a 1- or 2-vector; got {other:?}"),
                }
            }
        }
        other => panic!(":examples must be a Vec; got {other:?}"),
    }
}

/// Row 2 + 5 + 6 + 8 + 9 — six keys present, vectors populated, :ret is the pair,
/// :yields/:syntax absent.
#[test]
fn registry_lookup_carries_the_six_and_ret_is_the_pair() {
    let map = metadata_of(":user::step-payload-metadata");

    assert!(has_key(&map, ":args"), "registry lookup must carry :args");
    assert!(has_key(&map, ":examples"), "registry lookup must carry :examples");
    assert!(has_key(&map, ":see"), "registry lookup must carry :see");
    assert!(
        !has_key(&map, ":yields"),
        ":yields must NOT be emitted from the registry branch"
    );
    assert!(
        !has_key(&map, ":syntax"),
        ":syntax is outside the doc-row contract"
    );

    match get(&map, ":args") {
        Value::Vec(items) => assert_eq!(
            items.len(),
            5,
            ":wat::rete::step-payload declares five @args; got {}",
            items.len()
        ),
        other => panic!(":args must be a Vec; got {other:?}"),
    }
    match get(&map, ":examples") {
        Value::Vec(items) => {
            assert!(
                !items.is_empty(),
                ":examples must be populated, not an empty vector"
            );
            match &items[0] {
                Value::Vec(fields) => match &fields[0] {
                    Value::wat__WatAST(ast) => match ast.as_ref() {
                        WatAST::List(_, _) => {}
                        other => panic!("example expr must be the real form (a List); got {other:?}"),
                    },
                    other => panic!("example expr must be WatAST; got {other:?}"),
                },
                other => panic!(":examples[0] must be a vector; got {other:?}"),
            }
        }
        other => panic!(":examples must be a Vec; got {other:?}"),
    }

    match get(&map, ":ret") {
        Value::Vec(items) if items.len() == 2 => {
            match &items[0] {
                Value::wat__core__keyword(k) => assert_eq!(
                    k.as_str(),
                    ":wat::rete::DerivationStep",
                    ":ret type half"
                ),
                other => panic!(":ret[0] must be the type keyword; got {other:?}"),
            }
            match &items[1] {
                Value::String(s) => assert_eq!(
                    s.as_str(),
                    "the per-edge explain payload (pattern, per-step bindings, substituted constraints, supporting)",
                    ":ret description half"
                ),
                other => panic!(":ret[1] must be the description String; got {other:?}"),
            }
        }
        other => panic!(":ret must be the pair [type, description]; got {other:?}"),
    }

    // Row 7 — existing readers' shapes unchanged.
    assert!(matches!(get(&map, ":arity"), Value::i64(_)), ":arity stays i64");
    assert!(matches!(get(&map, ":name"), Value::wat__core__keyword(_)), ":name stays keyword");
    match get(&map, ":purity") {
        Value::Enum(ev) => assert_eq!(ev.type_path, ":wat::runtime::Purity"),
        other => panic!(":purity must stay Enum; got {other:?}"),
    }
    match get(&map, ":determinism") {
        Value::Enum(ev) => assert_eq!(ev.type_path, ":wat::runtime::Determinism"),
        other => panic!(":determinism must stay Enum; got {other:?}"),
    }
    match get(&map, ":totality") {
        Value::Enum(ev) => assert_eq!(ev.type_path, ":wat::runtime::Totality"),
        other => panic!(":totality must stay Enum; got {other:?}"),
    }
}

/// Row 3 + 10 — the round trip closes, and a `#wat.doc/Row` renders from the lookup alone.
#[test]
fn from_metadata_of_the_lookup_equals_the_entry_doc() {
    let map = metadata_of(":user::step-payload-metadata");
    let ast = metadata_map_to_watast(&map);
    let got = wat_doc::from_metadata(&ast)
        .unwrap_or_else(|e| panic!("from_metadata(metadata-of step-payload) must succeed: {e:?}"));

    let src = include_str!("../../src/rete/step_payload.rs");
    let raw = rust_doc_before_attr(src, "wat_intrinsic(\":wat::rete::step-payload\")");
    let expected =
        wat_doc::parse(&raw).unwrap_or_else(|e| panic!("step-payload rust doc must parse: {e:?}"));

    assert_eq!(got.prose, expected.prose, ":doc");
    assert_eq!(got.added, expected.added, ":added");
    assert_eq!(got.ret_type, expected.ret_type, ":ret type");
    assert_eq!(got.ret, expected.ret, ":ret description");
    assert_eq!(got.args, expected.args, ":args");
    assert_eq!(got.see, expected.see, ":see");
    assert_eq!(got.deprecated, expected.deprecated, ":deprecated");
    assert_eq!(got.alias, expected.alias, ":alias");
    assert_eq!(got.purity, expected.purity);
    assert_eq!(got.determinism, expected.determinism);
    assert_eq!(got.totality, expected.totality);
    assert_eq!(got.expand_time, expected.expand_time);
    assert_eq!(got.category, expected.category);
    assert_eq!(got.examples, expected.examples, ":examples");
    assert!(
        got.yields.is_empty(),
        "named gap: :yields is not emitted; got {:?}",
        got.yields
    );

    let printed = wat_doc::print(&got);
    wat::assert_edn_matches_file!(
        printed,
        "probe_stone_metadata_of_whole_row__step_payload_row.edn",
        "lookup alone must render the step-payload #wat.doc/Row"
    );
}

/// Row 4 — both branches agree, key for key, on shape.
#[test]
fn both_branches_agree_key_for_key() {
    let registry = metadata_of(":user::step-payload-metadata");
    let wat = metadata_of(":user::sort-metadata");

    assert_eq!(defined_in_variant(&registry), "Rust", "step-payload is the registry branch");
    assert_eq!(defined_in_variant(&wat), "Wat", "sort is the wat-binding branch");

    for key in [
        ":args",
        ":examples",
        ":see",
        ":ret",
        ":doc",
        ":added",
        ":purity",
        ":determinism",
        ":totality",
        ":expand-time",
        ":category",
    ] {
        assert!(has_key(&registry, key), "registry missing {key}");
        assert!(has_key(&wat, key), "wat branch missing {key}");
    }

    assert_shape_vec_of_triples(get(&registry, ":args"), ":args");
    assert_shape_vec_of_triples(get(&wat, ":args"), ":args");
    assert_shape_examples(get(&registry, ":examples"));
    assert_shape_examples(get(&wat, ":examples"));

    // `:see` is the last Vec-shaped key checked by matches! rather than an exact compare —
    // its contents differ per row, only the SHAPE is the claim here. Not a loop: clippy's
    // `for loop over a single element`, and a one-element loop reads as if more are coming.
    assert!(
        matches!(get(&registry, ":see"), Value::Vec(_)),
        "registry :see must be Vec"
    );
    assert!(
        matches!(get(&wat, ":see"), Value::Vec(_)),
        "wat :see must be Vec"
    );
    for key in [":doc", ":added"] {
        assert!(
            matches!(get(&registry, key), Value::String(_)),
            "registry {key} must be String"
        );
        assert!(
            matches!(get(&wat, key), Value::String(_)),
            "wat {key} must be String"
        );
    }
    for key in [":purity", ":determinism", ":totality", ":expand-time", ":category"] {
        assert!(
            matches!(get(&registry, key), Value::Enum(_)),
            "registry {key} must be Enum"
        );
        assert!(
            matches!(get(&wat, key), Value::Enum(_)),
            "wat {key} must be Enum"
        );
    }

    match (get(&registry, ":ret"), get(&wat, ":ret")) {
        (Value::Vec(a), Value::Vec(b)) if a.len() == 2 && b.len() == 2 => {
            assert!(
                matches!(&a[1], Value::String(_)) && matches!(&b[1], Value::String(_)),
                ":ret[1] must be the description String on both branches"
            );
        }
        (r, w) => panic!(":ret must be a 2-vector on both branches; registry={r:?} wat={w:?}"),
    }

    assert!(!has_key(&registry, ":yields") && !has_key(&wat, ":yields"));
    assert!(!has_key(&registry, ":syntax") && !has_key(&wat, ":syntax"));

    // Wat-branch round-trip: from_metadata succeeds on a wat-defined binding too.
    let wat_doc = wat_doc::from_metadata(&metadata_map_to_watast(&wat))
        .unwrap_or_else(|e| panic!("from_metadata(metadata-of sort) must succeed: {e:?}"));
    assert_eq!(wat_doc.added, "1.0.0");
    assert!(!wat_doc.examples.is_empty());
}

/// Alias rows carry `:alias` as a single keyword (not a vector). Axes stay on the
/// map — metadata-of reports the target's axes AND the alias relationship.
/// `from_metadata` forbids that pairing; the round-trip test uses a non-alias FQDN.
#[test]
fn alias_row_carries_alias_keyword() {
    let map = metadata_of(":user::alias-metadata");
    match get(&map, ":alias") {
        Value::wat__core__keyword(k) => assert_eq!(k.as_str(), ":wat::i64::>"),
        other => panic!(":alias must be a single keyword FQDN; got {other:?}"),
    }
    assert!(
        matches!(get(&map, ":purity"), Value::Enum(_)),
        "alias rows still report the target's axes"
    );
}

/// Row 11 — committed negative control. Delete `:examples` → round-trip RED.
#[test]
fn deleting_examples_makes_from_metadata_refuse() {
    let mut map = metadata_of(":user::step-payload-metadata");
    map.retain(|k, _| match k {
        Value::wat__core__keyword(s) => s.as_str() != ":examples",
        _ => true,
    });
    let ast = metadata_map_to_watast(&map);
    match wat_doc::from_metadata(&ast) {
        Err(DocError::MissingExample) => {}
        other => panic!("expected MissingExample after deleting :examples; got {other:?}"),
    }
}

/// Same control for `:ret` reverting to a bare description — the decoder wants the pair.
#[test]
fn ret_as_a_bare_string_makes_from_metadata_refuse() {
    let mut map = metadata_of(":user::step-payload-metadata");
    let key = Value::wat__core__keyword(Arc::new(":ret".to_string()));
    map.insert(key, Value::String(Arc::new("a bare description".into())));
    let ast = metadata_map_to_watast(&map);
    match wat_doc::from_metadata(&ast) {
        Err(DocError::MalformedDirective { tag, .. }) if tag == ":ret" => {}
        other => panic!("expected MalformedDirective :ret after a bare string; got {other:?}"),
    }
}
