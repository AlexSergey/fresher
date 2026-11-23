import type { AgreementView, Dictionary, Text } from '../ipc/bindings.generated';

import coreTexts from '../../../../crates/core/locales/en.json' with { type: 'json' };

// The agreement as the core builds its view: the core's own texts, read from its locale file. Shared by the unit tests
// and the e2e mock.

export const AGREEMENT_VERSION = 1;

// When the sample agreement was accepted (1 January 2025, noon UTC).
export const AGREEMENT_ACCEPTED_AT = 1_735_732_800;

const text = (key: string): Text => ({ args: {}, key });

const LINES = ['no-warranty', 'your-decision', 'best-judgement', 'backup', 'trash'] as const;

export const AGREEMENT_DICTIONARY: Dictionary = Object.fromEntries(
  Object.entries(coreTexts as Dictionary).filter(
    ([key]) =>
      key.startsWith('core.agreement.') ||
      key === 'core.cleanup.blocked.agreement' ||
      key === 'core.error.agreement-changed',
  ),
);

const view = (accepted: AgreementView['accepted'], changes: AgreementView['changes']): AgreementView => ({
  accepted,
  changes,
  intro: text('core.agreement.intro'),
  lines: LINES.map((id) => text(`core.agreement.line.${id}`)),
  title: text('core.agreement.title'),
  version: AGREEMENT_VERSION,
});

// Accepted: what the core answers after `accept_agreement`.
export const AGREEMENT_ACCEPTED: AgreementView = view({ at: AGREEMENT_ACCEPTED_AT, version: AGREEMENT_VERSION }, null);

// Never accepted: the first cleanup asks.
export const AGREEMENT_NOT_ACCEPTED: AgreementView = view(null, null);

// An older version was accepted and the text changed since (the core has no second version yet: unit tests only).
export const AGREEMENT_CHANGED: AgreementView = {
  ...view({ at: AGREEMENT_ACCEPTED_AT, version: AGREEMENT_VERSION }, text('core.agreement.changes.v2')),
  version: AGREEMENT_VERSION + 1,
};

export const AGREEMENT_CHANGES_DICTIONARY: Dictionary = {
  'core.agreement.changes.v2': 'A line about backups was added.',
};
