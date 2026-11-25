import type { Build, Dictionary, Text } from '../ipc/bindings.generated';

import coreTexts from '../../../../crates/core/locales/en.json' with { type: 'json' };

// A development build in a sandbox, as the core describes it: its folder, what its world simulates and where its
// cleanups go. The texts are the core's, read from its locale file, so a changed sentence changes the tests.
// Shared by the unit tests, the stories and the e2e mock.

export const SANDBOX_PATH = '/Users/dev/sandboxes/trial';

// The sandbox's own home, the one its engine works on.
export const SANDBOX_HOME = `${SANDBOX_PATH}/home`;

// Where the sandbox's cleanups go: its own Trash folder, as the core says it.
export const SANDBOX_TRASH: Text = {
  args: { path: { kind: 'path', value: `${SANDBOX_PATH}/home/.Trash` } },
  key: 'core.build.trash.sandbox',
};

// Where they go with `--real-trash`: the Trash of this Mac.
export const SYSTEM_TRASH: Text = { args: {}, key: 'core.build.trash.system' };

// The core's sentences of where a sandbox's cleanups go.
export const BUILD_DICTIONARY: Dictionary = {
  'core.build.trash.sandbox': coreTexts['core.build.trash.sandbox'],
  'core.build.trash.system': coreTexts['core.build.trash.system'],
};

// A sandbox build simulating `simulated`, its cleanups going to its own Trash.
export const sandboxBuild = (
  simulated: Extract<Build, { readonly kind: 'sandbox' }>['simulated'] = ['running'],
): Build => ({ kind: 'sandbox', path: SANDBOX_PATH, simulated, trash: SANDBOX_TRASH });
