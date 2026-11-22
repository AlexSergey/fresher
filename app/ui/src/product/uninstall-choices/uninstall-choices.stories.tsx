import { type JSX, useState } from 'react';

import type { UninstallChoice } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { VIDEO_STUDIO_DETAIL } from '../../test/app-detail';
import { APPS_DICTIONARY } from '../../test/apps-list';
import { UninstallChoices } from './uninstall-choices';

const nothing = (): void => undefined;

const state = (
  start: UninstallChoice,
  preparing = false,
): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary(APPS_DICTIONARY);
    const [value, setValue] = useState(start);

    return (
      <UninstallChoices
        choices={VIDEO_STUDIO_DETAIL.choices}
        onChoose={setValue}
        onUninstall={nothing}
        preparing={preparing}
        value={value}
      />
    );
  },
  width: 640,
});

// Keep-data chosen as the page opens, everything chosen, and the button waiting for the core's plan.
export const stories = storiesOf('UninstallChoices', {
  everything: state('everything'),
  keepData: state('keep-data'),
  preparing: state('keep-data', true),
});
