import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { LayoutGrid, ListChecks, Trash2 } from '../../icons/icons';
import { Badge } from '../../primitives/badge/badge';
import { NavSection } from '../sidebar/nav-section';
import { Sidebar } from '../sidebar/sidebar';
import { NavItem, type NavItemProps } from './nav-item';
import styles from './nav-item.stories.module.css';

type State = Pick<NavItemProps, 'active' | 'data-focus' | 'data-hover'>;

const noop = (): void => undefined;

// The middle item of a short sidebar in the given state: the active one shows the sidebar's indicator.
const middle = (state: State): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <div className={styles.stage}>
        <Sidebar label="Screens">
          <NavSection>
            <NavItem icon={LayoutGrid} label="Overview" onClick={noop} />
            <NavItem
              icon={ListChecks}
              label="Review"
              onClick={noop}
              trailing={<Badge count={12} label="12 items" />}
              {...state}
            />
            <NavItem icon={Trash2} label="Trash" onClick={noop} trailing="1.2 GB" />
          </NavSection>
        </Sidebar>
      </div>
    );
  };

export const stories = storiesOf('NavItem', {
  active: { render: middle({ active: true }), surface: 'raised', width: 232 },
  focus: { render: middle({ 'data-focus': true }), surface: 'raised', width: 232 },
  hover: { render: middle({ 'data-hover': true }), surface: 'raised', width: 232 },
  rest: { render: middle({}), surface: 'raised', width: 232 },
});
