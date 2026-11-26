import { type JSX, useEffect, useRef } from 'react';

import type { CleanupPlanView } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import {
  ALL_LEFT_OUT_PLAN,
  CLEANUP_DICTIONARY,
  DEMO_PLAN,
  HIGH_PLAN,
  LEFT_OUT_PLAN,
  LONG_PLAN,
  LONG_TOGETHER_PLAN,
  MID_PLAN,
  SEVERAL_PLAN,
  TOGETHER_PLAN,
  TOOL_PLAN,
} from '../../test/cleanup-plan';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { ConfirmDialog } from './confirm-dialog';
import styles from './confirm-dialog.stories.module.css';

const ignore = (): void => undefined;

// The open confirmation over a page; `permanent` ticks the permanent choice, as a person would.
const open = (
  plan: CleanupPlanView,
  permanent = false,
  expanded = false,
): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    const stageRef = useRef<HTMLDivElement>(null);

    setCoreDictionary({ ...FINDING_DICTIONARY, ...CLEANUP_DICTIONARY });

    useEffect(() => {
      if (permanent) {
        stageRef.current?.querySelector<HTMLInputElement>('dialog input[type="checkbox"]')?.click();
      }
      if (expanded) {
        for (const toggle of stageRef.current?.querySelectorAll<HTMLButtonElement>('dialog button[aria-expanded]') ??
          []) {
          toggle.click();
        }
      }
    }, []);

    return (
      <div className={styles.stage} ref={stageRef}>
        <p className={styles.behind}>The review of the selection.</p>
        <ConfirmDialog onCancel={ignore} onConfirm={ignore} open plan={plan} />
      </div>
    );
  },
  width: 1052,
});

export const stories = storiesOf('ConfirmDialog', {
  'all-left-out': open(ALL_LEFT_OUT_PLAN),
  'highest-high': open(HIGH_PLAN),
  'left-out': open(LEFT_OUT_PLAN),
  'long-list': open(LONG_PLAN),
  'mid-list': open(MID_PLAN),
  permanent: open(DEMO_PLAN, true),
  'several-categories': open(SEVERAL_PLAN),
  'together-item': open(TOGETHER_PLAN),
  'together-item-open': open(TOGETHER_PLAN, false, true),
  'together-item-open-long': open(LONG_TOGETHER_PLAN, false, true),
  'tool-group': open(TOOL_PLAN),
  trash: open(DEMO_PLAN),
});
