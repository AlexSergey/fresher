import type { JSX } from 'react';

import { Dialog } from '../../design/layout/dialog/dialog';
import { t } from '../../i18n';
import styles from './notices-dialog.module.css';

export type Notices = {
  readonly licences: readonly NoticeLicence[];
  readonly sources: readonly NoticeSource[];
};

export type NoticesDialogProps = {
  // Null while the notices are read.
  readonly notices: Notices | null;
  readonly onClose: () => void;
  readonly open: boolean;
};

// One licence text and the parts of the app under it, as `npm run notices:generate` writes them.
type NoticeLicence = {
  readonly id: string;
  readonly name: string;
  readonly text: string;
  readonly usedBy: readonly string[];
};

// A part taken under a licence outside the allowlist (MPL-2.0), and where its source code is.
type NoticeSource = {
  readonly licence: string;
  readonly name: string;
  readonly url: string;
  readonly version: string;
};

// The addresses are plain, selectable text: the app opens nothing on the network.
const Sources = ({ sources }: { readonly sources: readonly NoticeSource[] }): JSX.Element => (
  <section className={styles.sources}>
    <h3 className={styles.heading}>{t('ui.notices.sources.title')}</h3>
    <p className={styles.line}>{t('ui.notices.sources.text')}</p>
    <ul className={styles.list}>
      {sources.map(({ licence, name, url, version }) => (
        <li key={`${name} ${version}`}>
          {name} {version} · {licence} · <span className={styles.address}>{url}</span>
        </li>
      ))}
    </ul>
  </section>
);

// One licence, folded: its name and how many parts use it; opened, the parts and the full text.
const Licence = ({ licence }: { readonly licence: NoticeLicence }): JSX.Element => (
  <details className={styles.licence}>
    <summary>
      {licence.name} ·{' '}
      <span className={styles.count}>
        {t('ui.notices.used-by', { count: { kind: 'count', value: licence.usedBy.length } })}
      </span>
    </summary>
    <p className={styles.parts}>{licence.usedBy.join(', ')}</p>
    <pre className={styles.text}>{licence.text}</pre>
  </details>
);

// The licences of the open-source parts the app is built from, bundled into the app: no network and no browser.
export const NoticesDialog = ({ notices, onClose, open }: NoticesDialogProps): JSX.Element => (
  <Dialog closeLabel={t('ui.notices.close')} onClose={onClose} open={open} size="large" title={t('ui.notices.title')}>
    {notices === null ? (
      <p className={styles.line}>{t('ui.notices.loading')}</p>
    ) : (
      <div className={styles.content}>
        <p className={styles.line}>{t('ui.notices.intro')}</p>
        {notices.sources.length === 0 ? null : <Sources sources={notices.sources} />}
        <div className={styles.licences}>
          {notices.licences.map((licence) => (
            <Licence key={`${licence.id} ${licence.usedBy[0] ?? ''}`} licence={licence} />
          ))}
        </div>
      </div>
    )}
  </Dialog>
);
