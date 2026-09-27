//! Stone 255.54 — membership in Orderable and Equatable, proved by a parameter.
//! Stone 255.56 deleted the predicate agreement test. The predicates are gone.

use std::collections::BTreeSet;
use std::process::Command;

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

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
