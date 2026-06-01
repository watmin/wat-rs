# BRIEF — Stone 243.5 R2 — home-polish sweep (2 clippy L2s in the warded resident)

**Agent:** sonnet (`model:"sonnet"`). **Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. `git -C` for git; ignore `.claude/worktrees/`. Do NOT commit. Do NOT stamp vigilatum.

The 243.5 carve landed clean (probe passes, lib 895/0/1). Phase B vigilia (orchestrator's independent re-run) found 2 clippy L2s **in the warded home** `src/types/defstruct.rs`. The REMARKABLE bar is L1+L2=0 for lifted residents; these gate the ward. Fix exactly these two, nothing else.

## The two findings (verbatim from clippy)

1. **`src/types/defstruct.rs:53` — `doc_lazy_continuation`:**
   `/// Unknown keys are silently accepted (D5).` is a doc-list continuation line lacking indentation. Fix per clippy's help: indent it to continue the list item, OR add a blank line to make it its own paragraph — whichever reads correctly given the surrounding doc structure (read the full doc comment to decide).

2. **`src/types/defstruct.rs:56` — `type_complexity`:**
   `) -> Result<(Vec<String>, HashMap<String, Vec<String>>), TypeError>` is flagged complex. Introduce a `type` alias with an HONEST, descriptive name for the `(Vec<String>, HashMap<String, Vec<String>>)` pair — name it for what it IS in this domain (it's the parsed struct metadata: field-name list + the restrictions map). Define the alias in `defstruct.rs` near the fn; use it in the return type. Do NOT change behavior or the tuple's contents — this is a naming/readability fix only.

## Constraints

- Touch ONLY `src/types/defstruct.rs`. Nothing else.
- After: `cargo clippy -p wat --release 2>&1 | grep "src/types/defstruct.rs"` returns ZERO. `cargo build -p wat` clean. `cargo test -p wat --test probe_arc243_stone5_register_subtype_span` still passes.
- Do NOT touch the `list_span` warnings anywhere — those are pre-existing RuntimeError debt in OTHER files, explicitly out of scope (banked separately).
- No runes, no "deferred"/"TODO"/"future" text.

## Return

The two fixes (before/after for each), confirmation that clippy on `src/types/defstruct.rs` is now zero, and the probe still green. Do NOT commit.
