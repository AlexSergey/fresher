import type { JSX } from 'react';

import { Modal, type ModalProps } from '../dialog/modal';

export type SheetProps = Omit<ModalProps, 'dismissible' | 'kind' | 'size'> & {
  // A click on the backdrop closes it: on by default, the sheet holds secondary content.
  readonly dismissible?: boolean;
};

// The same native modal dialog, attached to the bottom edge of the main pane and as wide as its content column; for
// longer secondary content, such as the details of a finding.
export const Sheet = ({ dismissible = true, ...props }: SheetProps): JSX.Element | null => (
  <Modal {...props} dismissible={dismissible} kind="sheet" />
);
