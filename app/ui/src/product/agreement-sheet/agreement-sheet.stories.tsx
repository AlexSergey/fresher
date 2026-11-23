import type { JSX } from 'react';

import type { AgreementView } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import {
  AGREEMENT_CHANGED,
  AGREEMENT_CHANGES_DICTIONARY,
  AGREEMENT_DICTIONARY,
  AGREEMENT_NOT_ACCEPTED,
} from '../../test/agreement';
import { AgreementSheet } from './agreement-sheet';
import styles from './agreement-sheet.stories.module.css';

const ignore = (): void => undefined;

// The open sheet over a page, as the first cleanup asks for it.
const open = (view: AgreementView): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary({ ...AGREEMENT_DICTIONARY, ...AGREEMENT_CHANGES_DICTIONARY });

    return (
      <div className={styles.stage}>
        <p className={styles.behind}>The review of the selection.</p>
        <AgreementSheet onAccept={ignore} onDecline={ignore} onLicence={ignore} open view={view} />
      </div>
    );
  },
  width: 1052,
});

export const stories = storiesOf('AgreementSheet', {
  changed: open(AGREEMENT_CHANGED),
  first: open(AGREEMENT_NOT_ACCEPTED),
});
