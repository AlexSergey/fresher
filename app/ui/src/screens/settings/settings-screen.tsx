import { type JSX, useId } from 'react';

import type { SettingsView } from '../../ipc/bindings.generated';
import type { Settings } from './use-settings';

import { PageHeader } from '../../design/layout/page-header/page-header';
import { Surface } from '../../design/layout/surface/surface';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { t, tText } from '../../i18n';
import { SettingsRow } from '../../product/settings-row/settings-row';
import { useModules } from '../use-modules';
import { FileNotice, isReadOnly } from './file-notice';
import { IgnoredGroup } from './ignored-group';
import { OwnDataGroup } from './own-data-group';
import { RescanNotice } from './rescan-notice';
import { type SettingsGroup, settingsGroups } from './settings-groups';
import styles from './settings-screen.module.css';
import { useAppVersion } from './use-app-version';
import { useSettings } from './use-settings';

// "What can I change?": the groups in order (General, Scan, each module, Ignored, Permissions, Fresher's own data,
// About); why the settings file cannot be changed, when it cannot; "Rescan to apply" when a change waits for a scan.
export const SettingsScreen = (): JSX.Element => {
  const settings = useSettings();
  const modules = useModules();
  const { view } = settings;
  const ignoredId = useId();
  const ownDataId = useId();
  const groups = view === null || modules === null ? null : settingsGroups(view.settings, modules);
  const readOnly = view !== null && isReadOnly(view.file);
  const rows = (group: SettingsGroup, inView: SettingsView): JSX.Element => (
    <Group group={group} key={group.kind === 'core' ? group.group : group.module} settings={settings} view={inView} />
  );

  return (
    <>
      <Toolbar />
      <PageHeader title={t('ui.screen.frame.nav.settings')} />
      <div className={styles.content}>
        {view === null ? null : <FileNotice file={view.file} />}
        <RescanNotice />
        {view === null || groups === null ? null : (
          <>
            {groups.before.map((group) => rows(group, view))}
            <IgnoredGroup headingId={ignoredId} readOnly={readOnly} view={view} />
            {groups.after.map((group) => rows(group, view))}
          </>
        )}
        <OwnDataGroup headingId={ownDataId} readOnly={readOnly} />
        <AboutGroup />
      </div>
    </>
  );
};

// The heading of each of the core's groups of rows.
const GROUP_HEADINGS = {
  general: 'ui.screen.settings.group.general',
  permissions: 'ui.screen.settings.group.permissions',
  scan: 'ui.screen.settings.group.scan',
} as const;

type GroupProps = {
  readonly group: SettingsGroup;
  readonly settings: Settings;
  readonly view: SettingsView;
};

// One group of rows under its heading: the core group's name, or the module's name for a module's.
const Group = ({ group, settings, view }: GroupProps): JSX.Element => {
  const headingId = useId();
  const readOnly = isReadOnly(view.file);

  return (
    <section aria-labelledby={headingId} className={styles.group}>
      <h2 className={styles.heading} id={headingId}>
        {group.kind === 'core'
          ? t(GROUP_HEADINGS[group.group])
          : tText({ args: {}, key: `${group.module}.enabled.label` })}
      </h2>
      <Surface elevation={1} padding="none">
        <div className={styles.rows}>
          {group.descriptors.map((descriptor) => {
            const refusal = settings.refusals[descriptor.key];

            return (
              <SettingsRow
                descriptor={descriptor}
                key={descriptor.key}
                onChange={(value) => {
                  settings.change(descriptor.key, value);
                }}
                problem={view.problems.find((problem) => problem.key === descriptor.key) ?? null}
                readOnly={readOnly}
                {...(refusal === undefined ? {} : { refusal })}
                {...(descriptor.kind.kind === 'folders'
                  ? {
                      onAddFolder: (): void => {
                        settings.addFolder(descriptor.key);
                      },
                    }
                  : {})}
                value={view.values[descriptor.key] ?? descriptor.default}
              />
            );
          })}
        </div>
      </Surface>
    </section>
  );
};

// The app's version and its licence. The licences of what it is built from come before the first release.
const AboutGroup = (): JSX.Element => {
  const headingId = useId();
  const version = useAppVersion();

  return (
    <section aria-labelledby={headingId} className={styles.group}>
      <h2 className={styles.heading} id={headingId}>
        {t('ui.screen.settings.group.about')}
      </h2>
      <Surface elevation={1} padding="none">
        <div className={styles.rows}>
          {version === null ? null : (
            <p className={styles.line}>{t('ui.app.version', { version: { kind: 'text', value: version } })}</p>
          )}
          <p className={styles.line}>{t('ui.screen.settings.about.licence')}</p>
        </div>
      </Surface>
    </section>
  );
};
