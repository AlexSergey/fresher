import type { JSX } from 'react';

import type { Screen } from '../state/navigation';

import { LogoMark } from '../design/brand/logo-mark';
import { AppWindow, ArrowDownWideNarrow, History, LayoutGrid, ListChecks, Settings } from '../design/icons/icons';
import { NavItem } from '../design/layout/nav-item/nav-item';
import { NavSection } from '../design/layout/sidebar/nav-section';
import { Sidebar } from '../design/layout/sidebar/sidebar';
import { Badge } from '../design/primitives/badge/badge';
import { t } from '../i18n';
import { summarize } from '../state/selection-summary';
import { useCore } from '../state/store';
import styles from './app-frame.module.css';
import { useAppInfo } from './use-app-info';

export type FrameSidebarProps = {
  readonly onNavigate: (screen: Screen) => void;
  readonly screen: Screen;
};

// The sidebar: the app's name, Scan (current on a category's details too), Largest (only while the core offers High),
// Apps (current on an app's page too), Review with the selection's count (or the categories to check), Cleanups, and
// Settings at the bottom.
export const FrameSidebar = ({ onNavigate, screen }: FrameSidebarProps): JSX.Element => {
  const overview = useCore((state) => state.overview);
  const selection = useCore((state) => state.selection);
  const summary = summarize(selection, overview?.totals ?? []);
  // The badge counts what goes; with nothing going, the categories to check in Review.
  const badge =
    summary.count > 0
      ? { count: summary.count, key: 'ui.screen.frame.review-count' as const }
      : { count: summary.toCheck.categories, key: 'ui.screen.frame.review-to-check' as const };
  // Largest only while the build shows it.
  const largest = useAppInfo()?.flags.largest === 'shown';

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
      <p className={styles.name}>
        <LogoMark />
        {t('ui.app.title')}
      </p>
      <NavSection>
        <NavItem
          active={screen.kind === 'scan' || screen.kind === 'category' || screen.kind === 'insight'}
          icon={LayoutGrid}
          label={t('ui.screen.frame.nav.scan')}
          onClick={() => {
            onNavigate({ kind: 'scan' });
          }}
        />
        {largest ? (
          <NavItem
            active={screen.kind === 'largest'}
            icon={ArrowDownWideNarrow}
            label={t('ui.screen.frame.nav.largest')}
            onClick={() => {
              onNavigate({ kind: 'largest' });
            }}
          />
        ) : null}
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
            badge.count === 0 ? undefined : (
              <Badge
                count={badge.count}
                label={t(badge.key, { count: { kind: 'count', value: badge.count } })}
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
