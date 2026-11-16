import type { JSX } from 'react';

import { Tooltip } from '../../primitives/tooltip/tooltip';
import { VisuallyHidden } from '../../primitives/visually-hidden/visually-hidden';
import styles from './size-text.module.css';

export type SizeTextProps = {
  // What assistive technology reads: the caller's words ("at least 42 GB", "Size not known").
  readonly accessibleText: string;
  readonly size?: SizeTextSize;
} & (
  | {
      readonly completeness: 'complete';
      readonly hint?: never;
      readonly text: string;
    }
  | {
      readonly completeness: 'partial';
      // Why the size is a lower bound, shown in a tooltip over the plus.
      readonly hint?: string;
      readonly text: string;
    }
  | { readonly completeness: 'unknown'; readonly hint?: never; readonly text?: never }
);

type SizeTextSize = 'body' | 'callout' | 'display' | 'hero' | 'title-3';

const UNKNOWN = '—';

// An honest size, already formatted by the caller: complete, a lower bound with a quiet plus, or unknown (a dash,
// never 0). Shown in tabular figures; assistive technology reads the caller's accessible text instead.
export const SizeText = ({ accessibleText, completeness, hint, size = 'body', text }: SizeTextProps): JSX.Element => (
  <span className={styles.size} data-numeric data-size={size}>
    <span aria-hidden="true">{completeness === 'unknown' ? UNKNOWN : text}</span>
    {completeness === 'partial' ? <Plus hint={hint} /> : null}
    <VisuallyHidden>{accessibleText}</VisuallyHidden>
  </span>
);

const Plus = ({ hint }: { readonly hint: string | undefined }): JSX.Element => {
  const plus = (
    <span aria-hidden="true" className={styles.plus}>
      +
    </span>
  );

  if (hint === undefined) {
    return plus;
  }

  // The accessible text already says it is a lower bound; the tooltip is for the eye.
  return (
    <Tooltip content={hint} repeatsName>
      {() => plus}
    </Tooltip>
  );
};
