# Contributing

Fresher is a macOS disk cleaner that removes real files, so safety and tests come first.

## Set up

```bash
npm install   # also installs the git hooks
npm run dev
npm test
```

## Where to start

Issues labelled [`good first issue`](https://github.com/AlexSergey/fresher/labels/good%20first%20issue) are small and
self-contained; most add a developer tool's cache (`new category`), built like the existing ones. Issues labelled
[`help wanted`](https://github.com/AlexSergey/fresher/labels/help%20wanted) are known defects that need someone to find
the cause. Say on an issue that you are taking it before you start, so two people do not build the same thing.

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
