# BRIEF — Ward-integrity clippy R2 — the 5 comms/ judgment-call allows (excusare-blessed)

**Agent:** sonnet (`model:"sonnet"`). **Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. `git -C` for git; ignore `.claude/worktrees/`. Do NOT commit. Do NOT touch any `//! vigilatum:` line (orchestrator re-stamps).

The mechanical ward-integrity fixes already landed (commit `04d5d1e5`). The ONLY remaining warded-home clippy findings are 5 sites in `src/comms/`, all of them the two judgment-call lints that the `excusare` spell already weighed and ruled **HOLDS-as-perennial** (the lint is genuinely wrong for the comms domain — a documented `#[allow]` is the correct exemption, not a code change). Apply exactly these 5 allows with the reasons given. Touch ONLY `src/comms/{mod,thread,process}.rs`.

## The 5 sites + exact attributes

Each gets a `// rune:excusare(perennial) — <reason>` comment on the line ABOVE the `#[allow(...)]`, then the `#[allow]` on the method/trait. (The `rune:excusare(perennial)` is excusare's self-rune form — a structurally-immutable exemption. excusare R-loop verified these warrants are perennial: the guarded property cannot change without an architectural change that trips the comms ward first.)

### A — `len_without_is_empty` (3 sites: trait + 2 impls)

The comms 9-spell cast deliberately narrowed `len()`'s contract — the process tier's `len()` is a kernel-invisible approximation (counts only locally-drained frames; kernel-pipe bytes not yet read are invisible). A naive `is_empty()` (`self.len()==0`) would return `true` while the kernel pipe holds unread data — misleading. The exemption is correct-forever: the transport-oblivion model makes this asymmetry permanent.

1. **`src/comms/mod.rs:587`** — the `CommReceiver` trait (the `len` method declaration). Reason:
   `// rune:excusare(perennial) — is_empty() structurally withheld: the process tier's len() is a kernel-invisible approximation (kernel-pipe bytes not-yet-drained are invisible); self.len()==0 returns true while unread frames sit in the pipe, so a naive is_empty() would mislead. The transport-oblivion model makes this asymmetry permanent; any change to the process pipe transport would trip the comms ward first. (Documented narrowed-len contract; 9-spell cast.)`
2. **`src/comms/thread.rs:163`** — `thread::Receiver::len`. Same `rune:excusare(perennial)` reason, abbreviated to: `is_empty() withheld at the trait level for the kernel-invisible process-tier len() approximation (see CommReceiver); the thread tier's len() is exact but the trait contract is unified — adding is_empty() to one tier and not the other breaks the unified surface. Perennial per the transport model.`
3. **`src/comms/process.rs:473`** — `process::Receiver::len`. Same reason as the trait (this is the tier where the approximation literally lives).

### B — `new_without_default` / Default-for-Select (2 sites)

The struere finding established that an empty `Select` is a footgun — `select()` panics (thread) / errors (process) with zero registered arms. A `Default` impl would manufacture exactly that prohibited empty value with no call-site signal that arm-registration is required. The exemption is perennial: the empty-Select guard is a deliberate architectural constraint.

4. **`src/comms/thread.rs:236`** — `thread::Select::new`. Reason:
   `// rune:excusare(perennial) — Default withheld by design: an empty Select panics at select() time (no-arm footgun the comms vigilia eliminated). A Default impl would produce the exact prohibited empty value with no call-site signal that arm registration is required. Any relaxation would require removing the empty-Select guard, which would trip the comms ward (struere empty-Select finding) first.`
5. **`src/comms/process.rs:819`** — `process::Select::new`. Same reason, with "panics" → "errors" (process tier returns `Err` not panic): `Default withheld by design: an empty Select errors at select() time (no-arm footgun). A Default impl would produce the prohibited empty value with no call-site signal. Removing this guard would trip the comms ward first.`

## After

- `cargo clippy -p wat --release 2>&1 | grep -E "src/comms/"` → returns ONLY `result_large_err` lines (out of scope), or empty. ALL 5 judgment-call lints gone (each now has a documented `#[allow]`).
- `cargo build -p wat` clean; `cargo test -p wat` green except the banked `probe_8_atom_round_trip`.
- Touch ONLY the 3 comms files. Do NOT touch the `function/`, `remedy/`, `rust_deps/` homes (already clippy-clean from `04d5d1e5`). Do NOT touch `result_large_err` anywhere (banked separately, #167).
- Do NOT re-stamp `//! vigilatum:` — orchestrator does that after independently confirming clippy-zero.

## Return

The 5 allows applied (file:line + the exact rune+allow you placed), the `cargo clippy | grep src/comms/` result (proving only result_large_err remains), the test tally. Do NOT commit.
