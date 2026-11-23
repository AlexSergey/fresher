import type { Page } from '@playwright/test';

import type { AgreementView, Refusal } from '../../src/ipc/bindings.generated';

import { COMMANDS } from '../../src/ipc';
import { AGREEMENT_ACCEPTED, AGREEMENT_NOT_ACCEPTED, AGREEMENT_VERSION } from '../../src/test/agreement';

// The agreement's commands of the mocked core. It states only what the core can produce together: the agreement is
// accepted by default; `not-accepted` answers a view without acceptance until `accept_agreement` is called, which
// records the version and answers the accepted view; another version is refused as the core does. Until then,
// `prepare_cleanup` and `prepare_uninstall` are refused for the agreement (`CleaningBlocked`), as the core refuses them.
// The core has no second version yet, so the mock never answers a `changes` text.
export type AgreementScript = 'accepted' | 'not-accepted';

// The page's name of the hook the mocked core calls for every command.
export const AGREEMENT_HOOK = '__fresherAgreementInvoke';

// The versions `accept_agreement` was called with, in the page.
const ACCEPTED_VERSIONS = '__fresherAcceptedVersions';

const CHANGED: Refusal = {
  error: { code: 'agreement-changed' },
  text: { args: {}, key: 'core.error.agreement-changed' },
};

const BLOCKED: Refusal = {
  error: { code: 'cleaning-blocked', reason: 'agreement' },
  text: { args: {}, key: 'core.error.cleaning-blocked' },
};

type Setup = {
  readonly accepted: AgreementView;
  readonly blocked: Refusal;
  readonly changed: Refusal;
  readonly commands: typeof COMMANDS;
  readonly hook: string;
  readonly notAccepted: AgreementView;
  readonly recorded: string;
  readonly start: AgreementScript;
  readonly version: number;
};

export const mockAgreement = async (page: Page, start: AgreementScript = 'accepted'): Promise<void> => {
  await page.addInitScript(
    (setup: Setup) => {
      const scope = globalThis as unknown as Record<string, unknown>;
      const recorded: number[] = [];
      let accepted = setup.start === 'accepted';

      scope[setup.recorded] = recorded;
      scope[setup.hook] = (command: string, args: Record<string, unknown>): Promise<unknown> | undefined => {
        if (command === setup.commands.getAgreement) {
          return Promise.resolve(accepted ? setup.accepted : setup.notAccepted);
        }
        if (!accepted && (command === setup.commands.prepareCleanup || command === setup.commands.prepareUninstall)) {
          // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- the core rejects with a plain object
          return Promise.reject(setup.blocked);
        }
        if (command !== setup.commands.acceptAgreement) {
          return undefined;
        }
        recorded.push(args.version as number);
        if (args.version !== setup.version) {
          // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- the core rejects with a plain object
          return Promise.reject(setup.changed);
        }
        accepted = true;

        return Promise.resolve(setup.accepted);
      };
    },
    {
      accepted: AGREEMENT_ACCEPTED,
      blocked: BLOCKED,
      changed: CHANGED,
      commands: COMMANDS,
      hook: AGREEMENT_HOOK,
      notAccepted: AGREEMENT_NOT_ACCEPTED,
      recorded: ACCEPTED_VERSIONS,
      start,
      version: AGREEMENT_VERSION,
    } satisfies Setup,
  );
};

// The versions the page accepted, in order.
export const acceptedVersions = (page: Page): Promise<readonly number[]> =>
  page.evaluate((name) => (globalThis as unknown as Record<string, number[]>)[name] ?? [], ACCEPTED_VERSIONS);
