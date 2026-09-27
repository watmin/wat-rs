//! Stone 255.54 — Orderable and Equatable agree with the predicates on the
//! types the corpus actually compares.
// rune:lint(no-inlined-wat) — the allow-list names are the checker's rendered types, compared exactly.

use std::collections::BTreeSet;
use std::process::Command;

use wat::check::error::CheckErrorKind;
use wat::check::ClassVerdict;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;
use wat::types::{Nature, Purity, TypeDef};

#[test]
fn a_pure_record_is_equatable_and_not_orderable() {
    match call_beside_value(file!(), ":user::eq") {
        Ok(Value::i64(1)) => {}
        other => panic!("a record joins Equatable through :wat::core::Record; got {other:?}"),
    }
    let result = startup_from_file("tests/types/probe_arc255_54_classes_ord.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, got, .. }
            if callee == ":wat::core::<" && got == ":u::Pt"
    );
}

#[test]
fn a_struct_is_not_equatable() {
    let result = startup_from_file("tests/types/probe_arc255_54_classes_struct.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":u::take-eq"
            && param == "#1"
            && expected == ":wat::core::Equatable"
            && got == ":u::St"
    );
}

#[test]
fn bigint_is_orderable() {
    match call_beside_value(file!(), ":user::ord-bigint") {
        Ok(Value::i64(1)) => {}
        other => panic!("bigint is Orderable; got {other:?}"),
    }
}

#[test]
fn a_newtype_of_i64_may_join_orderable() {
    match call_beside_value(file!(), ":user::ord-nt") {
        Ok(Value::i64(1)) => {}
        other => panic!("a newtype of i64 may declare Orderable; got {other:?}"),
    }
}

#[test]
fn a_newtype_of_a_struct_may_not_join_orderable() {
    let result = startup_from_file("tests/types/probe_arc255_54_classes_nt_struct.wat.bad");
    wat::assert_startup_error!(result,
        wat::freeze::StartupError::Type(err)
            if matches!(err.kind(), wat::types::TypeErrorKind::MalformedDecl { head, reason }
                if head == "extend-type"
                && reason == "newtype :u::Box joins Orderable only when its inner type is Orderable; :u::St is not")
    );
}

/// Every unique operand type the corpus passed to `<`/`>`/`<=`/`>=`/`=`/`not=`.
/// The list is the fixture (`probe_arc255_54_class_types.txt`), not a fresh census.
#[test]
fn declared_classes_agree_with_the_predicates() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let world = startup_from_file("tests/types/probe_arc255_54_class_world.wat")
        .expect("the agreement world declares the corpus's user types");
    let types = world.types();
    let fixture = std::fs::read_to_string(root.join("tests/types/probe_arc255_54_class_types.txt"))
        .expect("committed operand-type fixture");
    let mut rows = 0usize;
    let mut ruled = 0usize;
    let mut findings = 0usize;
    let mut problems = Vec::new();
    for line in fixture.lines() {
        if line.is_empty() {
            continue;
        }
        let (kind, rendered) = line.split_once('\t').unwrap_or_else(|| {
            panic!("fixture row has no tab: {line}");
        });
        rows += 1;
        let verdict = wat::check::class_verdict(rendered, types).unwrap_or_else(|err| {
            panic!("could not re-read {kind} operand {rendered}: {err}");
        });
        match classify(kind, rendered, &verdict, types) {
            Row::Agree => {}
            Row::Ruled => ruled += 1,
            Row::Finding => findings += 1,
            Row::Problem(msg) => problems.push(msg),
        }
    }
    assert_eq!(rows, 71, "the fixture is the collected unique operand types");
    assert_eq!(ruled, 2, "bigint and rational are the ruled Orderable corrections");
    // :T, :wat::core::Value, and :wat::core::nil. A new finding fails above as a problem.
    assert_eq!(findings, 3, "the three named findings");
    assert!(
        problems.is_empty(),
        "agreement disagreements:\n{}",
        problems.join("\n")
    );
}

enum Row {
    Agree,
    Ruled,
    Finding,
    Problem(String),
}

