import { type JSX, useState } from 'react';

import { Dialog } from '../../design/layout/dialog/dialog';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import styles from './recovery-help.module.css';

// The issue form for a wrong finding: `.github/ISSUE_TEMPLATE/wrong-finding.yml`. Shown as text to copy; the app opens
// no address and sends nothing.
const REPORT_ADDRESS = 'https://github.com/AlexSergey/fresher/issues/new?template=wrong-finding.yml';

// Each key is written out: a key no file names is reported as unused.
const PARTS = [
  { text: 'ui.recovery.put-back.text', title: 'ui.recovery.put-back.title' },
  { text: 'ui.recovery.trash.text', title: 'ui.recovery.trash.title' },
  { text: 'ui.recovery.permanent.text', title: 'ui.recovery.permanent.title' },
] as const;

export type RecoveryHelpProps = {
  // Starts open: for the gallery's story of the dialog.
  readonly defaultOpen?: boolean;
};

// "Removed something you need?": a ghost button that opens four short parts, from putting it back to telling us.
export const RecoveryHelp = ({ defaultOpen = false }: RecoveryHelpProps): JSX.Element => {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <>
      <Button
        onClick={() => {
          setOpen(true);
        }}
        variant="ghost"
      >
        {t('ui.recovery.button')}
      </Button>
      <Dialog
        closeLabel={t('ui.recovery.close')}
        onClose={() => {
          setOpen(false);
        }}
        open={open}
        title={t('ui.recovery.title')}
      >
        <div className={styles.content}>
          {PARTS.map((part) => (
            <section className={styles.part} key={part.title}>
              <h3 className={styles.heading}>{t(part.title)}</h3>
              <p className={styles.text}>{t(part.text)}</p>
            </section>
          ))}
          <section className={styles.part}>
            <h3 className={styles.heading}>{t('ui.recovery.report.title')}</h3>
            <p className={styles.text}>{t('ui.recovery.report.text')}</p>
            <code className={styles.address}>{REPORT_ADDRESS}</code>
          </section>
        </div>
      </Dialog>
    </>
  );
};
