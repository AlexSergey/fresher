import type { JSX, ReactNode } from 'react';

import type { Access } from '../../ipc/bindings.generated';

import { Notice } from '../../design/feedback/notice/notice';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import styles from './permission-card.module.css';

export type PermissionCardProps = {
  readonly access: Extract<Access, 'denied' | 'unknown'>;
  // What was not seen and what the permission would let the app see.
  readonly children: ReactNode;
  // The numbered steps that grant the permission, in a collapsible "How".
  readonly how: readonly string[];
  readonly onOpenSettings: () => void;
  // Given once System Settings was opened: a grant applies after the app reopens.
  readonly onReopen?: () => void;
};

const TEXTS = {
  denied: { title: 'ui.permission.denied.title', tone: 'warning' },
  unknown: { title: 'ui.permission.unknown.title', tone: 'info' },
} as const;

// What the app cannot see without Full Disk Access, and the way to grant it: an inline card on the page, never a
// dialog. Open System Settings is a plain button, never the default action.
export const PermissionCard = ({
  access,
  children,
  how,
  onOpenSettings,
  onReopen,
}: PermissionCardProps): JSX.Element => {
  const { title, tone } = TEXTS[access];

  return (
    <Notice action={{ label: t('ui.coverage.open-settings'), onClick: onOpenSettings }} title={t(title)} tone={tone}>
      <div className={styles.body}>
        {children}
        <details className={styles.how}>
          <summary>{t('ui.coverage.how')}</summary>
          <ol className={styles.steps}>
            {how.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>
        </details>
        {onReopen === undefined ? null : (
          <div className={styles.reopen}>
            <Button onClick={onReopen} size="small" variant="secondary">
              {t('ui.coverage.reopen')}
            </Button>
            <span>{t('ui.coverage.reopen.text')}</span>
          </div>
        )}
      </div>
    </Notice>
  );
};
