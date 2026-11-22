import { type JSX, useId } from 'react';

import type { TrashLine } from '../../ipc/bindings.generated';

import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import { formatSize } from '../../i18n/format-size';
import styles from './trash-line.module.css';

export type TrashLinesProps = {
  readonly lines: readonly TrashLine[];
  readonly onOpen: (index: number) => void;
};

const text = (value: string): { readonly kind: 'text'; readonly value: string } => ({ kind: 'text', value });

const sentenceOf = ({ fromCleanups, waiting }: TrashLine): string => {
  const share = text(formatSize(fromCleanups.allocated, fromCleanups.completeness));

  if (waiting === null) {
    return t('ui.trash.share-only', { share });
  }

  const size = text(formatSize(waiting.allocated, waiting.completeness));

  if (fromCleanups.allocated === 0) {
    return t('ui.trash.waiting', { size });
  }

  return fromCleanups.allocated >= waiting.allocated
    ? t('ui.trash.waiting-all-ours', { size })
    : t('ui.trash.waiting-share', { share, size });
};

const Block = ({
  line,
  named,
  onOpen,
}: {
  readonly line: TrashLine;
  readonly named: boolean;
  readonly onOpen: (index: number) => void;
}): JSX.Element => {
  const inviteId = useId();

  return (
    <div className={styles.block}>
      {named && line.volume !== null ? <p className={styles.volume}>{line.volume}</p> : null}
      <p className={styles.text}>{sentenceOf(line)}</p>
      {line.goneBy === null ? null : (
        <p className={styles.text}>{t('ui.trash.gone-by', { date: { kind: 'date', value: line.goneBy } })}</p>
      )}
      <p className={styles.text} id={inviteId}>
        {t('ui.trash.invite')}
      </p>
      <div>
        <Button
          aria-describedby={inviteId}
          onClick={() => {
            onOpen(line.index);
          }}
          size="small"
        >
          {t('ui.trash.open')}
        </Button>
      </div>
    </div>
  );
};

// What waits in each Trash and how much of it this app's cleanups put there. Information only: the one action is
// Open Trash, which shows the folder in Finder; nothing here empties a Trash.
export const TrashLines = ({ lines, onOpen }: TrashLinesProps): JSX.Element | null =>
  lines.length === 0 ? null : (
    <section aria-label={t('ui.trash.label')} className={styles.section}>
      {lines.map((line) => (
        <Block key={line.index} line={line} named={lines.length > 1} onOpen={onOpen} />
      ))}
    </section>
  );
