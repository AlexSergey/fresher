import { type JSX, useId } from 'react';

import type { Insight } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { Surface } from '../../design/layout/surface/surface';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { formatBytes } from '../../i18n/format-size';
import styles from './insight-card.module.css';

export type InsightCardProps = {
  readonly insight: Insight;
  readonly onDetails: () => void;
};

// What a module described and does not offer to clean: the title, the summary and the figures, with Details. It is not
// a category: no checkbox, no danger level, no share.
export const InsightCard = ({ insight, onDetails }: InsightCardProps): JSX.Element => {
  const titleId = useId();

  return (
    <Surface elevation={1} padding="none">
      <div className={styles.body}>
        <h3 className={styles.title} id={titleId}>
          {tText(insight.title)}
        </h3>
        <p className={styles.summary}>{tText(insight.summary)}</p>
        {insight.figures.length === 0 ? null : (
          <dl className={styles.figures}>
            {insight.figures.map(({ label, size }) => (
              <div className={styles.figure} key={label.key}>
                <dt>{tText(label)}</dt>
                <dd>
                  <SizeText accessibleText={formatBytes(size)} completeness="complete" text={formatBytes(size)} />
                </dd>
              </div>
            ))}
          </dl>
        )}
        <div className={styles.actions}>
          <Button aria-describedby={titleId} onClick={onDetails} size="small" variant="ghost">
            {t('ui.insight-card.details')}
          </Button>
        </div>
      </div>
    </Surface>
  );
};
