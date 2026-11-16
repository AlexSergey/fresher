import type { JSX } from 'react';

import { Modal, type ModalProps } from './modal';

export type DialogProps = Omit<ModalProps, 'dismissible' | 'kind'> & {
  // A click on the backdrop closes it: off by default, so a decision is not lost by a stray click.
  readonly dismissible?: boolean;
};

// A modal dialog: at most 560 px wide (720 for `size="large"`), its body scrolls inside. It carries the confirmation of
// a removal, so it never settles or bounces: it fades and scales in once.
export const Dialog = ({ dismissible = false, ...props }: DialogProps): JSX.Element | null => (
  <Modal {...props} dismissible={dismissible} kind="dialog" />
);
