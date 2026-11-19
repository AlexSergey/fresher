import { type JSX, type ReactNode, useId } from 'react';

import type { SettingsView } from '../../ipc/bindings.generated';
import type { Settings } from './use-settings';

import { PageHeader } from '../../design/layout/page-header/page-header';
import { Surface } from '../../design/layout/surface/surface';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { t, tText } from '../../i18n';
import { SettingsRow } from '../../product/settings-row/settings-row';
import { useNavigation } from '../../state/navigation';
import { PermissionsGroup } from '../permission/permissions-group';
import { useAppInfo } from '../use-app-info';
import { useModules } from '../use-modules';
import { FileNotice, isReadOnly } from './file-notice';
import { IgnoredGroup } from './ignored-group';
import { OwnDataGroup } from './own-data-group';
import { RescanNotice } from './rescan-notice';
import { type SettingsGroup, settingsGroups } from './settings-groups';
import styles from './settings-screen.module.css';
import { useSettings } from './use-settings';

// "What can I change?": the groups in order (General, Scan, each module group with its modules, Ignored, Permissions,
// Fresher's own data, About); why the settings file cannot be changed, when it cannot; "Rescan to apply" when a change waits for a scan.
export const SettingsScreen = (): JSX.Element => {
  const settings = useSettings();
  const modules = useModules();
  // Read once: the Ignored tab to open on is only the starting tab, so the screen needs no subscription.
  const screen = useNavigation().getScreen();
  const { view } = settings;
  const ignoredId = useId();
  const ownDataId = useId();
  const groups = view === null || modules === null ? null : settingsGroups(view.settings, modules);
  const readOnly = view !== null && isReadOnly(view.file);
  const rows = (group: SettingsGroup, inView: SettingsView): JSX.Element => (
    <Group group={group} key={group.kind === 'core' ? group.group : group.module} settings={settings} view={inView} />
  );
  const moduleGroups = (inView: SettingsView): readonly JSX.Element[] =>
    (groups?.modules ?? []).map(({ group, items }) => (
      <ModuleGroupRows group={group} key={group}>
        {items.map((item) => rows(item, inView))}
      </ModuleGroupRows>
    ));

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
            {moduleGroups(view)}
            <IgnoredGroup
              headingId={ignoredId}
              modules={modules ?? []}
              readOnly={readOnly}
              view={view}
              {...(screen.kind === 'settings' && screen.tab !== undefined ? { opensOn: screen.tab } : {})}
            />
            <PermissionsGroup settings={settings} view={view} />
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
  scan: 'ui.screen.settings.group.scan',
} as const;

// A module group's modules under the group's name.
const ModuleGroupRows = ({
  children,
  group,
}: {
  readonly children: ReactNode;
  readonly group: string;
}): JSX.Element => {
  const headingId = useId();

  return (
    <section aria-labelledby={headingId} className={styles.group}>
      <h2 className={styles.heading} id={headingId}>
        {tText({ args: {}, key: `core.group.${group}.name` })}
      </h2>
      {children}
    </section>
  );
};

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
      {group.kind === 'core' ? (
        <h2 className={styles.heading} id={headingId}>
          {t(GROUP_HEADINGS[group.group])}
        </h2>
      ) : (
        <h3 className={styles.moduleHeading} id={headingId}>
          {tText({ args: {}, key: `${group.module}.enabled.label` })}
        </h3>
      )}
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
  const version = useAppInfo()?.version ?? null;

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
