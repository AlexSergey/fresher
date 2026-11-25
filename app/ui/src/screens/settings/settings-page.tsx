import { type JSX, useId } from 'react';

import type { ModuleInfo, SettingDescriptor, SettingsView } from '../../ipc/bindings.generated';
import type { InGroup } from '../../product/module-groups';
import type { IgnoredTab, SettingsPage as Page } from '../../state/navigation';
import type { ModuleSettings } from './settings-groups';
import type { Settings } from './use-settings';

import { t, tText } from '../../i18n';
import { PermissionsGroup } from '../permission/permissions-group';
import { AboutGroup } from './about-group';
import { FileNotice, isReadOnly } from './file-notice';
import { IgnoredGroup } from './ignored-group';
import { OwnDataGroup } from './own-data-group';
import { RescanNotice } from './rescan-notice';
import styles from './settings-page.module.css';
import { SettingsPanel } from './settings-panel';

type CoreKind = 'about' | 'general' | 'ignored' | 'permissions' | 'scan';

const TITLES = {
  about: 'ui.screen.settings.group.about',
  general: 'ui.screen.settings.group.general',
  ignored: 'ui.screen.settings.group.ignored',
  permissions: 'ui.screen.settings.group.permissions',
  scan: 'ui.screen.settings.group.scan',
} as const satisfies Record<CoreKind, string>;

type SettingsPageProps = {
  readonly general: readonly SettingDescriptor[];
  readonly groups: readonly InGroup<ModuleSettings>[];
  readonly modules: readonly ModuleInfo[];
  // The Ignored tab to open on, when something names one.
  readonly opensOn?: IgnoredTab;
  readonly page: Page;
  readonly scan: readonly SettingDescriptor[];
  readonly settings: Settings;
  // `null` until the core has answered: the page shows its heading and nothing else.
  readonly view: null | SettingsView;
};

// The page of one section: why the settings file cannot be changed and "Scan again to apply it" at its top, then a core
// section's heading and part, or a module's group, name, line, rows and on/off switch.
export const SettingsPage = ({
  general,
  groups,
  modules,
  opensOn,
  page,
  scan,
  settings,
  view,
}: SettingsPageProps): JSX.Element => {
  const headingId = useId();
  const ownDataId = useId();
  const readOnly = view !== null && isReadOnly(view.file);
  const part = (kind: CoreKind, loaded: SettingsView): JSX.Element => {
    switch (kind) {
      case 'about':
        return <AboutGroup headingId={headingId} />;
      case 'general':
        return (
          <>
            <SettingsPanel descriptors={general} settings={settings} view={loaded} />
            <OwnDataGroup headingId={ownDataId} readOnly={readOnly} />
          </>
        );
      case 'ignored':
        return (
          <IgnoredGroup
            headingId={headingId}
            modules={modules}
            readOnly={readOnly}
            view={loaded}
            {...(opensOn === undefined ? {} : { opensOn })}
          />
        );
      case 'permissions':
        return <PermissionsGroup headingId={headingId} settings={settings} view={loaded} />;
      case 'scan':
        return <SettingsPanel descriptors={scan} settings={settings} view={loaded} />;
    }
  };
  const notices =
    view === null ? null : (
      <>
        <FileNotice file={view.file} />
        <RescanNotice />
      </>
    );

  if (page.kind === 'module') {
    const found = groups.find(({ items }) => items.some(({ module }) => module === page.module));
    const entry = found?.items.find(({ module }) => module === page.module);

    if (found !== undefined && entry !== undefined) {
      const name = tText({ args: {}, key: `${page.module}.enabled.label` });
      const descriptors = entry.switch === undefined ? entry.rows : [...entry.rows, entry.switch];

      return (
        <div className={styles.page}>
          {notices}
          <header className={styles.header}>
            <span className={styles.groupName}>{tText({ args: {}, key: `core.group.${found.group}.name` })}</span>
            <h1 className={styles.title}>{name}</h1>
            <p className={styles.about}>{tText({ args: {}, key: `${page.module}.enabled.description` })}</p>
          </header>
          {view === null ? null : (
            <SettingsPanel
              descriptors={descriptors}
              settings={settings}
              textsOf={(descriptor) =>
                descriptor === entry.switch
                  ? {
                      description: t('ui.screen.settings.module-switch.description'),
                      label: t('ui.screen.settings.module-switch', { module: { kind: 'text', value: name } }),
                    }
                  : undefined
              }
              view={view}
            />
          )}
        </div>
      );
    }
  }

  const kind: CoreKind = page.kind === 'module' ? 'general' : page.kind;

  return (
    <div className={styles.page}>
      {notices}
      <h1 className={styles.title} id={headingId}>
        {t(TITLES[kind])}
      </h1>
      {view === null ? null : part(kind, view)}
    </div>
  );
};
