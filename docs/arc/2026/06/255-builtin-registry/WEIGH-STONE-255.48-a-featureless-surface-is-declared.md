# WEIGH — STONE 255.48: a featureless surface's members are the declared ones — ACCEPTED

**Executor: grok via pulsare, commit `352eada3d`.** Weighed by the orchestrator on 2026-09-26. Ruling B1.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor | `.floor/2026-09-26T07-24-22Z`: **6145 passed / 22 skipped** (6141 + the 4 new rows) |
| clippy `--release --all-targets -D warnings` | rc 0 |
| hello world `wat --check` | rc 0 |
| the undeclared case (`:hello::bad` restored, in the session scratchpad) | rc 1, *":hello::takes-orderable: parameter #1 expects :hello::Orderable; got :hello::Opaque"* |
| delta | NEW 2 / RECOVERY 0; `new.txt` is byte-identical to the previous run (`probe-c1-clean-surface.wat`, `Ngram.wat`) |
| census | grok's `no STOP-8`, 215 non-zero. Not re-run; the change adds no refusal outside the floor's files |

The diff is read: two guards in `assignable` (`src/check.rs` ~17674 aggregate branch, ~17710 foreign branch),
both keyed on `surf_clone.members.is_empty()`. A declared edge still wins earlier, at the path-to-path
`is_subtype` arm. `struct_satisfies_surface` stays vacuous on an empty list, and both of its callers now refuse
that list first. `program::Env`'s `user-data` is typed as the `:wat::core::Record` root, not as a surface
(`wat/program.wat:34`), so the live stdlib path is untouched.

## The STOP-2 hit is the orchestrator's error

Grok quoted `probe_arc293_holder_root_symbol.rs`'s contract sentence verbatim (*"A 0-member `:nature` surface is
'any aggregate of that nature'"*) but did not stop. **The brief caused this:** item 2 required the
`:env::Rec` → `:env::Portable` edge in exactly the file that STOP-2 was about. That is a self-contradicting
brief, the second after 255.40. The assertion was not rewritten; the declared edge is now what satisfies it,
and the comments no longer claim the retired sentence. **Accepted on that basis.** The builder should see the
quoted contract (above and in the SCORE) as the one place B1 retired a stated design.

**Cure for the next brief:** before sending, check every STOP trigger against the brief's own work list. A
trigger that fires on a site the list names is a contradiction.

## Found by the strike

- `.wat.bad` fixtures are outside the census's file list. `probe_arc278_open_surface_dispatch_ambiguous.wat.bad`
  needed two edges that 255.47's census could not see.
- Historical design notes (`278-rules-engine/REALIZATIONS.md`, `DESIGN-telemetry-service-and-query-surface.md`)
  still describe the open `Reason`. They are records, not rewritten.
