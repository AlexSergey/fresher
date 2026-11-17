import type { JSX } from 'react';

import type { ModuleInfo, Sizes } from '../ipc/bindings.generated';
import type { Screen } from '../state/navigation';

import { History, LayoutGrid, ListChecks, Package, Settings } from '../design/icons/icons';
import { NavItem } from '../design/layout/nav-item/nav-item';
import { NavSection } from '../design/layout/sidebar/nav-section';
import { Sidebar } from '../design/layout/sidebar/sidebar';
import { Badge } from '../design/primitives/badge/badge';
import { t, tText } from '../i18n';
import { modulesByGroup } from '../product/module-groups';
import { sizeLabel } from '../product/sizes';
import { summarize } from '../state/selection-summary';
import { useCore } from '../state/store';
import styles from './app-frame.module.css';
import { hiddenBy, isGroupIgnored } from './use-ignored';
import { moduleFound } from './use-modules';

export type FrameSidebarProps = {
  readonly modules: null | readonly ModuleInfo[];
  readonly onNavigate: (screen: Screen) => void;
  readonly screen: Screen;
};

const groupTitle = (group: string, ignored: boolean): string => {
  const name = tText({ args: {}, key: `core.group.${group}.name` });

  return ignored ? t('ui.screen.frame.group-ignored', { group: { kind: 'text', value: name } }) : name;
};

const trailingOf = (ignored: boolean, found: null | Sizes): string => {
  if (ignored) {
    return t('ui.screen.frame.module-ignored');
  }

  return found === null ? NOT_SCANNED : sizeLabel(found);
};

const NOT_SCANNED = sizeLabel({ allocated: 0, completeness: 'unknown', logical: 0, reclaimable: 0 });

// The sidebar: the app's name, Overview, one section per module group with an item per module and what it found
// ("Ignored" for a module the core hides, dimmed; the heading of an ignored group says so too), Review with the
// selection's count, Cleanups, and Settings at the bottom.
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
      {modulesByGroup(modules ?? []).map(({ group, items }) => (
        <NavSection key={group} title={groupTitle(group, isGroupIgnored(overview, modules ?? [], group))}>
          {items.map((module) => {
            const found = moduleFound(module, overview);
            const ignored = hiddenBy(overview, module.id) !== undefined;

            return (
              <NavItem
                active={screen.kind === 'module' && screen.module === module.id}
                dimmed={ignored}
                icon={Package}
                key={module.id}
                label={tText({ args: {}, key: `${module.id}.enabled.label` })}
                onClick={() => {
                  onNavigate({ kind: 'module', module: module.id });
                }}
                trailing={trailingOf(ignored, found)}
              />
            );
          })}
        </NavSection>
      ))}
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
