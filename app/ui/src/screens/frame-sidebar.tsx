import type { JSX } from 'react';

import type { ModuleInfo } from '../ipc/bindings.generated';
import type { Screen } from '../state/navigation';

import { LayoutGrid, ListChecks, Package, Settings } from '../design/icons/icons';
import { NavItem } from '../design/layout/nav-item/nav-item';
import { NavSection } from '../design/layout/sidebar/nav-section';
import { Sidebar } from '../design/layout/sidebar/sidebar';
import { Badge } from '../design/primitives/badge/badge';
import { t, tText } from '../i18n';
import { sizeLabel } from '../product/sizes';
import { summarize } from '../state/selection-summary';
import { useCore } from '../state/store';
import styles from './app-frame.module.css';
import { moduleFound } from './use-modules';

export type FrameSidebarProps = {
  readonly modules: null | readonly ModuleInfo[];
  readonly onNavigate: (screen: Screen) => void;
  readonly screen: Screen;
};

const NOT_SCANNED = sizeLabel({ allocated: 0, completeness: 'unknown', logical: 0 });

// The sidebar: the app's name, Overview, one item per module with what it found, Review with the selection's count,
// and Settings at the bottom.
export const FrameSidebar = ({ modules, onNavigate, screen }: FrameSidebarProps): JSX.Element => {
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
          active={screen.kind === 'overview'}
          icon={LayoutGrid}
          label={t('ui.screen.frame.nav.overview')}
          onClick={() => {
            onNavigate({ kind: 'overview' });
          }}
        />
      </NavSection>
      {modules === null || modules.length === 0 ? null : (
        <NavSection title={t('ui.screen.frame.modules')}>
          {modules.map((module) => {
            const found = moduleFound(module, overview);

            return (
              <NavItem
                active={screen.kind === 'module' && screen.module === module.id}
                icon={Package}
                key={module.id}
                label={tText({ args: {}, key: `${module.id}.enabled.label` })}
                onClick={() => {
                  onNavigate({ kind: 'module', module: module.id });
                }}
                trailing={found === null ? NOT_SCANNED : sizeLabel(found)}
              />
            );
          })}
        </NavSection>
      )}
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
      </NavSection>
    </Sidebar>
  );
};
