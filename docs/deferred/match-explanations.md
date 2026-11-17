# Why an item matched

**Agreed:** 2026-11-17, by the user (plan 011, the light form) · **Due at:** with the first rule-based feature module
that needs more than its reason text

## What is postponed

"Why this?" on a finding: the rule, the conditions that held, the claim it won over another module's; and a rule's
own "left out because" (a cache younger than its `min-age-days`, a name its `must-not-match` refused), reported through
the module's `left_out` of plan 011. Today a finding carries one reason text, and plan 011 gave modules `left_out` for
items they recognize and do not offer.

## Why

The finding's reason text answers the first question; the full explanation is one mechanism for the UI, the
developer tools and the tests, and is best designed with a real module's rules in front of it.

## What doing it needs

1. A typed explanation per finding (rule id, conditions, the claim), kept in memory, never on disk.
2. A rule format field for a left-out reason, declared like the module's problems (plan 011).
3. A "Why this?" disclosure on the finding row.
