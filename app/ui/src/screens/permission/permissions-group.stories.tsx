import type { JSX } from 'react';

import type { Access, Capability, CapabilityState, ModuleInfo } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { ACCESS_DICTIONARY } from '../../test/demo-scan';
import { PermissionRows } from './permission-rows';

const ignore = (): void => undefined;

const DICTIONARY = {
  ...ACCESS_DICTIONARY,
  'core.access.desktop.label': 'Let Fresher look in Desktop',
  'core.access.state.denied': 'Not allowed',
  'core.access.state.granted': 'Allowed',
  'core.access.state.not-asked': 'Not asked yet',
  'core.access.state.unknown': 'Could not be checked',
  'core.access.tool.not-ready': '{tool} is installed but not running. Start it, then scan again.',
  'docker.enabled.label': 'Docker',
};

const ALL_FILES: Capability = { kind: 'all-files' };
const DESKTOP: Capability = { kind: 'folder', value: 'desktop' };
const REMOVABLE: Capability = { kind: 'volume', value: 'removable' };
const DOCKER: Capability = { kind: 'tool', value: 'docker' };

const MODULES: readonly ModuleInfo[] = [
  {
    categories: [
      {
        availability: { kind: 'ready' },
        descriptor: { danger: 'low', id: 'docker.images', needs: [{ capability: DOCKER, kind: 'required' }] },
      },
    ],
    choices: [],
    group: 'development',
    id: 'docker',
    settings: [],
  },
];

const state = (capability: Capability, access: Access, note: CapabilityState['note'] = null): CapabilityState => ({
  access,
  capability,
  note,
});

const NOT_RUNNING = { args: { tool: { kind: 'text', value: 'Docker' } }, key: 'core.access.tool.not-ready' } as const;

const rows = (states: readonly CapabilityState[]): (() => JSX.Element) =>
  function Story(): JSX.Element {
    // Set here, so a story shown alone has its texts whatever else the gallery loads.
    setCoreDictionary(DICTIONARY);

    return (
      <PermissionRows
        modules={MODULES}
        onOpenSettings={ignore}
        onReopen={ignore}
        states={states}
        switchOf={(id) => <p>{`The switch of ${id}`}</p>}
      />
    );
  };

export const stories = storiesOf('PermissionRows', {
  'all-granted': {
    render: rows([state(ALL_FILES, 'granted'), state(DESKTOP, 'granted'), state(DOCKER, 'granted')]),
    width: 560,
  },
  'every-state': {
    render: rows([
      state(ALL_FILES, 'denied'),
      state(DESKTOP, 'not-asked'),
      state(REMOVABLE, 'unknown'),
      state(DOCKER, 'unknown', NOT_RUNNING),
    ]),
    width: 560,
  },
});
