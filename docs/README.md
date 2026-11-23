# Documentation

All documentation of the repository lives here.

| Path                                        | What                                                                 |
| ------------------------------------------- | -------------------------------------------------------------------- |
| [`goal.md`](./goal.md)                      | what the product is and what it must achieve                         |
| [`workflow.md`](./workflow.md)              | how work is done: branches, merges, subagents, cleanup               |
| [`rules.md`](./rules.md)                    | the rules every plan and change is checked against                   |
| [`quality.md`](./quality.md)                | code quality tooling: git hooks, linters, formatters, checks         |
| [`writing.md`](./writing.md)                | the voice and the shapes of every text the app shows                 |
| [`dependencies.md`](./dependencies.md)      | every dependency and tool, with its version and licence              |
| [`testing.md`](./testing.md)                | test bars and kinds of tests                                         |
| [`manual-testing.md`](./manual-testing.md)  | one checklist of every run by hand, in order, with commands          |
| [`manual-testing/`](./manual-testing/)      | the user's feedback from each session of runs by hand, by date       |
| [`architecture/`](./architecture/README.md) | crates, layers, data flow, the snapshot database                     |
| [`categories/`](./categories/README.md)     | the cleanup categories and their danger levels                       |
| [`restrictions/`](./restrictions/README.md) | hard constraints, with the reason and where each is enforced         |
| [`plans/`](./plans/README.md)               | what is being implemented now                                        |
| [`deferred/`](./deferred/README.md)         | agreed work postponed for later — check it when starting a new stage |
| [`known-issues/`](./known-issues/README.md) | defects we live with for now, with what was tried                    |
| [`ideas/`](./ideas/README.md)               | the user's ideas, not planned yet                                    |
| [`hacks/`](./hacks/README.md)               | non-obvious decisions and shortcuts in the code, and why             |
| [`postmortem/`](./postmortem/README.md)     | attempts that did not work, and why                                  |

## The stack

| Part      | Stack                                                                    |
| --------- | ------------------------------------------------------------------------ |
| Core      | Rust: engine, modules, scanner, rules, plain versioned files, cleaner    |
| Shell     | Tauri 2                                                                  |
| Interface | React + TypeScript                                                       |
| Quality   | Clippy pedantic, cargo-deny, ESLint strict, 90 % coverage, git hooks, CI |

Build commands: [`quality.md`](./quality.md) and [`testing.md`](./testing.md).
