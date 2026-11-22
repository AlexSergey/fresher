import type { JSX } from 'react';

import type { Screen } from '../state/navigation';

import { AppWindow, ArrowDownWideNarrow, History, LayoutGrid, ListChecks, Settings } from '../design/icons/icons';
import { NavItem } from '../design/layout/nav-item/nav-item';
import { NavSection } from '../design/layout/sidebar/nav-section';
import { Sidebar } from '../design/layout/sidebar/sidebar';
import { Badge } from '../design/primitives/badge/badge';
import { t } from '../i18n';
import { summarize } from '../state/selection-summary';
import { useCore } from '../state/store';
import styles from './app-frame.module.css';

export type FrameSidebarProps = {
  readonly onNavigate: (screen: Screen) => void;
  readonly screen: Screen;
};

// The sidebar: the app's name, Scan (current on a category's details too), Largest, Apps (current on an app's page
// too), Review with the selection's count, Cleanups, and Settings at the bottom.
export const FrameSidebar = ({ onNavigate, screen }: FrameSidebarProps): JSX.Element => {
  const overview = useCore((state) => state.overview);
  const selection = useCore((state) => state.selection);
  const selected = summarize(selection, overview?.totals ?? []).count;

  return (
    <Sidebar
      footer={
        <NavItem
          active={screen.kind === 'settings'}
          icon={Settings}
          label={t('ui.screen.frame.nav.settings')}
          onClick={() => {
            onNavigate({ kind: 'settings' });
          }}
        />
      }
      label={t('ui.screen.frame.label')}
    >
      <p className={styles.name}>{t('ui.app.title')}</p>
      <NavSection>
        <NavItem
          active={screen.kind === 'scan' || screen.kind === 'category' || screen.kind === 'insight'}
          icon={LayoutGrid}
          label={t('ui.screen.frame.nav.scan')}
          onClick={() => {
            onNavigate({ kind: 'scan' });
          }}
        />
        <NavItem
          active={screen.kind === 'largest'}
          icon={ArrowDownWideNarrow}
          label={t('ui.screen.frame.nav.largest')}
          onClick={() => {
            onNavigate({ kind: 'largest' });
          }}
        />
        <NavItem
          active={screen.kind === 'apps' || screen.kind === 'app'}
          icon={AppWindow}
          label={t('ui.screen.frame.nav.apps')}
          onClick={() => {
            onNavigate({ kind: 'apps' });
          }}
        />
      </NavSection>
      <NavSection>
        <NavItem
          active={screen.kind === 'review'}
          icon={ListChecks}
          label={t('ui.screen.frame.nav.review')}
          onClick={() => {
            onNavigate({ kind: 'review' });
          }}
          trailing={
            selected === 0 ? undefined : (
              <Badge
                count={selected}
                label={t('ui.screen.frame.review-count', { count: { kind: 'count', value: selected } })}
                tone="accent"
              />
            )
          }
        />
        <NavItem
          active={screen.kind === 'cleanups' || screen.kind === 'cleanup'}
          icon={History}
          label={t('ui.screen.frame.nav.cleanups')}
          onClick={() => {
            onNavigate({ kind: 'cleanups' });
          }}
        />
      </NavSection>
    </Sidebar>
  );
};
