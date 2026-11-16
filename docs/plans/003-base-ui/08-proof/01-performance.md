# 003/08/01 — Performance

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/e2e/perf.spec.ts` — run only by `npm run test:perf --workspace app/ui` (a `perf` Playwright project, headed
  WebKit, full motion; excluded from `test:e2e`):
  - a frame sampler injected before load: `requestAnimationFrame` intervals into `window.__frames__`, and long tasks
    where the engine reports them (`PerformanceObserver('longtask')` in Chromium);
  - scenario 1, **the scan**: the mocked core streams `large-scan.ts` (50,000 findings, 12 categories) in batches of
    256 every 50 ms while the overview is open; then the same on the module page with the largest category expanded;
  - scenario 2, **the transitions**: each screen change, opening and closing the confirmation dialog (from the gallery),
    the selection bar appearing, a re-sort of 12 cards;
  - scenario 3, **the scroll**: the 100,000-row list scrolled from top to bottom at a steady speed;
  - for each: the 50th and 95th percentile and the longest frame, the number of frames over 50 ms, the DOM node count
    of the list, the JS heap (Chromium) before and after; printed as a table and attached to the report.
- `app/ui/package.json` — `test:perf`.
- `app/ui/playwright.config.ts` — the `perf` project (WebKit, headed, `testMatch: 'perf.spec.ts'`); the other projects
  ignore it.
- The bundle check: `npm run build` and the gzipped sizes of the main entry's JS and CSS, compared with the budget
  (200 kB, 40 kB).

## Produces

The measured numbers for the conclusions, against the design's budget: p95 ≤ 17 ms, no frame over 50 ms, at most about
60 list rows in the DOM, at most 1,000 findings held, the bundle within its budget.

## Checks

- Run on the reference Mac (named in the results, with its macOS and the WebKit version), three runs per scenario, the
  median written in this step's notes and in the plan's conclusions.
- A scenario over budget is fixed before the plan is done (the CSS guard and the profile point at the cause), or, if
  it cannot be, recorded in `docs/known-issues/` with what was tried.
- Chrome's Performance panel (or Safari's Timelines) on scenario 1, looked at once by hand: no forced layout in the
  frames, no paint storm outside the named exceptions.

## Conclusions

Done on 2026-11-16. `npm run test:perf --workspace app/ui` runs `e2e/perf.spec.ts` in two Playwright projects, `perf`
(WebKit, the reference: the budget is held there) and `perf-chromium` (reports only: long tasks and the JS heap);
`test:e2e` now names its three projects, so it never runs the perf spec. No new dependency, token or app code.

**The reference Mac**: MacBook Pro, Apple M3 Pro (`Mac15,7`), built-in Liquid Retina XDR display, macOS 27.2.1;
Playwright 1.65.1 with its WebKit 27.2 and Chromium 159.0.8154.6, headed, 1100 × 720 at 2×. Playwright's WebKit draws
at 60 Hz (a frame every 16.7 ms on average), Chromium at 120 Hz.

**How it is measured**: a sampler injected before load records every `requestAnimationFrame` interval, the long tasks
(Chromium) and the most `role="listitem"` rows in the DOM at any frame; the JS heap is read in Chromium through its
DevTools protocol after a full garbage collection, before and after each scenario. A measured window runs from the
action until no finite animation runs (at least 800 ms). Each scenario runs three times on a new page; the table holds
the median of each number. The mock streams the scan as the engine does: one event every 50 ms, each asked of Node as
JSON and parsed in the page (`interval` in `e2e/core-mock.ts`), with `get_overview` answering the totals of what was
streamed so far (`scanningOverviewOf` in `src/test/large-scan.ts`).

**The numbers** (WebKit, the median of three runs; two full runs of the suite, the second one written; "dropped" is
frames over 25 ms, a missed refresh):

| Scenario                          | p50 ms | p95 ms | Longest ms | Dropped | Over 50 ms | Rows | Nodes |
| --------------------------------- | ------ | ------ | ---------- | ------- | ---------- | ---- | ----- |
| idle overview (calibration)       | 17     | 18     | 18         | 0       | 0          | 0    | 103   |
| scan of 50,000, overview          | 17     | 22     | 86         | 12      | 1          | 0    | 609   |
| scan of 50,000, module page       | 17     | 18     | 33         | 9       | 0          | 27   | 1,075 |
| screen, overview → module         | 17     | 18     | 19         | 0       | 0          | 4    | 237   |
| screen, module → review           | 17     | 18     | 18         | 0       | 0          | 5    | 158   |
| screen, review → settings         | 17     | 18     | 18         | 0       | 0          | 1    | 84    |
| screen, settings → overview       | 17     | 18     | 20         | 0       | 0          | 0    | 194   |
| selection bar appears             | 17     | 18     | 18         | 0       | 0          | 4    | 259   |
| re-sort of 12 cards               | 17     | 18     | 18         | 0       | 0          | 0    | 609   |
| dialog opens (gallery)            | 17     | 18     | 19         | 0       | 0          | 3    | 145   |
| dialog closes (gallery)           | 17     | 18     | 18         | 0       | 0          | 3    | 124   |
| scroll, 100,000 rows in 10 s      | 17     | 18     | 23         | 0       | 0          | 21   | 254   |

The first full run measured the same within a millisecond, except the overview scan: p95 19 ms, longest 30 ms, no
frame over 50 ms. Chromium (120 Hz): every scenario at p50 8.3 ms and p95 9–10 ms, longest 17 ms, no long task, no
frame over 50 ms; the JS heap after a full collection grows from 2.7 MB to 4.9 MB over the 50,000-finding scan on the
overview and from 3.2 MB to 5.3 MB on the module page, so the page holds no scan (the store keeps no finding; the
lists' cache is at most 10 pages of 100, unit-tested in `state/findings.test.ts`). The bundle, gzipped, from
`vite build`: **JS 99.1 kB of 200, CSS 12.4 kB of 40**.

**Against the budget**:

- **No frame over 50 ms**: held everywhere but the overview during the scan, where one run in three of the second full
  run had one frame over 50 ms (86 ms). It came with the machine's load: during both runs other work shared the Mac
  (another worktree's release-build scan benchmark at over 200 % CPU and another agent's Playwright runs; load average
  11–17). Measured alone in 13 more runs of that scenario, none had a frame over 50 ms: 8 runs at a load of 3–5 had a
  p95 of 18 ms and frames over 25 ms only at the scan's start (26–38 ms, the hero changing to its scanning state) and
  at its end; 5 runs at a load of 14–17 had a p95 of 18–23 ms and a longest frame of 46 ms. Chromium, at twice
  WebKit's refresh rate, keeps 120 fps through the same scan with no long task (in the first full run, under the
  same load, it fell to 60 fps there with one 68 ms long task). So no defect is shown and nothing was
  changed; the spec keeps the bar, and a run on a busy Mac can fail it.
- **p95 ≤ 17 ms**: WebKit's frame clock counts whole milliseconds and jitters: an empty page reads intervals of
  15–18 ms (mean 16.7) and a p95 of 18 ms, 19 under load. The spec measures that idle page first in every test and
  holds each scenario's p95 to 17 ms or the idle page's p95 of the same run, whichever is larger (an agent's call, to
  check). Every scenario but the overview scan under load sits at the idle page's p95.
- **About 60 rows at most**: 27 on the module page (the largest category expanded), 21 in the 100,000-row list while
  it scrolls 4.4 million pixels in ten seconds.
- **At most 1,000 findings held**: by the heap above; the measurement cannot count findings in the page directly.
- **The bundle**: within, at half of each budget.

**The look at the profile** (Chrome's trace of the overview scan, read once): the scan's endless loops paint nothing
(a held scan: no paint in three seconds); during the stream each frame lays out once and checks the hover once (two
layouts per task, never more), 112 style or layout passes forced by script in 11 s, the longest 2.8 ms; the main
thread is busy about a third of the time. The twelve cards repaint on most frames while their numbers tween toward the
overview refreshed four times a second: the text changes, so it repaints (about 1 ms a frame in all); not a CSS
animation, so not one of the named exceptions, and within the budget.

Decided while building (agent's calls, to check):

- **Two projects, one spec**: WebKit holds the budget; Chromium runs the same scenarios to report what only it
  reports. `test:perf` runs both with one worker.
- **The re-sort** is a held scan of twelve categories whose final overview reverses their order, so every card glides
  (the grid has no other re-sort: the order is always largest first).
- **The dialog** is opened and closed by moving the gallery to and from the `ConfirmDialog/trash` story: no story sets
  `open` to false, so the measured close is the dialog's unmount, not its 120 ms fade-out (opacity only).
- **The scroll** drives `scrollTop` from `requestAnimationFrame` at a constant speed, top to bottom in ten seconds:
  every frame shows new rows, harder than a person's scroll.
- **The mock asks Node for each event** instead of carrying the whole scan into the page: carried in, the 50,000
  findings were 25 MB of JSON held by the page, and its garbage collection stalled a frame for up to 137 ms in the
  middle of a scan, which the app does not do.
- **The bundle check** is a test of the perf spec: it runs `vite build` into the test's output folder and gzips every
  script and stylesheet `index.html` loads.

