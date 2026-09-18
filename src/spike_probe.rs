//! SPIKE PROBE — `can-a-user-def-change-a-stdlib-verdict` (excursus 2026/08/001-sns-sqs).
//!
//! ⚠ NOT a feature. A measurement instrument for one spike: it records every NAME that
//! the `check:body-infer(ALL fns)` sweep probes against user-reachable check state, so the
//! question "which state does body-infer consult that user code can reach?" is answered by a
//! WITNESS rather than by an argument (the shape Tier A's `MacroRegistry::contains` gate used).
//!
//! Off unless `WAT_SPIKE_WITNESS` is set. Zero cost otherwise: one relaxed atomic load.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// True only while `check_program`'s `P_CHECK_BODIES` loop (stdlib + user fn bodies) runs.
static IN_BODY_SWEEP: AtomicBool = AtomicBool::new(false);
/// The function whose body is currently being inferred, plus the file its body came from.
static CURRENT_FN: Mutex<Option<(String, String)>> = Mutex::new(None);
static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// door → set of names probed at that door during the sweep.
static WITNESS: Mutex<Option<BTreeSet<(&'static str, String)>>> = Mutex::new(None);
/// (body-file, fn-path, door, probed-name) — the full, unfiltered record.
static TRACE: Mutex<Option<BTreeSet<(String, String, &'static str, String)>>> = Mutex::new(None);

pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var_os("WAT_SPIKE_WITNESS").is_some())
}

pub fn begin_body_sweep(fn_path: &str, body_file: &str) {
    if enabled() {
        *CURRENT_FN.lock().expect("spike current fn") =
            Some((fn_path.to_string(), body_file.to_string()));
        IN_BODY_SWEEP.store(true, Ordering::Relaxed);
    }
}

pub fn end_body_sweep() {
    if enabled() {
        IN_BODY_SWEEP.store(false, Ordering::Relaxed);
    }
}

/// Record a name probed at `door`. Cheap no-op outside the sweep / when disabled.
pub fn probe(door: &'static str, name: &str) {
    if !enabled() || !IN_BODY_SWEEP.load(Ordering::Relaxed) {
        return;
    }
    let (fn_path, body_file) = CURRENT_FN
        .lock()
        .expect("spike current fn")
        .clone()
        .unwrap_or_else(|| ("<none>".into(), "<none>".into()));
    TRACE.lock().expect("spike trace").get_or_insert_with(BTreeSet::new).insert((
        body_file,
        fn_path,
        door,
        name.to_string(),
    ));
    let mut g = WITNESS.lock().expect("spike witness mutex");
    g.get_or_insert_with(BTreeSet::new)
        .insert((door, name.to_string()));
}

/// A name a user program could actually declare: NOT under a reserved prefix.
/// The reserved set is the one `check.rs`'s `ReservedPrefix` gate enforces.
pub fn user_reachable(name: &str) -> bool {
    !(name.starts_with(":wat::")
        || name.starts_with(":rust::")
        || name.starts_with(":$bound::"))
}

/// Print the witness, partitioned into user-reachable names and the rest.
pub fn report() {
    if !enabled() {
        return;
    }
    let g = WITNESS.lock().expect("spike witness mutex");
    let Some(set) = g.as_ref() else {
        eprintln!("[spike-witness] EMPTY — the body sweep probed nothing (did it run?)");
        return;
    };
    let mut doors: BTreeSet<&'static str> = BTreeSet::new();
    for (d, _) in set.iter() {
        doors.insert(d);
    }
    eprintln!("[spike-witness] {} (door, name) pairs across {} doors", set.len(), doors.len());
    if std::env::var_os("WAT_SPIKE_WITNESS_ALL").is_some() {
        if let Some(tr) = TRACE.lock().expect("spike trace").as_ref() {
            for (file, f, d, n) in tr.iter() {
                eprintln!("[spike-trace] {}\t{}\t{}\t{}", file, f, d, n);
            }
        }
    }
    for d in doors {
        let all: Vec<&String> = set.iter().filter(|(dd, _)| *dd == d).map(|(_, n)| n).collect();
        let reach: Vec<&&String> = all.iter().filter(|n| user_reachable(n)).collect();
        eprintln!(
            "[spike-witness] door {:<26} probed={:<6} user-reachable={}",
            d,
            all.len(),
            reach.len()
        );
        for n in reach {
            eprintln!("[spike-witness]     ⭑ {}", n);
        }

    }
}
