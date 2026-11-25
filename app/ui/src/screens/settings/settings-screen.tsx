import { type JSX, useEffect, useRef, useState } from 'react';

import type { SettingKey } from '../../ipc/bindings.generated';

import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { type SettingsPage as Page, useNavigation } from '../../state/navigation';
import { useModules } from '../use-modules';
import { focusSetting } from './focus-setting';
import { settingsGroups } from './settings-groups';
import { SettingsPage } from './settings-page';
import styles from './settings-screen.module.css';
import { type SearchHit, searchSettings, settingsEntries } from './settings-search';
import { SearchResults, SettingsSearch } from './settings-search-view';
import { SettingsSidebar } from './settings-sidebar';
import { useSettings } from './use-settings';

// "What can I change?": a sidebar of the sections (General, Scan, Ignored, Permissions, About, then what the scan looks
// for: each module group with its modules) beside the page of the current one, or the results of the search while it
// holds text.
export const SettingsScreen = (): JSX.Element => {
  const settings = useSettings();
  const modules = useModules();
  // Read once: the page, Ignored tab and setting to open on are only the starting ones, so the screen needs no
  // subscription.
  const screen = useNavigation().getScreen();
  const opensOn = screen.kind === 'settings' ? screen.tab : undefined;
  const [page, setPage] = useState<Page>(
    screen.kind === 'settings'
      ? (screen.page ?? { kind: opensOn === undefined ? 'general' : 'ignored' })
      : { kind: 'general' },
  );
  const [query, setQuery] = useState('');
  // The setting to focus once its page shows: asked by the screen that opened Settings or by a click on a hit.
  const focusRef = useRef<null | SettingKey>(screen.kind === 'settings' ? (screen.focus ?? null) : null);
  const { view } = settings;
  const groups = settingsGroups(view?.settings ?? [], modules ?? []);
  const searching = query.trim() !== '';
  const hits = searching ? searchSettings(query, settingsEntries(groups, view?.values ?? {})) : null;

  const loaded = view !== null;

  useEffect(() => {
    if (focusRef.current !== null && loaded && !searching) {
      focusSetting(focusRef.current);
      focusRef.current = null;
    }
  }, [loaded, searching]);

  const open = (hit: SearchHit): void => {
    setQuery('');
    setPage(hit.entry.page);
    focusRef.current = hit.entry.key;
  };

  return (
    <>
      <Toolbar />
      <div className={styles.body}>
        <SettingsSidebar
          groups={groups.modules}
          hits={hits}
          onSelect={setPage}
          page={page}
          search={
            <SettingsSearch
              onEnter={() => {
                const first = hits?.[0];

                if (first !== undefined) {
                  open(first);
                }
              }}
              onQuery={setQuery}
              query={query}
            />
          }
        />
        {hits === null ? (
          <SettingsPage
            general={groups.general}
            groups={groups.modules}
            modules={modules ?? []}
            page={page}
            scan={groups.scan}
            settings={settings}
            view={view}
            {...(opensOn === undefined ? {} : { opensOn })}
          />
        ) : (
          <SearchResults hits={hits} onOpen={open} query={query} />
        )}
      </div>
    </>
  );
};
