import { type JSX, useRef } from 'react';

import type { DangerLevel } from '../../ipc/bindings.generated';

import { Dialog } from '../../design/layout/dialog/dialog';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import { levelMeaning, levelName } from '../../product/finding-row/texts';
import styles from './scan-screen.module.css';

export type SelectAllQuestionProps = {
  // The zone's level; kept while the dialog fades out.
  readonly level: DangerLevel;
  // Cancel, Escape and the close button: nothing is selected.
  readonly onCancel: () => void;
  readonly onSelect: () => void;
  readonly open: boolean;
};

// "Select all" of a Medium or High zone asks first: what the level means, and that nothing is removed before Review.
// Cancel has the first focus, so Enter selects nothing.
export const SelectAllQuestion = ({ level, onCancel, onSelect, open }: SelectAllQuestionProps): JSX.Element | null => {
  const cancelRef = useRef<HTMLButtonElement>(null);

  return (
    <Dialog
      closeLabel={t('ui.screen.frame.close')}
      footer={
        <>
          <Button onClick={onCancel} ref={cancelRef} variant="secondary">
            {t('ui.danger-zone.ask.cancel')}
          </Button>
          <Button onClick={onSelect} variant="primary">
            {t('ui.danger-zone.ask.confirm')}
          </Button>
        </>
      }
      initialFocus={cancelRef}
      onClose={onCancel}
      open={open}
      title={t('ui.danger-zone.ask.title', { level: { kind: 'text', value: levelName(level) } })}
    >
      <div className={styles.askText}>
        <p>{levelMeaning(level)}</p>
        <p>{t('ui.danger-zone.ask.text')}</p>
      </div>
    </Dialog>
  );
};
