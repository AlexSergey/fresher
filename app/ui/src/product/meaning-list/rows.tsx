import type { JSX } from 'react';

import type { CategoryShare, MapPart, PlaceShare } from '../../ipc/bindings.generated';

import { ListRow } from '../../design/data/list-row/list-row';
import { SizeText } from '../../design/data/size-text/size-text';
import { ChevronRight } from '../../design/icons/icons';
import { IconButton } from '../../design/primitives/icon-button/icon-button';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import { partName } from '../storage-map/parts';
import styles from './meaning-list.module.css';

const textOf = (key: string): string => tText({ args: {}, key });

// A place's name: the folder's own, or the place the core names.
const placeName = ({ name }: PlaceShare): string =>
  name.kind === 'trash' ? textOf('core.storage.place.trash.name') : name.value;

const Size = ({ sizes }: { readonly sizes: CategoryShare['sizes'] }): JSX.Element => (
  <span className={styles.size}>
    <SizeText {...honestSize(sizes)} />
  </span>
);

// The way into a row: a category opens its module's page, a place the place view.
const Open = ({ label, onOpen }: { readonly label: string; readonly onOpen: () => void }): JSX.Element => (
  <IconButton icon={ChevronRight} label={label} onClick={onOpen} size="small" />
);

// A folder no module claims: its name (the Trash with its sentence), its size, and a way into the place view. Never a
// checkbox: Fresher suggests nothing here.
export const PlaceRow = ({
  onOpen,
  place,
}: {
  readonly onOpen: () => void;
  readonly place: PlaceShare;
}): JSX.Element => {
  const name = placeName(place);

  return (
    <ListRow
      {...(place.name.kind === 'trash' ? { subtitle: textOf('core.storage.place.trash.description') } : {})}
      title={name}
      trailing={
        <>
          <Size sizes={place.sizes} />
          <Open label={t('ui.meaning-list.open-place', { name: { kind: 'text', value: name } })} onOpen={onOpen} />
        </>
      }
    />
  );
};

// A category of an app: its name and size, opening its module's page.
export const CategoryRow = ({
  onOpen,
  share,
}: {
  readonly onOpen: () => void;
  readonly share: CategoryShare;
}): JSX.Element => {
  const name = textOf(`${share.category}.name`);

  return (
    <ListRow
      title={name}
      trailing={
        <>
          <Size sizes={share.sizes} />
          <Open label={t('ui.meaning-list.open', { name: { kind: 'text', value: name } })} onOpen={onOpen} />
        </>
      }
    />
  );
};

// One of the volume's lines: the system's parts, the remainder and Free, for information only.
export const PartRow = ({ part }: { readonly part: MapPart }): JSX.Element => (
  <ListRow title={partName(part.kind)} trailing={<Size sizes={part.sizes} />} />
);
