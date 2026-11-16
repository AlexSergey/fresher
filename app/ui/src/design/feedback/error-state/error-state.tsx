import type { JSX } from 'react';

import type { FeedbackAction } from '../action';

import { CircleAlert } from '../../icons/icons';
import { EmptyState } from '../empty-state/empty-state';

export type ErrorStateProps = {
  // "Try again", from the caller.
  readonly action: FeedbackAction;
  // The core's text, shown as given.
  readonly text: string;
  readonly title: string;
};

// The empty state's layout for a failure: `CircleAlert` in High's glow, the core's text and a way to try again.
export const ErrorState = ({ action, text, title }: ErrorStateProps): JSX.Element => (
  <EmptyState action={action} icon={CircleAlert} text={text} title={title} tone="danger" />
);
