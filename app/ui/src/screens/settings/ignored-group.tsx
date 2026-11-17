import { type JSX, useState } from 'react';

import type { ModuleInfo, SettingsView } from '../../ipc/bindings.generated';

import { Tabs } from '../../design/layout/tabs/tabs';
import { t } from '../../i18n';
import { type IgnoredTab } from '../../state/navigation';
import { IgnoredGroups } from './ignored-groups';
import { IgnoredItems } from './ignored-items';
import { IgnoredModules } from './ignored-modules';
import { countsOf, firstWithEntry } from './ignored-tab';
import styles from './settings-screen.module.css';

type IgnoredGroupProps = {
  readonly headingId: string;
  readonly modules: readonly ModuleInfo[];
  // The tab to open on, when something names one; else the first with an entry.
  readonly opensOn?: IgnoredTab;
  readonly readOnly: boolean;
  readonly view: SettingsView;
};

// Everything the user ignores, in three tabs: groups, modules, and kinds of items and paths. Each tab's label carries
// its count.
export const IgnoredGroup = ({ headingId, modules, opensOn, readOnly, view }: IgnoredGroupProps): JSX.Element => {
  const groups = listOf(view, 'core.ignore.groups');
  const ignoredModules = listOf(view, 'core.ignore.modules');
  const categories = listOf(view, 'core.ignore.categories');
  const paths = listOf(view, 'core.ignore.paths');
  const counts = countsOf({
    categories: categories.length,
    groups: groups.length,
    modules: ignoredModules.length,
    paths: paths.length,
  });
  const [tab, setTab] = useState<IgnoredTab>(() => opensOn ?? firstWithEntry(counts));

  return (
    <section aria-labelledby={headingId} className={styles.group}>
      <h2 className={styles.heading} id={headingId}>
        {t('ui.screen.settings.group.ignored')}
      </h2>
      <p className={styles.groupText}>{t('ui.screen.settings.ignored.text')}</p>
      <Tabs
        label={t('ui.screen.settings.group.ignored')}
        onChange={setTab}
        tabs={[
          { count: counts.groups, label: t('ui.screen.settings.ignored.tab.groups'), value: 'groups' },
          { count: counts.modules, label: t('ui.screen.settings.ignored.tab.modules'), value: 'modules' },
          { count: counts.items, label: t('ui.screen.settings.ignored.tab.items'), value: 'items' },
        ]}
        value={tab}
      >
        <div className={styles.group}>
          {tab === 'groups' ? <IgnoredGroups ignored={groups} modules={modules} readOnly={readOnly} /> : null}
          {tab === 'modules' ? (
            <IgnoredModules
              ignoredGroups={groups}
              ignoredModules={ignoredModules}
              modules={modules}
              readOnly={readOnly}
            />
          ) : null}
          {tab === 'items' ? <IgnoredItems categories={categories} paths={paths} readOnly={readOnly} /> : null}
        </div>
      </Tabs>
    </section>
  );
};

// The entries of one ignore list, as the core keeps them.
const listOf = (view: SettingsView, key: string): readonly string[] => {
  const value = view.values[key];

  return value?.kind === 'ids' || value?.kind === 'paths' ? value.value : [];
};
