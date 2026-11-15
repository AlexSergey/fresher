# Deferred

Agreed work postponed to a known stage. Unlike [`../ideas/`](../ideas/README.md), deferred work is decided: it waits
for its turn. Before starting a new stage (a plan, a release), check whether a row here is due and remind the user.

## Format

One file per item, `<topic>.md`: what is postponed, who agreed and when, the stage it is due at, and what doing it
needs. When it is done, its row and file are removed in the same change.

| Item                                    | Agreed     | Due at                   | What                                                                          |
| --------------------------------------- | ---------- | ------------------------ | ----------------------------------------------------------------------------- |
| [Licence notices](./licence-notices.md) | 2026-11-15 | before the first release | ship the licences of every Rust and npm dependency in the app (MPL-2.0 first) |
