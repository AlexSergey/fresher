import type { JSX } from 'react';

import type { Access } from '../../ipc/bindings.generated';

import { Notice } from '../../design/feedback/notice/notice';
import { t } from '../../i18n';

export type PermissionCardProps = {
  readonly access: Extract<Access, 'denied' | 'unknown'>;
  readonly onLearnMore?: () => void;
};

const TEXTS = {
  denied: { text: 'ui.permission.denied.text', title: 'ui.permission.denied.title', tone: 'warning' },
  unknown: { text: 'ui.permission.unknown.text', title: 'ui.permission.unknown.title', tone: 'info' },
} as const;

// What the app cannot see without Full Disk Access, and that it still works with less: an inline card on the page,
// never a dialog. Plan 006 adds the action that opens System Settings.
export const PermissionCard = ({ access, onLearnMore }: PermissionCardProps): JSX.Element => {
  const { text, title, tone } = TEXTS[access];

  return (
    <Notice
      {...(onLearnMore === undefined ? {} : { action: { label: t('ui.permission.learn-more'), onClick: onLearnMore } })}
      title={t(title)}
      tone={tone}
    >
      {t(text)}
    </Notice>
  );
};
