import type { JSX } from 'react';

import type { NodeInfo } from '../../ipc/bindings.generated';

import { ListRow } from '../../design/data/list-row/list-row';
import { SizeText } from '../../design/data/size-text/size-text';
import { ChevronRight } from '../../design/icons/icons';
import { IconButton } from '../../design/primitives/icon-button/icon-button';
import { t } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './place-list.module.css';

export type RootListProps = {
  readonly onOpen: (node: NodeInfo) => void;
  // The volume's scan roots, in the order scanned.
  readonly roots: readonly NodeInfo[];
};

// The scanned folders of a volume that holds several, one row each with its size and a way into it; the place view
// starts here.
export const RootList = ({ onOpen, roots }: RootListProps): JSX.Element => (
  <div aria-label={t('ui.place-list.roots')} className={styles.roots} role="list">
    {roots.map((root) => (
      <ListRow
        key={`${String(root.id.state)}-${String(root.id.index)}`}
        title={root.name}
        trailing={
          <>
            <span className={styles.value}>
              <SizeText {...honestSize(root.sizes)} />
            </span>
            <IconButton
              icon={ChevronRight}
              label={t('ui.place-list.open', { name: { kind: 'text', value: root.name } })}
              onClick={() => {
                onOpen(root);
              }}
              size="small"
            />
          </>
        }
      />
    ))}
  </div>
);
