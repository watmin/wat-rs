//! VIGILIA experiri probe — is the `:wat::` vocabulary open?
//!
//! `src/resolve/walk.rs:268` accepts any `:wat::`-prefixed call head by PREFIX alone;
//! `src/check.rs:4884` and `:5558` each accept an unregistered scheme; `:5585` exempts
//! `:wat::` from the UnknownCallee heuristic. Each layer names the other as the checker.
//! This drives a freshly invented head — one that has never existed in this tree — through
//! `startup_from_file` in several positions, and reports which accept and which refuse.
//!
//! Run: cargo nextest run --release -p wat --test rete probe_vig_phantom_head

use std::sync::Arc;

use wat::freeze::startup_from_file;
use wat::runtime::Value;

/// (loaded?, applied-`:user::go`-result) for one fixture.
fn drive(path: &str) -> (Result<(), String>, Option<Result<Value, String>>) {
    match startup_from_file(path) {
        Err(e) => (Err(format!("{e:?}")), None),
        Ok(world) => {
            let Some(f) = world.symbols().get(":user::go") else {
                return (Ok(()), None);
            };
            let span = wat::span::Span::new(Arc::new("vigilia-probe".to_string()), 0, 0);
            let r = wat::runtime::apply_function(f.clone(), vec![], world.symbols(), span);
            (Ok(()), Some(r.map_err(|e| format!("{e:?}"))))
        }
    }
}

fn line(name: &str, path: &str) -> String {
    let (load, applied) = drive(path);
    match (load, applied) {
        (Err(e), _) => format!("{name}: LOAD REFUSED — {}", first_line(&e)),
        (Ok(()), None) => format!("{name}: LOADED, no :user::go"),
        (Ok(()), Some(Ok(v))) => format!("{name}: LOADED, applied -> Ok({v:?})"),
        (Ok(()), Some(Err(e))) => format!("{name}: LOADED, applied -> Err({})", first_line(&e)),
    }
}

fn first_line(s: &str) -> String {
    s.chars().take(400).collect()
}

#[test]
fn report_every_position() {
    let rows = [
        line("p0 real-head/forced          (CALIBRATION fire)", "tests/rete/probe_vig_phantom_p0_real.wat"),
        line("p5 :vph::phantom/forced      (CALIBRATION refuse)", "tests/rete/probe_vig_phantom_p5_unreserved.wat"),
        line("p1 :wat::core::phantom/unforced", "tests/rete/probe_vig_phantom_p1_unforced.wat"),
        line("p2 :wat::core::phantom/forced", "tests/rete/probe_vig_phantom_p2_forced.wat"),
        line("p6 :wat::kernel::phantom/unforced", "tests/rete/probe_vig_phantom_p6_kernel_unforced.wat"),
        line("p3 :wat::kernel::abort/TAKEN arm", "tests/rete/probe_vig_phantom_p3_kernel_forced.wat"),
        line("p4 :wat::kernel::abort/UNTAKEN arm", "tests/rete/probe_vig_phantom_p4_kernel_untaken.wat"),
    ];
    panic!("PROBE REPORT (deliberate):\n{}", rows.join("\n"));
}
