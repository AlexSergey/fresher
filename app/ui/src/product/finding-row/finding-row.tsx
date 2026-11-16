import { type JSX, useId } from 'react';

import type { VirtualRowProps } from '../../design/data/virtual-list/virtual-list';
import type { Forced } from '../../design/primitives/forced';
import type { Finding, Subject } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { ListRow } from '../../design/data/list-row/list-row';
import { PathText } from '../../design/data/path-text/path-text';
import { SizeText } from '../../design/data/size-text/size-text';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { Skeleton } from '../../design/primitives/skeleton/skeleton';
import { Tooltip } from '../../design/primitives/tooltip/tooltip';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './finding-row.module.css';
import { levelName } from './texts';

export type FindingRowProps = Forced &
  Partial<VirtualRowProps> & {
    // `undefined` while its page loads: the row is a skeleton of the same height.
    readonly finding: Finding | undefined;
    readonly onToggle: (finding: Finding) => void;
    readonly selected: boolean;
  };

// The class of a frame of finding rows: their height (52 px) and the widths of the level and size columns.
export const findingRowsClass = styles.rows ?? '';

const SubjectText = ({ subject }: { readonly subject: Subject }): JSX.Element =>
  subject.kind === 'path' ? (
    <PathText path={subject.value} />
  ) : (
    <span className={styles.external}>
      {t('ui.finding-list.external', {
        id: { kind: 'text', value: subject.value.id },
        tool: { kind: 'text', value: subject.value.tool },
      })}
    </span>
  );

// The row of a page still loading: skeleton lines where the title, the subject and the values will be.
const LoadingRow = ({ row }: { readonly row: Forced & Partial<VirtualRowProps> }): JSX.Element => (
  <div {...row} aria-label={t('ui.finding-list.loading')} className={styles.loading} role="listitem">
    <span className={styles.lines}>
      <span className={styles.title}>
        <Skeleton shape="line" />
      </span>
      <span className={styles.subject}>
        <Skeleton shape="line" />
      </span>
    </span>
    <span className={styles.level}>
      <span className={styles.badge}>
        <Skeleton shape="line" />
      </span>
    </span>
    <span className={styles.value}>
      <span className={styles.amount}>
        <Skeleton shape="line" />
      </span>
    </span>
  </div>
);

// One finding in a row (52 px inside a frame of `findingRowsClass`): its checkbox, its title over its
// path, its danger level and its size. Its accessible name reads title, level and size; its description is the
// reason, which the level's badge also shows in a tooltip. A selected High row lights its edge in High's colour.
export const FindingRow = ({ finding, onToggle, selected, ...row }: FindingRowProps): JSX.Element => {
  const reasonId = useId();

  if (finding === undefined) {
    return <LoadingRow row={row} />;
  }

  const title = tText(finding.title);
  const reason = tText(finding.reason);
  const level = levelName(finding.danger);
  const size = honestSize(finding.sizes);

  return (
    <ListRow
      {...row}
      aria-describedby={reasonId}
      aria-label={t('ui.finding-list.row', {
        level: { kind: 'text', value: level },
        size: { kind: 'text', value: size.accessibleText },
        title: { kind: 'text', value: title },
      })}
      {...(finding.danger === 'high' ? { edgeLevel: 'high' as const } : {})}
      leading={
        <Checkbox
          checked={selected}
          hideLabel
          label={t('ui.finding-list.select', { title: { kind: 'text', value: title } })}
          onChange={() => {
            onToggle(finding);
          }}
          tabIndex={-1}
        />
      }
      selected={selected}
      subtitle={
        <>
          <SubjectText subject={finding.subject} />
          <span hidden id={reasonId}>
            {reason}
          </span>
        </>
      }
      title={title}
      trailing={
        <>
          <span className={styles.level}>
            <Tooltip content={reason} repeatsName>
              {() => <DangerBadge label={level} level={finding.danger} size="small" />}
            </Tooltip>
          </span>
          <span className={styles.value}>
            <SizeText {...size} />
          </span>
        </>
      }
    />
  );
};
