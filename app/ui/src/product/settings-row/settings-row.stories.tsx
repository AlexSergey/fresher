import type { JSX } from 'react';

import type { SettingDescriptor } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { SettingsRow, type SettingsRowProps } from './settings-row';

const ignore = (): void => undefined;

const DICTIONARY = {
  'demo.enabled.description': 'Look for the example findings of the demo module.',
  'demo.enabled.label': 'Demo module',
  'demo.method.description': 'Where removed items go. A permanent delete is still confirmed every time.',
  'demo.method.label': 'Remove to',
  'demo.method.options.permanent': 'Delete permanently',
  'demo.method.options.trash': 'Trash',
  'demo.min-age.description': 'A project untouched for this long counts as stale.',
  'demo.min-age.label': 'Stale after',
  'demo.roots.description': 'Where your projects live.',
  'demo.roots.label': 'Project folders',
};

const SWITCH: SettingDescriptor = {
  default: { kind: 'switch', value: true },
  effect: 'next-scan',
  group: 'module',
  key: 'demo.enabled',
  kind: { kind: 'switch' },
};
const NUMBER: SettingDescriptor = {
  default: { kind: 'number', value: 90 },
  effect: 'next-scan',
  group: 'module',
  key: 'demo.min-age',
  kind: { kind: 'number', max: 365, min: 1, unit: 'days' },
};
const CHOICE: SettingDescriptor = {
  default: { kind: 'choice', value: 'trash' },
  effect: 'next-scan',
  group: 'module',
  key: 'demo.method',
  kind: { kind: 'choice', options: ['trash', 'permanent'] },
};
const FOLDERS: SettingDescriptor = {
  default: { kind: 'folders', value: [] },
  effect: 'next-scan',
  group: 'module',
  key: 'demo.roots',
  kind: { kind: 'folders' },
};

const row = (props: Partial<SettingsRowProps> & Pick<SettingsRowProps, 'descriptor'>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    // Set here, so a story shown alone has its texts whatever else the gallery loads.
    setCoreDictionary(DICTIONARY);

    return (
      <SettingsRow onChange={ignore} problem={null} readOnly={false} value={props.descriptor.default} {...props} />
    );
  };

export const stories = storiesOf('SettingsRow', {
  choice: { render: row({ descriptor: CHOICE }), width: 560 },
  folders: {
    render: row({ descriptor: FOLDERS, value: { kind: 'folders', value: ['~/Projects', '~/Work/clients/archive'] } }),
    width: 560,
  },
  invalid: {
    render: row({ descriptor: NUMBER, problem: { key: 'demo.min-age', kind: 'out-of-range', max: 365, min: 1 } }),
    width: 560,
  },
  number: { render: row({ descriptor: NUMBER }), width: 560 },
  'read-only': { render: row({ descriptor: SWITCH, readOnly: true }), width: 560 },
  switch: { render: row({ descriptor: SWITCH }), width: 560 },
});
