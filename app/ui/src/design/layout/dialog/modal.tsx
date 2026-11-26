import { type JSX, type ReactNode, type RefObject, useEffectEvent, useId, useLayoutEffect, useRef } from 'react';

import { X } from '../../icons/icons';
import { usePresence } from '../../motion';
import { IconButton } from '../../primitives/icon-button/icon-button';
import { DURATION } from '../../tokens/motion';
import { useOverflowFocus } from '../scroll-area/use-overflow-focus';
import styles from './modal.module.css';

export type ModalProps = {
  readonly children?: ReactNode;
  // The accessible name of the close button.
  readonly closeLabel: string;
  readonly description?: string;
  // A click on the backdrop closes it.
  readonly dismissible: boolean;
  // The buttons. The dialog binds no key to them: Enter presses only the focused one.
  readonly footer?: ReactNode;
  // The element focused on open; the first focusable one otherwise.
  readonly initialFocus?: RefObject<HTMLElement | null>;
  readonly kind: 'dialog' | 'sheet';
  // Asks the owner to close it (Escape, the close button, the backdrop): the owner sets `open` to false.
  readonly onClose: () => void;
  readonly open: boolean;
  readonly size?: 'large' | 'medium';
  readonly title: string;
};

type PanelProps = Omit<ModalProps, 'open'> & {
  readonly closing: boolean;
  readonly dialogRef: RefObject<HTMLDialogElement | null>;
};

const FADE_OUT: readonly Keyframe[] = [{ opacity: 1 }, { opacity: 0 }];

const FOCUSABLE =
  'a[href], button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])';

// The dialog's content. A long one without a control of its own takes the focus, so the keyboard can scroll it.
const Body = ({ children, titleId }: { readonly children: ReactNode; readonly titleId: string }): JSX.Element => {
  const areaRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const focusable = useOverflowFocus(areaRef, contentRef);

  return (
    <div
      aria-labelledby={focusable ? titleId : undefined}
      className={styles.body}
      ref={areaRef}
      role={focusable ? 'region' : undefined}
      tabIndex={focusable ? 0 : undefined}
    >
      <div className={styles.content} ref={contentRef}>
        {children}
      </div>
    </div>
  );
};

// The open dialog, from `showModal()` on mount to `close()` on unmount, when the focus goes back to its opener.
const Panel = ({
  children,
  closeLabel,
  closing,
  description,
  dialogRef,
  dismissible,
  footer,
  initialFocus,
  kind,
  onClose,
  size = 'medium',
  title,
}: PanelProps): JSX.Element => {
  const titleId = useId();
  const descriptionId = useId();

  const focusFirst = useEffectEvent((dialog: HTMLDialogElement): void => {
    (initialFocus?.current ?? dialog.querySelector<HTMLElement>(FOCUSABLE))?.focus();
  });

  useLayoutEffect(() => {
    const dialog = dialogRef.current;

    if (dialog === null) {
      return;
    }

    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;

    // Modal: the focus is trapped inside and the rest of the page is inert.
    dialog.showModal();
    focusFirst(dialog);

    return (): void => {
      dialog.close();
      opener?.focus();
    };
  }, [dialogRef]);

  const requestClose = (): void => {
    if (!closing) {
      onClose();
    }
  };

  return (
    // The backdrop is the dialog itself: a click on it has the dialog as its target.
    // eslint-disable-next-line jsx-a11y-x/click-events-have-key-events, jsx-a11y-x/no-noninteractive-element-interactions -- Escape is the keyboard's way out, through `cancel`
    <dialog
      aria-describedby={description === undefined ? undefined : descriptionId}
      aria-labelledby={titleId}
      className={styles.modal}
      data-closing={closing || undefined}
      data-kind={kind}
      data-size={size}
      onCancel={(event) => {
        // Escape: the owner closes it, after the exit.
        event.preventDefault();
        requestClose();
      }}
      onClick={(event) => {
        if (dismissible && event.target === event.currentTarget) {
          requestClose();
        }
      }}
      ref={dialogRef}
    >
      <div className={styles.panel}>
        <header className={styles.header}>
          <div className={styles.heading}>
            <h2 className={styles.title} id={titleId}>
              {title}
            </h2>
            {description === undefined ? null : (
              <p className={styles.description} id={descriptionId}>
                {description}
              </p>
            )}
          </div>
          <IconButton icon={X} label={closeLabel} onClick={requestClose} size="small" />
        </header>
        {children === undefined ? null : <Body titleId={titleId}>{children}</Body>}
        {footer === undefined ? null : <footer className={styles.footer}>{footer}</footer>}
      </div>
    </dialog>
  );
};

// A native modal `<dialog>`: Escape, the focus trap and the inert page come from the platform. It fades in once;
// closing, it fades out over `--duration-fast`, then closes, and the focus returns to the element that opened it.
export const Modal = ({ open, ...props }: ModalProps): JSX.Element | null => {
  const { mounted, ref } = usePresence<HTMLDialogElement>(open, { duration: DURATION.fast, exit: FADE_OUT });

  return mounted ? <Panel {...props} closing={!open} dialogRef={ref} /> : null;
};
