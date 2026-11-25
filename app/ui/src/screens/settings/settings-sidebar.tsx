import { type JSX, type ReactNode, useState } from 'react';

import type { InGroup } from '../../product/module-groups';
import type { SettingsPage } from '../../state/navigation';
import type { ModuleSettings } from './settings-groups';
import type { SearchHit } from './settings-search';

import { Icon } from '../../design/icons/icon';
import { ChevronDown, ChevronRight } from '../../design/icons/icons';
import { t, tText } from '../../i18n';
import { hitsOn } from './settings-search';
import styles from './settings-sidebar.module.css';

const CORE_PAGES = [
  { kind: 'general', label: 'ui.screen.settings.group.general' },
  { kind: 'scan', label: 'ui.screen.settings.group.scan' },
  { kind: 'ignored', label: 'ui.screen.settings.group.ignored' },
  { kind: 'permissions', label: 'ui.screen.settings.group.permissions' },
  { kind: 'about', label: 'ui.screen.settings.group.about' },
] as const;

type SettingsSidebarProps = {
  readonly groups: readonly InGroup<ModuleSettings>[];
  // What the search found while it holds text: the tree dims what has no hit and counts the modules' hits.
  readonly hits: null | readonly SearchHit[];
  readonly onSelect: (page: SettingsPage) => void;
  readonly page: SettingsPage;
  // The search field, above the sections.
  readonly search: ReactNode;
};

// The sections of Settings, then "What it looks for": each module group that has a module with settings as a branch
// that opens on a click (always open while it holds the current page), its modules under it.
export const SettingsSidebar = ({ groups, hits, onSelect, page, search }: SettingsSidebarProps): JSX.Element => {
  const [toggled, setToggled] = useState<Readonly<Record<string, boolean>>>({});

  return (
    <nav aria-label={t('ui.screen.settings.sections')} className={styles.sidebar}>
      <div className={styles.list}>
        {search}
        {CORE_PAGES.map(({ kind, label }) => (
          <button
            aria-current={page.kind === kind ? 'page' : undefined}
            className={styles.item}
            data-dimmed={hits !== null && hitsOn(hits, { kind }) === 0 ? '' : undefined}
            key={kind}
            onClick={() => {
              onSelect({ kind });
            }}
            type="button"
          >
            {t(label)}
          </button>
        ))}
        <p className={styles.looksFor}>{t('ui.screen.settings.looks-for')}</p>
        {groups.map(({ group, items }) => {
          const holdsCurrent = items.some(({ module }) => page.kind === 'module' && page.module === module);
          const found =
            hits === null ? 0 : items.reduce((sum, { module }) => sum + hitsOn(hits, { kind: 'module', module }), 0);
          const open = holdsCurrent || toggled[group] === true || found > 0;

          return (
            <div className={styles.branch} key={group}>
              <button
                aria-expanded={open}
                className={`${styles.item} ${styles.group}`}
                data-dimmed={hits !== null && found === 0 ? '' : undefined}
                onClick={() => {
                  setToggled({ ...toggled, [group]: !open });
                }}
                type="button"
              >
                <Icon icon={open ? ChevronDown : ChevronRight} />
                {tText({ args: {}, key: `core.group.${group}.name` })}
              </button>
              {open
                ? items.map(({ module }) => {
                    const inModule = hits === null ? 0 : hitsOn(hits, { kind: 'module', module });

                    return (
                      <button
                        aria-current={page.kind === 'module' && page.module === module ? 'page' : undefined}
                        className={`${styles.item} ${styles.module}`}
                        data-dimmed={hits !== null && inModule === 0 ? '' : undefined}
                        key={module}
                        onClick={() => {
                          onSelect({ kind: 'module', module });
                        }}
                        type="button"
                      >
                        {tText({ args: {}, key: `${module}.enabled.label` })}
                        {inModule === 0 ? null : <span className={styles.count}>{inModule}</span>}
                      </button>
                    );
                  })
                : null}
            </div>
          );
        })}
      </div>
    </nav>
  );
};
