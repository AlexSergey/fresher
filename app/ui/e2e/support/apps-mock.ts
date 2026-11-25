import type { Page } from '@playwright/test';

import type {
  AppDetail,
  AppId,
  AppsQuery,
  AppSummary,
  AppsView,
  Refusal,
  UnusedFor,
} from '../../src/ipc/bindings.generated';

import { COMMANDS } from '../../src/ipc';
import { APP_DETAILS, VIDEO_STUDIO_DENIED_DETAIL } from '../../src/test/app-detail';

// The Apps screen's commands of the mocked core: the list under its filter, an app's page and Show in Finder. It states
// only what the core can produce together: the list comes largest first, the filter keeps the apps whose last use is
// known and old enough (counted from the scan's time, never from the clock), and an id of another state is refused.
// `denied` is the world after macOS refused to let Fresher move an app: the page says so.

// The scan's time (1 January 2025).
const TAKEN = 1_735_689_600;

const DAY = 86_400;

const DAYS: Readonly<Record<UnusedFor, number>> = { 'a-year': 365, 'six-months': 182, 'three-months': 90 };

const STATE = 1;

const UNKNOWN_APP = (id: AppId): Refusal => ({
  error: { code: 'unknown-app', id },
  text: { args: {}, key: 'core.error.unknown-app' },
});

const COMMAND_NAMES = [COMMANDS.getApps, COMMANDS.getApp, COMMANDS.revealApp] as const;

type Answer = { readonly answer: unknown } | { readonly refused: Refusal };

// The page's name of the hook the mocked core calls for every command.
export const APPS_HOOK = '__fresherAppsInvoke';

const ANSWER = '__fresherAppsAnswer';

// The apps the mock revealed in Finder, by index, in the page.
export const REVEALED = '__revealedApps';

type Setup = {
  readonly answer: string;
  readonly commands: readonly string[];
  readonly hook: string;
  readonly revealed: string;
};

const listOf = (details: readonly AppDetail[], { unused }: AppsQuery): AppsView => {
  const age = unused === null ? 0 : DAYS[unused] * DAY;
  const kept = details
    .map(({ summary }) => summary)
    .filter(({ lastUsed }) => unused === null || (lastUsed !== null && TAKEN - lastUsed >= age));

  return {
    apps: kept.toSorted((a: AppSummary, b: AppSummary) => b.sizes.allocated - a.sizes.allocated),
    notice: null,
  };
};

// The summary of `detail` while High is withheld (mirrors crates/engine/src/apps/list.rs): no part of a kind whose
// items are at High, and its size and sentence without their bytes.
const withheldSummary = ({ items, summary }: AppDetail): AppSummary => {
  // The engine asks `kind.danger()`; the mock takes High from the items' danger, the same for today's fixtures.
  const high = new Set(items.filter(({ danger }) => danger === 'high').map(({ kind }) => kind));
  const gone = summary.parts
    .filter(({ kind }) => high.has(kind))
    .reduce(
      (sum, { sizes }) => ({
        allocated: sum.allocated + sizes.allocated,
        logical: sum.logical + sizes.logical,
        reclaimable: sum.reclaimable + sizes.reclaimable,
      }),
      { allocated: 0, logical: 0, reclaimable: 0 },
    );
  const sizes = {
    ...summary.sizes,
    allocated: summary.sizes.allocated - gone.allocated,
    logical: summary.sizes.logical - gone.logical,
    reclaimable: summary.sizes.reclaimable - gone.reclaimable,
  };
  const size = summary.summary.args.size;

  return {
    ...summary,
    parts: summary.parts.filter(({ kind }) => !high.has(kind)),
    sizes,
    summary:
      size?.kind === 'size'
        ? { ...summary.summary, args: { ...summary.summary.args, size: { kind: 'size', value: sizes.allocated } } }
        : summary.summary,
  };
};

// Adds the Apps commands to the mocked core of `core-mock.ts`, which calls the hook for every command.
// `withheld` is the world of a core that withholds High: no page offers Everything or lists an item at High, and no
// row of the list shows a part at High or counts its bytes.
export const mockApps = async (
  page: Page,
  { denied = false, withheld = false }: { readonly denied?: boolean; readonly withheld?: boolean } = {},
): Promise<void> => {
  const details = APP_DETAILS.map((detail) =>
    denied && detail.summary.app.index === 0 ? VIDEO_STUDIO_DENIED_DETAIL : detail,
  ).map((detail) =>
    withheld
      ? {
          ...detail,
          choices: detail.choices.filter(({ choice }) => choice !== 'everything'),
          items: detail.items.filter(({ danger }) => danger !== 'high'),
          summary: withheldSummary(detail),
        }
      : detail,
  );

  await page.exposeFunction(ANSWER, (command: string, args: Record<string, unknown>): Answer => {
    if (command === COMMANDS.getApps) {
      return { answer: listOf(details, args.query as AppsQuery) };
    }
    const app = args.app as AppId;
    const detail = app.state === STATE ? details.find(({ summary }) => summary.app.index === app.index) : undefined;

    if (detail === undefined) {
      return { refused: UNKNOWN_APP(app) };
    }

    return { answer: command === COMMANDS.getApp ? detail : null };
  });
  await page.addInitScript(
    (setup: Setup) => {
      const scope = globalThis as unknown as Record<string, unknown>;
      const revealed: number[] = [];

      scope[setup.revealed] = revealed;
      scope[setup.hook] = (command: string, args: Record<string, unknown>): Promise<unknown> | undefined => {
        if (!setup.commands.includes(command)) {
          return undefined;
        }
        const answer = scope[setup.answer] as (command: string, args: Record<string, unknown>) => Promise<Answer>;

        return answer(command, args).then((result) => {
          if ('refused' in result) {
            // eslint-disable-next-line @typescript-eslint/only-throw-error -- the core rejects with a plain object
            throw result.refused;
          }
          if (command === 'reveal_app') {
            revealed.push((args.app as { readonly index: number }).index);
          }

          return result.answer;
        });
      };
    },
    { answer: ANSWER, commands: COMMAND_NAMES, hook: APPS_HOOK, revealed: REVEALED } satisfies Setup,
  );
};
