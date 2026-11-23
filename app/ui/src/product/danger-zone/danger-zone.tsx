import { type JSX, type ReactNode, useId } from 'react';

import type { DangerLevel } from '../../ipc/bindings.generated';

import { Icon } from '../../design/icons/icon';
import { OctagonAlert, TriangleAlert } from '../../design/icons/icons';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { t, tText } from '../../i18n';
import styles from './danger-zone.module.css';

export type DangerZoneProps = {
  readonly children: ReactNode;
  readonly level: DangerLevel;
  // Inside one of Review's parts: the heading is a third-level one named by the level, under the part's.
  readonly nested?: boolean;
  // Scan's "Select all" for the zone; absent on Review, where everything listed is already selected.
  readonly selectAll?: undefined | { readonly checked: 'mixed' | boolean; readonly onChange: () => void };
};

// Only the two risky levels carry a warning icon; both have an exclamation mark, so the shape tells them from the rest.
const WARNINGS = { high: OctagonAlert, medium: TriangleAlert } as const;

// The categories of one danger level: the zone's name and what the level means, the level's colour in a rule above it,
// and, on Scan, a "Select all" for the whole zone. Nested in a part of Review, it is named by its level.
export const DangerZone = ({ children, level, nested = false, selectAll }: DangerZoneProps): JSX.Element => {
  const headingId = useId();
  const warning = level === 'medium' || level === 'high' ? WARNINGS[level] : null;
  const Heading = nested ? 'h3' : 'h2';

  return (
    <section aria-labelledby={headingId} className={styles.zone} data-level={level}>
      <div className={styles.head}>
        <div className={styles.title}>
          {warning === null ? null : (
            <span aria-hidden="true" className={styles.warning}>
              <Icon icon={warning} size={20} />
            </span>
          )}
          <Heading className={styles.name} id={headingId}>
            {tText({ args: {}, key: `core.danger.${level}.${nested ? 'name' : 'zone'}` })}
          </Heading>
        </div>
        <p className={styles.meaning}>{tText({ args: {}, key: `core.danger.${level}.meaning` })}</p>
        {selectAll === undefined ? null : (
          <div className={styles.select}>
            <Checkbox
              aria-describedby={headingId}
              checked={selectAll.checked}
              description={t('ui.danger-zone.select-all.description')}
              label={t('ui.danger-zone.select-all')}
              onChange={selectAll.onChange}
            />
          </div>
        )}
      </div>
      {children}
    </section>
  );
};
