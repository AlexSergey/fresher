import type { JSX } from 'react';

import type { NodeId, NodeInfo } from '../../ipc/bindings.generated';
import type { Crumb } from '../../product/place-list/breadcrumb';

import { t } from '../../i18n';
import { PlaceList } from '../../product/place-list/place-list';
import { RootList } from '../../product/place-list/root-list';
import { type PlaceAt, placeTop, useNavigation } from '../../state/navigation';
import { useStoragePlace, useStorageRoots } from '../../state/storage';
import { useCore } from '../../state/store';
import styles from './overview-screen.module.css';

const sameNode = (a: NodeId | null, b: NodeId | null): boolean =>
  a !== null && b !== null && a.state === b.state && a.index === b.index;

const keyOf = ({ id }: NodeInfo): string => `${String(id.state)}-${String(id.index)}`;

// The overview's view by place, at the folder the navigation keeps: the volume's first root at first, or its scanned
// folders as rows when it holds several. Opening a folder keeps the way down to it, so a step of the breadcrumb goes
// back up; a node of an older scan reopens the volume's first root, with no way down kept.
export const ByPlace = ({ at }: { readonly at: PlaceAt }): JSX.Element => {
  const navigation = useNavigation();
  const volume = useCore((state) => state.storageMap?.volumes[at.volume] ?? null);
  const firstRoot = volume?.roots[0] ?? null;
  const severalRoots = (volume?.roots.length ?? 0) > 1;
  const roots = useStorageRoots(at.node === null ? at.volume : null);
  // While the roots are asked, nothing is; when the core refused them, the first root's view says why.
  const asked = at.node ?? (roots === null ? firstRoot : null);
  const place = useStoragePlace(asked, at.volume, at.descent);

  const show = (next: Omit<PlaceAt, 'kind' | 'volume'>): void => {
    navigation.showOverviewView({ ...next, kind: 'place', volume: at.volume });
  };

  if (at.node === null && Array.isArray(roots)) {
    return (
      <RootList
        onOpen={(root) => {
          show({ descent: 'through', node: root.id, trail: [] });
        }}
        roots={roots}
      />
    );
  }

  if (asked === null) {
    return roots === undefined ? (
      <div aria-busy="true" />
    ) : (
      <p className={styles.intro}>{t('ui.place-list.waiting')}</p>
    );
  }

  // The way down: the folders the user came through, then the folder open with the descent the core went through.
  // Until the view has taken up the node asked, its path is the previous folder's, so no step is shown.
  const current = sameNode(place.asked, asked);
  const steps = current ? [...(sameNode(place.node, asked) ? at.trail : []), ...place.path] : [];
  const top: readonly Crumb[] = severalRoots ? [{ key: 'roots', name: t('ui.place-list.roots') }] : [];
  const crumbs = current ? [...top, ...steps.map((step) => ({ key: keyOf(step), name: step.name }))] : [];

  return (
    <PlaceList
      crumbs={crumbs}
      failure={place.failure}
      folder={place.path.at(-1) ?? null}
      onOpen={(node) => {
        show({ descent: 'through', node: node.id, trail: steps });
      }}
      onRangeChange={place.onRangeChange}
      onStep={(index) => {
        const step = steps[index - top.length];

        if (step === undefined) {
          navigation.showOverviewView(placeTop(at.volume));
        } else {
          show({ descent: 'stay', node: step.id, trail: steps.slice(0, index - top.length) });
        }
      }}
      rowAt={place.rowAt}
      total={place.total}
    />
  );
};
