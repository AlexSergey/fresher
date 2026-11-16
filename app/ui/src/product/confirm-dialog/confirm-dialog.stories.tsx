import { type JSX, useEffect, useRef } from 'react';

import type { ConfirmContent } from './confirm-dialog';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { confirmContentOf, LONG_CONTENT, SHORT_CONTENT } from '../../test/confirm-content';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { ConfirmDialog } from './confirm-dialog';
import styles from './confirm-dialog.stories.module.css';

const ignore = (): void => undefined;

// The open confirmation over a page; `permanent` ticks the permanent choice, as a person would.
const open = (
  content: ConfirmContent,
  permanent = false,
): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    const stageRef = useRef<HTMLDivElement>(null);

    setCoreDictionary(FINDING_DICTIONARY);

    useEffect(() => {
      if (permanent) {
        stageRef.current?.querySelector<HTMLInputElement>('dialog input[type="checkbox"]')?.click();
      }
    }, []);

    return (
      <div className={styles.stage} ref={stageRef}>
        <p className={styles.behind}>The review of the selection.</p>
        <ConfirmDialog content={content} onCancel={ignore} onConfirm={ignore} open />
      </div>
    );
  },
  width: 1052,
});

export const stories = storiesOf('ConfirmDialog', {
  'highest-high': open(confirmContentOf('high')),
  'long-list': open(LONG_CONTENT),
  permanent: open(SHORT_CONTENT, true),
  trash: open(SHORT_CONTENT),
});
