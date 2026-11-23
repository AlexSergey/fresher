# Contributing

Fresher is a macOS disk cleaner that removes real files, so safety and tests come first.

## Set up

```bash
npm install   # also installs the git hooks
npm run dev
npm test
```

## Read first

- [`goal.md`](../docs/goal.md) and [`rules.md`](../docs/rules.md)
- [`workflow.md`](../docs/workflow.md)
- [`architecture/`](../docs/architecture/README.md)
- [`restrictions/`](../docs/restrictions/README.md)

## A new kind of folder

Add a category file in [`docs/categories/`](../docs/categories/README.md) (with its `**Example:**` line), write the
negative tests first, and add a sandbox case file ([`testing.md`](../docs/testing.md)).

## Commits

Conventional Commits. The git hooks are never bypassed ([`quality.md`](../docs/quality.md)).

## Licence and origin

Write your own solution: code copied from another project is not accepted, whatever its licence.

A contribution is accepted under MIT OR Apache-2.0, the project's licences.
