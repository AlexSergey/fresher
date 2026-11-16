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
import { type SettingsGroup, settingsGroups } from './settings-groups';
import styles from './settings-screen.module.css';
import { useAppVersion } from './use-app-version';
import { useSettings } from './use-settings';

// "What can I change?": every setting the core describes, the core's first, then each module's under its name; why
// the settings file cannot be changed, when it cannot; the app's version at the bottom.
export const SettingsScreen = (): JSX.Element => {
  const settings = useSettings();
  const modules = useModules();
  const version = useAppVersion();
  const { view } = settings;

  return (
    <>
      <Toolbar />
      <PageHeader title={t('ui.screen.frame.nav.settings')} />
      <div className={styles.content}>
        {view === null ? null : <FileNotice file={view.file} />}
        {view === null || modules === null
          ? null
          : settingsGroups(view.settings, modules).map((group) => (
              <Group group={group} key={group.module ?? ''} settings={settings} view={view} />
            ))}
      </div>
      {version === null ? null : (
        <footer className={styles.footer}>{t('ui.app.version', { version: { kind: 'text', value: version } })}</footer>
      )}
    </>
  );
};

type GroupProps = {
  readonly group: SettingsGroup;
  readonly settings: Settings;
  readonly view: SettingsView;
};

// One group of settings under its heading: "General" for the core's, the module's name for a module's.
const Group = ({ group, settings, view }: GroupProps): JSX.Element => {
  const headingId = useId();
  const readOnly = isReadOnly(view.file);

  return (
    <section aria-labelledby={headingId} className={styles.group}>
      <h2 className={styles.heading} id={headingId}>
        {group.module === null
          ? t('ui.screen.settings.general')
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
                value={view.values[descriptor.key] ?? descriptor.default}
              />
            );
          })}
        </div>
      </Surface>
    </section>
  );
};
