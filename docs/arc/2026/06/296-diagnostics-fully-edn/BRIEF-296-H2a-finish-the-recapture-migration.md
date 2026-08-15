# BRIEF — 296 H-2a: finish the recapture migration

Baseline: HEAD `a85b7605`, floor **4421 / 4421 / 263 skipped**, clippy 0.

## WHY — the door was built for this job, piloted, and never adopted

Arc 296 minted two golden macros. `assert_edn_eq!` was the **stone C pilot**
(`cd741bb1`, *"the swarm's reference"*). `assert_edn_matches_file!` is its **successor**
(`0c2b37ff`, *"296 recapture wall: assert_edn_matches_file! (UPDATE_EDN regen) — **proven on
wat_core_cond**"*) — the same assertion plus the ability to regenerate the golden from a real run.

Proven on one file. Adoption then stopped at **7 files, against 62 still on the pilot macro.**

Two things are stuck behind that unfinished migration:

1. **70 files carry `296-recapture-pending` `#[ignore]`s**, each naming *"unlock: 296 recapture (.edn
   data-equality flip)"*. The mechanism that unlocks them exists and reaches 7 files.
2. **Stone H** flips the variant wire form across 213 occurrences. 71 of the 76 affected `.edn`
   goldens cannot self-update, so H would mean hand-editing 71 goldens — precisely what
   capture-don't-guess exists to prevent.

This strike finishes the migration. Then H recaptures instead of hand-editing.

## THE CONVERSION — semantically identical, measured

Both macro bodies run the same assertion: `parse_owned` each side, `assert_eq!` on the resulting
`OwnedValue`. The **only** difference is where the expected side comes from — an `include_str!`
string versus a path the macro can also *write* to.

```rust
assert_edn_eq!(actual, include_str!("stem__case.edn"))
  →
assert_edn_matches_file!(actual, "stem__case.edn")
```

**62 call sites.** The path resolves as `<dir-of-file!()>/<name>`, so a co-located golden needs only
its bare filename.

This is observationally inert: same comparison, same data, same failures. **The floor must not move.**

## ⛔ THE GATE — a mechanism you never fire is a mechanism you do not have

Converting 62 sites and never exercising the regen path would leave us believing in a capability
proven on exactly one file — the same shape as the defect this strike is fixing.

So, after the conversion and a green floor:

1. Run the suite with **`UPDATE_EDN=1`**.
2. `git diff --stat` the `.edn` goldens.
3. **Any diff must be formatting-only.** The macro writes pretty-printed (`wat_edn::write_pretty`,
   2-space indent), so a golden that is data-equal but flat will be reformatted — that is the
   mechanism normalising, not a content change. **Prove it**: for every changed `.edn`, the parsed
   `OwnedValue` before and after must be equal. A diff that changes *data* is a finding — capture it,
   name the file, and report; do not accept it.
4. Then run the floor again **without** `UPDATE_EDN` and confirm green.

That sequence proves the regen path works on converted sites, not just on the pilot.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — a golden that is not co-located** with its `.rs` file. `assert_edn_matches_file!`
  resolves `<dir-of-file!()>/<name>` and cannot reach elsewhere. Report the pair; do not move the
  file to fit the macro, and do not convert that site.
- **STOP-2 — an `assert_edn_eq!` whose expected side is not an `include_str!` of a `.edn`** (an
  inline literal, a computed string, another test's golden). That site is not a golden comparison and
  is out of scope. Name it.
- **STOP-3 — the floor moves on conversion.** The two macros assert identically; a moved count means
  they do not, and that difference is the finding. Capture it before adjusting anything.
- **STOP-4 — `UPDATE_EDN=1` changes a golden's DATA.** Formatting may change; data may not. A data
  change means either a golden was stale-and-passing or the regen writes something different from
  what the assertion compares. Either is a real finding.

## OUT OF SCOPE, AFFIRMATIVELY

**Do not un-ignore the 70 `296-recapture-pending` tests.** This strike removes their blocker; it does
not adjudicate them. Each needs its own reading of what it measures before its `#[ignore]` comes off,
and several name a second unlock condition besides the recapture. Lifting them blind would be
inheriting 70 unexamined assertions in one motion. A later strike takes them with dispositions.

Also out of scope: the stone H tag/body flip itself. This is only the mechanism.

## BLAST RADIUS

The 62 `assert_edn_eq!` call sites and whatever `.edn` files `UPDATE_EDN=1` reformats. No `src/`
changes — both macros already exist and are correct. No `.wat` corpus changes.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.
Baseline **4421 / 0 / 263**, unchanged. Then the four-step gate above.

**On any red: do NOT re-run.** A re-run that goes green destroys the only evidence. Copy the failing
test's whole stdout+stderr block verbatim — never a `| head` window — name the exact assertion that
fired, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, and no notification is coming.
Run every build and test in the FOREGROUND and block on it; your turn ends when the numbers are in
your hands, not when a command is launched. Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first.
Leave the work uncommitted; the orchestrator weighs and commits.

Report: the converted count as the compiler saw it, every STOP with its `file:line`, the
`UPDATE_EDN=1` diff summary with your data-equality proof for each changed golden, both floor Summary
lines verbatim, and the honest deltas — especially anywhere this brief did not match the disk.
