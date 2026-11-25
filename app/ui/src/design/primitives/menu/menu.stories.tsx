import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Settings } from '../../icons/icons';
import { Menu, type MenuItem, SEPARATOR } from './menu';
import styles from './menu.stories.module.css';

const noop = (): void => undefined;

// The row's menu: Finder, a separator, the item and its category, a separator, a far-reaching choice (quiet, as the last one).
const ITEMS: readonly MenuItem[] = [
  { key: 'reveal', label: 'Show in Finder', onSelect: noop },
  SEPARATOR,
  {
    key: 'item',
    label: 'Ignore this item',
    onSelect: noop,
  },
  { key: 'category', label: 'Ignore all App caches', onSelect: noop },
  SEPARATOR,
  {
    key: 'quiet',
    label: 'A far-reaching choice…',
    onSelect: noop,
    quiet: true,
  },
];

const stage = (props: { readonly bottom?: boolean; readonly defaultOpen?: boolean }): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <div className={`${styles.stage ?? ''} ${props.bottom === true ? (styles.bottom ?? '') : ''}`}>
        <Menu defaultOpen={props.defaultOpen} icon={Settings} items={ITEMS} label="Options for Caches" />
      </div>
    );
  };

// The menu is drawn on <body>; the stage leaves it room inside the photographed frame.
export const stories = storiesOf('Menu', {
  closed: { render: stage({}), width: 320 },
  open: { render: stage({ defaultOpen: true }), width: 320 },
  'open-near-the-bottom': { render: stage({ bottom: true, defaultOpen: true }), width: 320 },
});
