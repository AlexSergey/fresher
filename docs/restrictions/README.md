# Restrictions

Hard constraints of this project. Each entry gives the rule, the reason and where it is enforced. Read these before
writing a plan.

- [deletion-safety.md](deletion-safety.md) — what may be deleted, when, how, and what never is.
- [privacy.md](privacy.md) — no network, no telemetry, local data only.
- [dependencies.md](dependencies.md) — permissive licences only, exact versions, locked builds.
- [repository.md](repository.md) — English only, commit messages, generated and protected files.

## Format

```markdown
## <Short name>

- **Rule:** what is required or forbidden.
- **Why:** the reason.
- **Enforced by:** the code, lint, test or hook that holds it (or "review" when nothing automatic does yet).
```
