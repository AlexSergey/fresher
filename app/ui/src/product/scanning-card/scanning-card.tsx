import type { JSX } from 'react';

import type { Sizes } from '../../ipc/bindings.generated';

import { SwooshProgress } from '../../design/data/swoosh-progress/swoosh-progress';
import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import { t } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './scanning-card.module.css';

export type ScanningCardProps = {
  // What the scan found so far.
  readonly found: Sizes;
  // The name of the part the scan looks at; `null` when none is known.
  readonly module: null | string;
  // 0 to 100.
  readonly percent: number;
  readonly volumeName: string;
};

// The scan in place of the disk card: the swoosh fills with the progress, the percent, what the scan looks at and
// what it found so far.
export const ScanningCard = ({ found, module, percent, volumeName }: ScanningCardProps): JSX.Element => {
  const { accessibleText, completeness, text } = honestSize(found);
  const said = { kind: 'text', value: String(percent) } as const;

  return (
    <section aria-label={t('ui.scanning.label')} className={styles.card}>
      <div className={styles.swoosh}>
        <SwooshProgress label={t('ui.scanning.progress', { percent: said })} value={percent / 100} />
      </div>
      <div className={styles.top}>
        <span className={styles.title}>{t('ui.scanning.title', { name: { kind: 'text', value: volumeName } })}</span>
        <span aria-hidden="true" className={styles.percent} data-numeric>
          {t('ui.scanning.percent', { percent: said })}
        </span>
        {module === null ? null : (
          <span className={styles.looking}>
            {t('ui.scanning.looking-at', { module: { kind: 'text', value: module } })}
          </span>
        )}
      </div>
      <div className={styles.bottom}>
        <span className={styles.found}>
          <span aria-hidden="true" data-numeric>
            {completeness === 'unknown' ? '—' : text}
            {completeness === 'partial' ? '+' : null}
          </span>
          <VisuallyHidden>{accessibleText}</VisuallyHidden>
        </span>
        <span className={styles.caption}>{t('ui.scanning.found')}</span>
      </div>
    </section>
  );
};
