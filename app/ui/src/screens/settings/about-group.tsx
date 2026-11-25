import { type JSX, useState } from 'react';

import { Surface } from '../../design/layout/surface/surface';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { AgreementLines } from '../../product/agreement-sheet/agreement-lines';
import { LicenceDialog } from '../../product/licence-dialog/licence-dialog';
import { NoticesDialog } from '../../product/notices-dialog/notices-dialog';
import { isAccepted } from '../../state/agreement';
import { useCore } from '../../state/store';
import { useAppInfo } from '../use-app-info';
import styles from './settings-screen.module.css';
import { useNotices } from './use-notices';

// The app's version, the agreement and when it was accepted, the licence, and the licences of the open-source parts
// it is built from. Named by the page's heading.
export const AboutGroup = ({ headingId }: { readonly headingId: string }): JSX.Element => {
  const version = useAppInfo()?.version ?? null;
  const agreement = useCore((state) => state.agreement.view);
  const [licence, setLicence] = useState(false);
  const [noticesOpen, setNoticesOpen] = useState(false);
  const notices = useNotices(noticesOpen);

  return (
    <section aria-labelledby={headingId} className={styles.group}>
      <Surface elevation={1} padding="none">
        <div className={styles.rows}>
          {version === null ? null : (
            <p className={styles.line}>{t('ui.app.version', { version: { kind: 'text', value: version } })}</p>
          )}
          {agreement === null ? null : (
            <div className={styles.agreement}>
              <p className={styles.agreementTitle}>{tText(agreement.title)}</p>
              <AgreementLines view={agreement} />
              <p className={styles.groupText}>
                {isAccepted(agreement)
                  ? t('ui.agreement.accepted-on', { date: { kind: 'date', value: agreement.accepted.at } })
                  : t('ui.agreement.not-accepted')}
              </p>
            </div>
          )}
          <p className={styles.line}>{t('ui.screen.settings.about.licence')}</p>
          <div className={styles.line}>
            <Button
              onClick={() => {
                setLicence(true);
              }}
              size="small"
              variant="secondary"
            >
              {t('ui.agreement.licence')}
            </Button>
          </div>
          <p className={styles.line}>{t('ui.screen.settings.about.notices')}</p>
          <div className={styles.line}>
            <Button
              onClick={() => {
                setNoticesOpen(true);
              }}
              size="small"
              variant="secondary"
            >
              {t('ui.notices.open')}
            </Button>
          </div>
        </div>
      </Surface>
      <LicenceDialog
        onClose={() => {
          setLicence(false);
        }}
        open={licence}
      />
      <NoticesDialog
        notices={notices}
        onClose={() => {
          setNoticesOpen(false);
        }}
        open={noticesOpen}
      />
    </section>
  );
};
