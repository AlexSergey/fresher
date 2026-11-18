import type { JSX } from 'react';

import type { MapPlace, NodeInfo } from '../../ipc/bindings.generated';

import { ListRow } from '../../design/data/list-row/list-row';
import { SizeText } from '../../design/data/size-text/size-text';
import { ChevronRight } from '../../design/icons/icons';
import { IconButton } from '../../design/primitives/icon-button/icon-button';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './place-list.module.css';

export type RootListProps = {
  readonly onOpen: (node: NodeInfo) => void;
  // The roots that are places walked outside the scan roots, each with the core's name for it.
  readonly places: readonly MapPlace[];
  // The volume's scan roots, then its places, in the order scanned.
  readonly roots: readonly NodeInfo[];
};

// A root's name: a place's from the core's text, any other root's its folder's.
const nameOf = (root: NodeInfo, places: readonly MapPlace[]): string => {
  const place = places.find(({ root: id }) => id.state === root.id.state && id.index === root.id.index);

  return place === undefined ? root.name : tText(place.name);
};

// The scanned folders of a volume that holds several, one row each with its size and a way into it; the place view
// starts here.
export const RootList = ({ onOpen, places, roots }: RootListProps): JSX.Element => (
  <div aria-label={t('ui.place-list.roots')} className={styles.roots} role="list">
    {roots.map((root) => {
      const name = nameOf(root, places);

      return (
        <ListRow
          key={`${String(root.id.state)}-${String(root.id.index)}`}
          title={name}
          trailing={
            <>
              <span className={styles.value}>
                <SizeText {...honestSize(root.sizes)} />
              </span>
              <IconButton
                icon={ChevronRight}
                label={t('ui.place-list.open', { name: { kind: 'text', value: name } })}
                onClick={() => {
                  onOpen(root);
                }}
                size="small"
              />
            </>
          }
        />
      );
    })}
  </div>
);
