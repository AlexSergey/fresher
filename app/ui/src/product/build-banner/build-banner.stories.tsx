import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { BUILD_DICTIONARY, sandboxBuild } from '../../test/sandbox-build';
import { BuildBanner } from './build-banner';

const WIDTH = 820;

function Sandbox(): JSX.Element {
  // The core's sentence of where the cleanups go; set here, so the story shown alone has it.
  setCoreDictionary(BUILD_DICTIONARY);

  return <BuildBanner build={sandboxBuild(['running', 'spotlight', 'access'])} />;
}

export const stories = storiesOf('BuildBanner', {
  development: { render: () => <BuildBanner build={{ kind: 'development' }} />, width: WIDTH },
  sandbox: { render: Sandbox, width: WIDTH },
});
