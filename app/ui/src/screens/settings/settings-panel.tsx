import type { JSX } from 'react';

import type { SettingDescriptor, SettingsView } from '../../ipc/bindings.generated';
import type { Settings } from './use-settings';

import { SettingsRow, type SettingsRowProps } from '../../product/settings-row/settings-row';
import { isReadOnly } from './file-notice';
import styles from './settings-page.module.css';

type SettingsPanelProps = {
  readonly descriptors: readonly SettingDescriptor[];
  readonly settings: Settings;
  // The texts to show in place of a setting's own, when it has others (a module's on/off switch).
  readonly textsOf?: (descriptor: SettingDescriptor) => SettingsRowProps['texts'];
  readonly view: SettingsView;
};

// The rows of settings in one raised panel, a divider between them.
export const SettingsPanel = ({ descriptors, settings, textsOf, view }: SettingsPanelProps): JSX.Element => {
  const readOnly = isReadOnly(view.file);

  return (
    <div className={styles.panel}>
      {descriptors.map((descriptor) => {
        const refusal = settings.refusals[descriptor.key];
        const texts = textsOf?.(descriptor);

        return (
          <div className={styles.panelRow} key={descriptor.key}>
            <SettingsRow
              descriptor={descriptor}
              onChange={(value) => {
                settings.change(descriptor.key, value);
              }}
              problem={view.problems.find((problem) => problem.key === descriptor.key) ?? null}
              readOnly={readOnly}
              {...(refusal === undefined ? {} : { refusal })}
              {...(texts === undefined ? {} : { texts })}
              {...(descriptor.kind.kind === 'folders'
                ? {
                    onAddFolder: (): void => {
                      settings.addFolder(descriptor.key);
                    },
                  }
                : {})}
              value={view.values[descriptor.key] ?? descriptor.default}
            />
          </div>
        );
      })}
    </div>
  );
};
