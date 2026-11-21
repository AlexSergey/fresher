import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { LayoutGrid, ListChecks, Search, Settings, Trash2 } from '../../icons/icons';
import { Badge } from '../../primitives/badge/badge';
import { NavItem } from '../nav-item/nav-item';
import { NavSection } from './nav-section';
import { Sidebar } from './sidebar';
import styles from './sidebar.stories.module.css';

const noop = (): void => undefined;

const WithSections = (): JSX.Element => (
  <div className={styles.stage}>
    <Sidebar footer={<NavItem icon={Settings} label="Settings" onClick={noop} />} label="Screens">
      <NavSection>
        <NavItem active icon={LayoutGrid} label="Scan" onClick={noop} />
        <NavItem icon={ListChecks} label="Review" onClick={noop} trailing={<Badge count={12} label="12 items" />} />
      </NavSection>
      <NavSection title="Modules">
        <NavItem icon={Search} label="Developer caches" onClick={noop} trailing="18.4 GB" />
        <NavItem icon={Trash2} label="Trash" onClick={noop} trailing="1.2 GB" />
      </NavSection>
    </Sidebar>
  </div>
);

// A frame 360 px high with five sections: the sections scroll under the strip, the footer after them.
const Short = (): JSX.Element => (
  <div className={`${styles.stage} ${styles.short}`}>
    <Sidebar footer={<NavItem icon={Settings} label="Settings" onClick={noop} />} label="Screens">
      {['Library', 'Modules', 'Tools', 'History', 'Help'].map((title) => (
        <NavSection key={title} title={title}>
          <NavItem icon={Search} label={`${title} first`} onClick={noop} />
          <NavItem icon={Trash2} label={`${title} second`} onClick={noop} />
        </NavSection>
      ))}
    </Sidebar>
  </div>
);

export const stories = storiesOf('Sidebar', {
  short: { render: Short, width: 232 },
  'with-sections-and-a-count': { render: WithSections, width: 232 },
});
