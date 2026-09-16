//! `ring-ceiling` — how many io_uring instances will THIS box hand out, and what does
//! commit accounting say at the moment it refuses?
//!
//! ⛔ WHY THIS EXISTS. Excursus 001 `a-deadline-does-not-cost-a-ring` spent three days on a
//! CI-only red — `IoUring::new(4) … Cannot allocate memory (os error 12)` — at a feedback
//! cost of ~15 minutes per attempt, because the only way to ask the question was to run a
//! 5259-test floor and hope the chaos gate tripped. Builder: *"there's gotta be a faster
//! feedback loop here... its like ~15min per run"*. He is right: the decisive experiment is
//! a counter, and it takes seconds.
//!
//! Four hypotheses died before this existed (RLIMIT_MEMLOCK, memory exhaustion,
//! `vm.max_map_count`, then memlock again on arithmetic). Each was reasoned from the errno
//! and each died on a measurement. This binary is the measurement, made cheap enough to run
//! anywhere, on demand.
//!
//! ⚠ It holds every ring it creates — the ceiling is about CONCURRENT instances, so they must
//! not be dropped as it goes. Bounded at `CAP` so it cannot destabilise the host it is
//! diagnosing; a run that reaches `CAP` reports "no ceiling found", which is a real answer.
//!
//!   cargo run --release --bin ring-ceiling            # default cap
//!   cargo run --release --bin ring-ceiling -- 20000   # explicit cap

use io_uring::IoUring;

const CAP: usize = 5000;

fn proc_field(path: &str, key: &str) -> String {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with(key))
                .map(|l| l.split_whitespace().nth(1).unwrap_or("?").to_string())
        })
        .unwrap_or_else(|| "?".into())
}

fn sysctl(path: &str) -> String {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "?".into())
}

fn census(tag: &str) {
    println!(
        "{tag}: overcommit_memory={} overcommit_ratio={} CommitLimit={}kB Committed_AS={}kB \
         MemAvailable={}kB | this process VmSize={}kB VmPeak={}kB threads={}",
        sysctl("/proc/sys/vm/overcommit_memory"),
        sysctl("/proc/sys/vm/overcommit_ratio"),
        proc_field("/proc/meminfo", "CommitLimit:"),
        proc_field("/proc/meminfo", "Committed_AS:"),
        proc_field("/proc/meminfo", "MemAvailable:"),
        proc_field("/proc/self/status", "VmSize:"),
        proc_field("/proc/self/status", "VmPeak:"),
        proc_field("/proc/self/status", "Threads:"),
    );
}

fn main() {
    let cap: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(CAP);

    println!("kernel={} nproc={}", sysctl("/proc/sys/kernel/osrelease"), num_cpus());
    println!(
        "memlock={} max_map_count={} io_uring_disabled={}",
        rlimit_memlock(),
        sysctl("/proc/sys/vm/max_map_count"),
        sysctl("/proc/sys/kernel/io_uring_disabled"),
    );
    census("before");

    // Hold them all: the question is how many can exist AT ONCE.
    let mut held: Vec<IoUring> = Vec::with_capacity(cap.min(4096));
    for n in 0..cap {
        match IoUring::new(4) {
            Ok(r) => held.push(r),
            Err(e) => {
                println!("CEILING: refused at ring #{} — {e} (raw os error {:?})", n + 1, e.raw_os_error());
                census("at-refusal");
                println!("held={} rings when it refused", held.len());
                return;
            }
        }
        if n + 1 == 16 || n + 1 == 256 || (n + 1) % 1000 == 0 {
            census(&format!("held={}", n + 1));
        }
    }
    println!("NO CEILING FOUND under cap={cap} — {} rings held simultaneously", held.len());
    census("after");
}

fn num_cpus() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0)
}

fn rlimit_memlock() -> String {
    std::fs::read_to_string("/proc/self/limits")
        .ok()
        .and_then(|s| s.lines().find(|l| l.contains("Max locked memory")).map(|l| l.to_string()))
        .unwrap_or_else(|| "?".into())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
