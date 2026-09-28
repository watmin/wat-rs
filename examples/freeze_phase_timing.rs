//! Excursus 003 D5 — measure where a wat world freeze spends its time, and which phase
//! grows with the number of declared stdlib-shaped records.
//!
//! Companion instrument for
//! `docs/excursus/2026/09/003-the-little-wat-findings/MEASURE-where-a-freeze-spends-its-time.md`
//! (`AUDIT-the-shape-of-an-error.md`'s Strike B worklist, the S2 "cost" row). Read that
//! report for the numbers this produced on one box; run this yourself to re-derive them, or
//! to check a later commit against them.
//!
//! ## What it does
//!
//! Sets `WAT_FREEZE_PHASE_TIMING` itself (no need to export it) so the freeze pipeline's
//! existing `pass_order::record` announce points (`src/freeze/pass_order.rs`) also stamp an
//! `Instant`, then:
//!
//! 1. **Phase breakdown** — freezes a hello-world program `SAMPLES` times (default 8, one
//!    warm-up run discarded first) and prints each sample's total wall time, its per-phase
//!    breakdown (`wat::freeze::take_freeze_phase_timings`), and the residual (total minus
//!    the sum of phases — see that function's doc for exactly what the residual covers:
//!    the entry parse plus `check_program` plus `FrozenWorld::freeze`'s own body, none of
//!    which has an individual announce point).
//! 2. **Per-record attribution** — repeats the same measurement with `N_RECORDS` (default
//!    40) synthetic `defrecord`s of stdlib-`check-errors.wat` shape (`message`/`location`/
//!    `causes` plus three plain fields) appended to the SAME entry source, so they are
//!    user-declared forms flowing through the identical macro-expand → register-types →
//!    register-defines → resolve pipeline a stdlib record does (see the report's
//!    methodology note for the one place this proxy and a true stdlib record diverge:
//!    which of the paired stdlib/user `pass_order` steps — e.g. `5-register-stdlib-types`
//!    vs `5-register-types` — absorbs the cost).
//!
//! Every sample prints as one EDN map per line to stdout. This is a diagnostic example
//! binary, never invoked by a normal `wat` run and never linked into `src/bin/wat.rs` — its
//! stdout is not the wire the harness rules protect; it is still EDN so the numbers are
//! machine-readable without a bespoke parser.
//!
//! ## Run
//!
//! ```text
//! cargo run --release --example freeze_phase_timing -- [samples] [n_records]
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};

use wat::freeze::{startup_from_source, take_freeze_phase_timings};
use wat::load::loader::InMemoryLoader;

const HELLO_WORLD: &str =
    "(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println 1))";

/// One synthetic record, shaped like a typical `wat/check-errors.wat` variant: the
/// floor (`message`/`location`/`causes`) plus a few plain fields — six fields total,
/// close to the ~5-8 field average the stdlib's declared-error records carry.
fn synthetic_record(i: usize) -> String {
    format!(
        "(:wat::core::defrecord :user::SyntheticRecord{i}\n  \
         [message <- :wat::core::String\n   \
          location <- :wat::core::Span\n   \
          causes <- (:wat::core::Vector :- [:wat::core::Error])\n   \
          field-a <- :wat::core::String\n   \
          field-b <- :wat::core::i64\n   \
          field-c <- :wat::core::bool])\n"
    )
}

fn synthetic_records(n: usize) -> String {
    (0..n).map(synthetic_record).collect()
}

struct Sample {
    total: Duration,
    phases: Vec<(&'static str, Duration)>,
}

fn run_once(src: &str) -> Sample {
    let t0 = Instant::now();
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    let total = t0.elapsed();
    // A probe that fails to freeze invalidates every downstream number — panic loudly
    // rather than silently skip a sample.
    world.unwrap_or_else(|e| panic!("probe program must freeze cleanly: {e:?}"));
    let phases = take_freeze_phase_timings();
    Sample { total, phases }
}

fn edn_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn print_sample(label: &str, iter: usize, sample: &Sample) {
    let phase_sum: Duration = sample.phases.iter().map(|(_, d)| *d).sum();
    let residual = sample.total.saturating_sub(phase_sum);
    print!(
        "{{:label \"{}\" :iter {} :total-us {} :residual-us {} :phases {{",
        edn_escape(label),
        iter,
        sample.total.as_micros(),
        residual.as_micros(),
    );
    for (name, d) in &sample.phases {
        print!(" :{} {}", name, d.as_micros());
    }
    println!(" }}}}");
}

fn main() {
    // Own the flag — a reader of this file need not remember to export anything.
    std::env::set_var("WAT_FREEZE_PHASE_TIMING", "1");

    let mut args = std::env::args().skip(1);
    let samples: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(8);
    let n_records: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(40);

    println!(
        "{{:label \"meta\" :samples {samples} :n-records {n_records} :binary \"freeze_phase_timing\"}}"
    );

    // One untimed warm-up per arm — page cache / allocator warmup, discarded.
    let _ = run_once(HELLO_WORLD);
    for i in 0..samples {
        print_sample("baseline", i, &run_once(HELLO_WORLD));
    }

    let treatment_src = format!("{HELLO_WORLD}\n{}", synthetic_records(n_records));
    let _ = run_once(&treatment_src);
    for i in 0..samples {
        print_sample("with-synthetic-records", i, &run_once(&treatment_src));
    }
}
