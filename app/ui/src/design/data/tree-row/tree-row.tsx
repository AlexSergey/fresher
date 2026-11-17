import type { CSSProperties, JSX } from 'react';

import { Icon } from '../../icons/icon';
import { ChevronRight } from '../../icons/icons';
import { ListRow, type ListRowProps } from '../list-row/list-row';
import styles from './tree-row.module.css';

export type TreeRowProps = ListRowProps & {
  // The id of the element holding the row's children, when the caller renders them as a group.
  readonly controls?: string;
  // Whether the row's children are shown; the caller renders them.
  readonly expanded: boolean;
  // The depth: 0 for a top row, each level indents one step.
  readonly level?: number;
  readonly onExpandedChange: (expanded: boolean) => void;
  // The disclosure's accessible name ("Show the items of Xcode").
  readonly toggleLabel: string;
};

// A list row with a disclosure before its leading slot: the chevron turns to point down when the row is expanded.
export const TreeRow = ({
  controls,
  disabled = false,
  expanded,
  leading,
  level = 0,
  onExpandedChange,
  toggleLabel,
  ...row
}: TreeRowProps): JSX.Element => (
  <ListRow
    {...row}
    disabled={disabled}
    leading={
      <span className={styles.indent} style={{ '--tree-level': level } as CSSProperties}>
        <button
          aria-controls={controls}
          aria-expanded={expanded}
          aria-label={toggleLabel}
          className={styles.disclosure}
          disabled={disabled}
          onClick={() => {
            onExpandedChange(!expanded);
          }}
          type="button"
        >
          <span className={styles.chevron}>
            <Icon icon={ChevronRight} />
          </span>
        </button>
        {leading}
      </span>
    }
  />
);
