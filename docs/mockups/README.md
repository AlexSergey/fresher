# Mockups

The approved mockups of the app's screens, one folder per release they are drawn for: `docs/mockups/<version>/`. A
plan that changes a screen links the mockup it builds from; a mockup changes only with the user's approval, and the
plan that approved it is named below.

The files are Design Component pages (`*.dc.html`): plain HTML with inline styles, drawn on a design canvas. They load
the canvas's `support.js`, so opened from the repository they show their markup but not every style; read them as the
reference for layout, colours, texts and sizes. The data in them is made up: names, paths and sizes are examples.

## 0.1.0

Approved on 2026-11-24 for [plan 042](../plans/042-plain-interface/readme.md) (the third round; the feedback that
shaped it: [`manual-testing/2026-11-24-feedback.md`](../manual-testing/2026-11-24-feedback.md)).

| File                                                               | Screen                                                                 |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------- |
| [`scan.dc.html`](./0.1.0/scan.dc.html)                             | Scan: the disk card with its ring, the separator, the tiles by group   |
| [`review.dc.html`](./0.1.0/review.dc.html)                         | Review: the summary card, one tile per category with its items         |
| [`category-details.dc.html`](./0.1.0/category-details.dc.html)     | A category's details: three answers, the left-out line, the items      |
| [`settings.dc.html`](./0.1.0/settings.dc.html)                     | Settings: its own sidebar and tree, a module's page, the slider        |
| [`settings-search.dc.html`](./0.1.0/settings-search.dc.html)       | Settings: the search's results                                         |
| [`row-menu.dc.html`](./0.1.0/row-menu.dc.html)                     | A row's menu: the pointed item filled with the accent colour           |
