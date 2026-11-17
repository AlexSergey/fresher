import { type JSX, useId, useState } from 'react';

import type { AppRef, CategoryId, CategoryShare, NodeId, PlaceShare, Sizes } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { TreeRow } from '../../design/data/tree-row/tree-row';
import { t } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './meaning-list.module.css';
import { CategoryRow, PlaceRow } from './rows';

export type AppRowProps = {
  readonly app: AppRef;
  readonly categories: readonly CategoryShare[];
  readonly data: readonly PlaceShare[];
  // Opened at first: a story shows what an app keeps.
  readonly defaultExpanded?: boolean;
  readonly onOpen: (category: CategoryId) => void;
  readonly onOpenPlace: (node: NodeId) => void;
  readonly sizes: Sizes;
};

// An app and what it keeps: its name, "not installed" when it was removed, its size; opened, its categories and its
// data folders, one row each, in a group the disclosure controls. The app's row is a list of its own, so the group
// beside it sits in no list.
export const AppRow = ({
  app,
  categories,
  data,
  defaultExpanded = false,
  onOpen,
  onOpenPlace,
  sizes,
}: AppRowProps): JSX.Element => {
  const [expanded, setExpanded] = useState(defaultExpanded);
  const childrenId = useId();
  const kept = t('ui.meaning-list.app.kept', { name: { kind: 'text', value: app.name } });

  return (
    <>
      <div aria-label={app.name} role="list">
        <TreeRow
          controls={childrenId}
          expanded={expanded}
          onExpandedChange={setExpanded}
          {...(app.installed ? {} : { subtitle: t('ui.meaning-list.not-installed') })}
          title={app.name}
          toggleLabel={t('ui.meaning-list.app.toggle', { name: { kind: 'text', value: app.name } })}
          trailing={
            <span className={styles.size}>
              <SizeText {...honestSize(sizes)} />
            </span>
          }
        />
      </div>
      <div aria-label={kept} className={styles.children} hidden={!expanded} id={childrenId} role="group">
        <div role="list">
          {categories.map((share) => (
            <CategoryRow
              key={share.category}
              onOpen={() => {
                onOpen(share.category);
              }}
              share={share}
            />
          ))}
          {data.map((place) => (
            <PlaceRow
              key={`${String(place.node.state)}-${String(place.node.index)}`}
              onOpen={() => {
                onOpenPlace(place.node);
              }}
              place={place}
            />
          ))}
        </div>
      </div>
    </>
  );
};
