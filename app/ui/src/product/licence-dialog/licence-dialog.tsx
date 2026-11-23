import type { JSX } from 'react';

import apache from '../../../../../LICENSE-APACHE?raw';
import mit from '../../../../../LICENSE-MIT?raw';
import { Dialog } from '../../design/layout/dialog/dialog';
import { t } from '../../i18n';
import styles from './licence-dialog.module.css';

export type LicenceDialogProps = {
  readonly onClose: () => void;
  readonly open: boolean;
};

// One licence file under its name; the dialog's body scrolls it.
const Licence = ({ name, text }: { readonly name: string; readonly text: string }): JSX.Element => (
  <div>
    <h3 className={styles.heading}>{name}</h3>
    <pre className={styles.text}>{text}</pre>
  </div>
);

// The full licence texts, bundled into the app: no network and no browser. Both are selectable.
export const LicenceDialog = ({ onClose, open }: LicenceDialogProps): JSX.Element => (
  <Dialog closeLabel={t('ui.licence.close')} onClose={onClose} open={open} size="large" title={t('ui.licence.title')}>
    <div className={styles.content}>
      <Licence name={t('ui.licence.mit')} text={mit} />
      <Licence name={t('ui.licence.apache')} text={apache} />
    </div>
  </Dialog>
);