fn classify(
    kind: &str,
    rendered: &str,
    verdict: &ClassVerdict,
    types: &wat::types::TypeEnv,
) -> Row {
    let (declared, predicate) = match kind {
        "ord" => (verdict.declared_orderable, verdict.predicate_orderable),
        "eq" => (verdict.declared_equatable, verdict.predicate_equatable),
        other => return Row::Problem(format!("unknown kind {other}")),
    };
    if declared == predicate {
        return Row::Agree;
    }
    // Ruled correction: the runtime orders these; the predicate's leaf arm does not.
    if kind == "ord" && (rendered == ":wat::core::bigint" || rendered == ":wat::core::rational") {
        return if declared && !predicate {
            Row::Ruled
        } else {
            Row::Problem(format!(
                "ruled Orderable correction has the wrong direction: {rendered} declared={declared} predicate={predicate}"
            ))
        };
    }
    // Q1 — a struct is an aggregate, so the predicate says equatable. The class does not.
    if kind == "eq" && !declared && predicate {
        if let Some(TypeDef::Aggregate(agg)) = types.get(rendered) {
            if agg.nature == Nature::Struct {
                return Row::Ruled;
            }
        }
        // EN-P — an Impure enum (and a variant, which is its own Enum singleton) is
        // equatable to the predicate and not a member of the class.
        if let Some(TypeDef::Enum(e)) = types.get(rendered) {
            if e.purity == Purity::Impure {
                return Row::Ruled;
            }
        }
        // N-R — a newtype joins Equatable only when its inner type is pure.
        // The predicate recurses into the inner type regardless.
        if let Some(TypeDef::Newtype(_)) = types.get(rendered) {
            return Row::Ruled;
        }
    }
    // Section 4, reported and not declared away. A rigid `:T` is `Path(":T")`.
    // The predicate defers (`is_type_param_letter`). The class has no edge.
    if kind == "eq" && rendered == ":T" && !declared && predicate {
        return Row::Finding;
    }
    // Same deferral, for the protocol holder `:wat::core::Value`. Value is the
    // universal supertype, so it is not a member of Equatable.
    if kind == "eq" && rendered == ":wat::core::Value" && !declared && predicate {
        return Row::Finding;
    }
    // `:wat::core::nil` is an alias of the empty tuple. The `:..` edge admits
    // every slot, and there are none, so the class says Orderable. The predicate
    // does not. 255.53 left this unpinned; this stone does not add an arity check.
    if kind == "ord" && rendered == ":wat::core::nil" && declared && !predicate {
        return Row::Finding;
    }
    Row::Problem(format!(
        "{kind}\t{rendered}\tdeclared={declared}\tpredicate={predicate}"
    ))
}

/// One-shot collector. Not part of the floor. Run with `--ignored` to refresh
/// `probe_arc255_54_class_hits.txt`.
#[test]
#[ignore]
fn collect_compared_types() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let listed = Command::new("git")
        .args(["ls-files", "*.wat", "*.wat.bad"])
        .current_dir(root)
        .output()
        .expect("git ls-files");
    let files = String::from_utf8(listed.stdout).unwrap();
    let rels: Vec<String> = files
        .lines()
        .filter(|rel| !rel.is_empty() && !rel.contains("deep_freeze_recursion"))
        .map(|s| s.to_string())
        .collect();
    let mut all = BTreeSet::new();
    let mut pending = Vec::new();
    for rel in rels {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                wat::check::class_capture_begin();
                let _ = startup_from_file(&rel);
            }));
            let _ = tx.send(wat::check::class_capture_end());
        });
        pending.push(rx);
        if pending.len() == 8 {
            if let Ok(hits) = pending.remove(0).recv_timeout(std::time::Duration::from_secs(3)) {
                for hit in hits {
                    all.insert(hit);
                }
            }
        }
    }
    for rx in pending {
        if let Ok(hits) = rx.recv_timeout(std::time::Duration::from_secs(3)) {
            for hit in hits {
                all.insert(hit);
            }
        }
    }
    let path = root.join("tests/types/probe_arc255_54_class_hits.txt");
    std::fs::write(&path, all.into_iter().collect::<Vec<_>>().join("\n") + "\n").unwrap();
}
