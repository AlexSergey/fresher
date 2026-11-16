import { type JSX, type ReactElement, useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

import { placeTooltip, type TooltipPlace, type TooltipSide } from './place';
import styles from './tooltip.module.css';

export type TooltipProps = {
  // Renders one focusable element, given the id to pass on as its `aria-describedby` while the tooltip describes it.
  readonly children: (describedBy: string | undefined) => ReactElement;
  readonly content: string;
  // Shown from the start, until the pointer leaves, the focus goes or Escape.
  readonly defaultOpen?: boolean;
  // Above the child (the default) or below it; the tooltip flips when its side would leave the window.
  readonly placement?: TooltipSide;
  // The tooltip repeats the child's accessible name (an icon button's label), so it is not linked as its description.
  readonly repeatsName?: boolean;
};

// How long the pointer rests on the child before the tooltip shows, in milliseconds.
export const HOVER_DELAY = 500;

// A short text over a control: after a rest of the pointer, or at once on keyboard focus; hidden by Escape, blur and
// the pointer leaving. Rendered in a portal on <body> and placed by the component (no Popover API in the baseline).
export const Tooltip = ({
  children,
  content,
  defaultOpen = false,
  placement = 'top',
  repeatsName = false,
}: TooltipProps): JSX.Element => {
  const anchorRef = useRef<HTMLSpanElement>(null);
  const tipRef = useRef<HTMLDivElement>(null);
  const id = useId();
  const [open, setOpen] = useState(defaultOpen);
  const [place, setPlace] = useState<null | TooltipPlace>(null);

  useEffect(() => {
    const anchor = anchorRef.current;

    if (anchor === null) {
      return;
    }

    let timer: ReturnType<typeof setTimeout> | undefined;
    // A press focuses the child too; only a focus that did not come from the pointer shows the tooltip at once.
    let pressed = false;

    const show = (): void => {
      setOpen(true);
    };
    const hide = (): void => {
      clearTimeout(timer);
      setOpen(false);
    };
    const onPointerEnter = (): void => {
      clearTimeout(timer);
      timer = setTimeout(show, HOVER_DELAY);
    };
    const onPointerDown = (): void => {
      pressed = true;
    };
    const onFocusIn = (): void => {
      if (!pressed) {
        show();
      }

      pressed = false;
    };
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') {
        hide();
      }
    };

    anchor.addEventListener('pointerenter', onPointerEnter);
    anchor.addEventListener('pointerleave', hide);
    anchor.addEventListener('pointerdown', onPointerDown);
    anchor.addEventListener('focusin', onFocusIn);
    anchor.addEventListener('focusout', hide);
    anchor.addEventListener('keydown', onKeyDown);

    return (): void => {
      clearTimeout(timer);
      anchor.removeEventListener('pointerenter', onPointerEnter);
      anchor.removeEventListener('pointerleave', hide);
      anchor.removeEventListener('pointerdown', onPointerDown);
      anchor.removeEventListener('focusin', onFocusIn);
      anchor.removeEventListener('focusout', hide);
      anchor.removeEventListener('keydown', onKeyDown);
    };
  }, []);

  // Measured before the first paint: the tooltip is drawn hidden once, placed, then shown.
  useLayoutEffect(() => {
    const anchor = anchorRef.current?.firstElementChild ?? anchorRef.current;
    const tip = tipRef.current;

    if (!open || anchor === null || tip === null) {
      setPlace(null);

      return;
    }

    setPlace(
      placeTooltip({
        anchor: anchor.getBoundingClientRect(),
        preferred: placement,
        tip: { height: tip.offsetHeight, width: tip.offsetWidth },
        viewport: { height: window.innerHeight, width: window.innerWidth },
      }),
    );
  }, [open, placement, content]);

  const described = open && !repeatsName;

  return (
    <span className={styles.anchor} ref={anchorRef}>
      {children(described ? id : undefined)}
      {open
        ? createPortal(
            <div
              className={styles.tooltip}
              data-placed={place === null ? undefined : true}
              data-side={place?.side ?? placement}
              id={id}
              ref={tipRef}
              role="tooltip"
              style={place === null ? undefined : { left: place.left, top: place.top }}
            >
              {content}
            </div>,
            document.body,
          )
        : null}
    </span>
  );
};
