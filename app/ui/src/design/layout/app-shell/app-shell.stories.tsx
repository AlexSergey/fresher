import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { LayoutGrid, ListChecks, RotateCw, Settings, Trash2 } from '../../icons/icons';
import { Badge } from '../../primitives/badge/badge';
import { Button } from '../../primitives/button/button';
import { IconButton } from '../../primitives/icon-button/icon-button';
import { NavItem } from '../nav-item/nav-item';
import { PageHeader } from '../page-header/page-header';
import { ScrollArea } from '../scroll-area/scroll-area';
import { NavSection } from '../sidebar/nav-section';
import { Sidebar } from '../sidebar/sidebar';
import { Surface } from '../surface/surface';
import { Toolbar } from '../toolbar/toolbar';
import { AppShell } from './app-shell';
import styles from './app-shell.stories.module.css';

const noop = (): void => undefined;

const CARDS = ['Xcode caches', 'npm cache', 'Docker images', 'Old iOS backups'] as const;

// The window with the fallback sidebar and a screen, as a test or the browser shows it.
const Window = (): JSX.Element => (
  <div className={styles.stage}>
    <AppShell
      sidebar={
        <Sidebar footer={<NavItem icon={Settings} label="Settings" onClick={noop} />} label="Screens">
          <NavSection>
            <NavItem active icon={LayoutGrid} label="Overview" onClick={noop} />
            <NavItem icon={ListChecks} label="Review" onClick={noop} trailing={<Badge count={12} label="12 items" />} />
            <NavItem icon={Trash2} label="Trash" onClick={noop} trailing="1.2 GB" />
          </NavSection>
        </Sidebar>
      }
    >
      <ScrollArea label="Overview" screenKey="gallery-app-shell">
        <Toolbar actions={<IconButton icon={RotateCw} label="Scan again" />} />
        <PageHeader
          actions={<Button variant="primary">Review</Button>}
          subtitle="Last scanned 5 minutes ago"
          title="Overview"
        />
        <div className={styles.cards}>
          {CARDS.map((name) => (
            <Surface elevation={1} key={name}>
              {name}
            </Surface>
          ))}
        </div>
      </ScrollArea>
    </AppShell>
  </div>
);

export const stories = storiesOf('AppShell', {
  narrow: { render: Window, width: 900 },
  wide: { render: Window, width: 1052 },
});
