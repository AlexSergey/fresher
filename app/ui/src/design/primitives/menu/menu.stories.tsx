import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Settings } from '../../icons/icons';
import { Menu, type MenuItem } from './menu';
import styles from './menu.stories.module.css';

const noop = (): void => undefined;

const ITEMS: readonly MenuItem[] = [
  {
    description: 'Hides it from the results and never removes it.',
    key: 'category',
    label: 'Ignore this category',
    onSelect: noop,
  },
  { description: 'Hides every category of this module.', key: 'module', label: 'Ignore Caches', onSelect: noop },
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
