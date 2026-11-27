import type { JSX } from 'react';

import type { VirtualRowProps } from '../../design/data/virtual-list/virtual-list';
import type { Forced } from '../../design/primitives/forced';
import type { FolderRow as FolderRowData, LeftFor } from '../../ipc/bindings.generated';

import { ListRow } from '../../design/data/list-row/list-row';
import { SizeText } from '../../design/data/size-text/size-text';
import { Badge } from '../../design/primitives/badge/badge';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import { MiddlePath } from '../middle-path/middle-path';
import { honestSize } from '../sizes';
import styles from './folder-row.module.css';

export type FolderRowProps = Forced &
  Partial<VirtualRowProps> & {
    readonly onReveal: () => void;
    readonly row: FolderRowData;
  };

// The class of a frame of these rows: their height, for the lines under the name.
export const folderRowsClass = styles.rows ?? '';

const leftForKey = {
  months: 'ui.folder-row.months',
  years: 'ui.folder-row.years',
} as const satisfies Record<LeftFor['unit'], string>;

// One folder left alone: its name over where it is and how long nothing in it changed, then Show in Finder and the
// size. The row is information; Fresher never removes it, so there is nothing to tick.
export const FolderRow = ({ onReveal, row, ...forced }: FolderRowProps): JSX.Element => {
  const { leftFor, node, place, project } = row;
  const size = honestSize(node.sizes);

  return (
    <ListRow
      {...forced}
      subtitle={
        <>
          <span className={styles.line}>
            <MiddlePath path={place} />
            {project ? <Badge>{t('ui.folder-row.project')}</Badge> : null}
          </span>
          <span className={styles.line}>
            {t(leftForKey[leftFor.unit], { count: { kind: 'count', value: leftFor.count } })}
          </span>
        </>
      }
      title={node.name}
      trailing={
        <>
          <Button
            aria-label={t('ui.folder-row.reveal-label', { name: { kind: 'text', value: node.name } })}
            onClick={onReveal}
            size="small"
            variant="secondary"
          >
            {t('ui.folder-row.reveal')}
          </Button>
          <span className={styles.value}>
            <SizeText {...size} />
          </span>
        </>
      }
    />
  );
};
