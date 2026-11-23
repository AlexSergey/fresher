import { type JSX, useRef } from 'react';

import type { AgreementView } from '../../ipc/bindings.generated';

import { Notice } from '../../design/feedback/notice/notice';
import { Sheet } from '../../design/layout/sheet/sheet';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { AgreementLines } from './agreement-lines';
import styles from './agreement-sheet.module.css';

export type AgreementSheetProps = {
  readonly onAccept: () => void;
  // Not now, Escape and the close button: nothing is prepared.
  readonly onDecline: () => void;
  readonly onLicence: () => void;
  readonly open: boolean;
  readonly view: AgreementView;
};

// The agreement, asked before the first cleanup is prepared: Not now has the first focus, so a stray Enter accepts
// nothing. Static: nothing moves but the sheet's own entrance.
export const AgreementSheet = ({ onAccept, onDecline, onLicence, open, view }: AgreementSheetProps): JSX.Element => {
  const laterRef = useRef<HTMLButtonElement>(null);

  return (
    <Sheet
      closeLabel={t('ui.screen.frame.close')}
      footer={
        <>
          <Button onClick={onDecline} ref={laterRef} variant="secondary">
            {t('ui.agreement.later')}
          </Button>
          <Button onClick={onAccept} variant="primary">
            {t('ui.agreement.accept')}
          </Button>
        </>
      }
      initialFocus={laterRef}
      onClose={onDecline}
      open={open}
      title={tText(view.title)}
    >
      <div className={styles.content}>
        {view.changes === null ? null : (
          <Notice title={t('ui.agreement.changed')} tone="info">
            {tText(view.changes)}
          </Notice>
        )}
        <p className={styles.intro}>{tText(view.intro)}</p>
        <AgreementLines view={view} />
        <div>
          <Button onClick={onLicence} size="small" variant="ghost">
            {t('ui.agreement.licence')}
          </Button>
        </div>
      </div>
    </Sheet>
  );
};
