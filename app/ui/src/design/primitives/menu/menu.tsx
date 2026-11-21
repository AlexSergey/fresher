import { type JSX, type KeyboardEvent, useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

import { type IconShape } from '../../icons/icons';
import { IconButton } from '../icon-button/icon-button';
import styles from './menu.module.css';
import { type MenuPlace, placeMenu } from './place';

export type MenuItem = {
  readonly description?: string;
  readonly key: string;
  readonly label: string;
  readonly onSelect: () => void;
};

export type MenuProps = {
  // Open from the start, without taking the focus: for the gallery.
  readonly defaultOpen?: boolean | undefined;
  readonly icon: IconShape;
  readonly items: readonly MenuItem[];
  // The trigger's accessible name.
  readonly label: string;
  // -1 keeps the trigger out of the tab order: a list's row that is not its tab stop.
  readonly tabIndex?: -1 | 0;
};

// A button that opens a short list of actions, in a portal on <body>, placed by the component. The focus rests on one
// item at a time (arrows, Home and End move it, Enter or Space chooses). Choosing closes the menu, gives the focus back
// to the trigger, then calls the item; Escape does the same without a choice; Tab and a pointer down outside close it.
export const Menu = ({ defaultOpen = false, icon, items, label, tabIndex = 0 }: MenuProps): JSX.Element => {
  const id = useId();
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const itemsRef = useRef<(HTMLDivElement | null)[]>([]);
  // Set when the focus has to follow the active item: not for a menu forced open.
  const focusItemRef = useRef(false);
  const [open, setOpen] = useState(defaultOpen);
  const [active, setActive] = useState(0);
  const [place, setPlace] = useState<MenuPlace | null>(null);

  const openAt = (index: number): void => {
    focusItemRef.current = true;
    setActive(index);
    setOpen(true);
  };
  const close = (): void => {
    focusItemRef.current = false;
    setOpen(false);
  };
  const closeToTrigger = (): void => {
    close();
    triggerRef.current?.focus();
  };

  useEffect(() => {
    if (!open) {
      return;
    }

    const onPointerDown = (event: PointerEvent): void => {
      const target = event.target as Node;

      if (!menuRef.current?.contains(target) && !triggerRef.current?.contains(target)) {
        close();
      }
    };

    document.addEventListener('pointerdown', onPointerDown);

    return (): void => {
      document.removeEventListener('pointerdown', onPointerDown);
    };
  }, [open]);

  // Measured before the first paint: the menu is drawn hidden once, placed, then shown. Placed again when the window
  // resizes or anything scrolls.
  useLayoutEffect(() => {
    const trigger = triggerRef.current;
    const menu = menuRef.current;

    if (!open || trigger === null || menu === null) {
      setPlace(null);

      return;
    }

    const measure = (): void => {
      setPlace(
        placeMenu({
          anchor: trigger.getBoundingClientRect(),
          menu: { height: menu.offsetHeight, width: menu.offsetWidth },
          viewport: { height: window.innerHeight, width: window.innerWidth },
        }),
      );
    };

    measure();
    window.addEventListener('resize', measure);
    window.addEventListener('scroll', measure, true);

    return (): void => {
      window.removeEventListener('resize', measure);
      window.removeEventListener('scroll', measure, true);
    };
  }, [open, items]);

  // Only once placed: the menu is hidden until then, and a hidden item cannot take the focus.
  const placed = place !== null;

  useEffect(() => {
    if (open && placed && focusItemRef.current) {
      itemsRef.current[active]?.focus();
    }
  }, [open, placed, active]);

  const onTriggerKeyDown = (event: KeyboardEvent<HTMLElement>): void => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      openAt(event.key === 'ArrowDown' ? 0 : items.length - 1);
    }
  };

  const choose = (item: MenuItem): void => {
    closeToTrigger();
    item.onSelect();
  };

  const onMenuKeyDown = (event: KeyboardEvent<HTMLElement>): void => {
    const last = items.length - 1;
    const target: Partial<Record<string, number>> = {
      ArrowDown: active === last ? 0 : active + 1,
      ArrowUp: active === 0 ? last : active - 1,
      End: last,
      Home: 0,
    };
    const next = target[event.key];

    if (next !== undefined) {
      event.preventDefault();
      focusItemRef.current = true;
      setActive(next);
    } else if (event.key === 'Escape') {
      event.preventDefault();
      closeToTrigger();
    } else if (event.key === 'Tab') {
      // The focus goes back to the trigger first, so the browser moves on from there.
      closeToTrigger();
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      const item = items[active];

      if (item !== undefined) {
        choose(item);
      }
    }
  };

  return (
    <>
      <IconButton
        aria-controls={open ? id : undefined}
        aria-expanded={open}
        aria-haspopup="menu"
        icon={icon}
        label={label}
        onClick={() => {
          if (open) {
            closeToTrigger();
          } else {
            openAt(0);
          }
        }}
        onKeyDown={onTriggerKeyDown}
        ref={triggerRef}
        size="small"
        tabIndex={tabIndex}
      />
      {open
        ? createPortal(
            <div
              aria-label={label}
              className={styles.menu}
              data-placed={place === null ? undefined : true}
              id={id}
              onKeyDown={onMenuKeyDown}
              ref={menuRef}
              role="menu"
              style={place === null ? undefined : { left: place.left, top: place.top }}
              tabIndex={-1}
            >
              {items.map((item, index) => (
                // The menu's own key handler serves the keyboard; a click is the pointer's way to choose.
                // eslint-disable-next-line jsx-a11y-x/click-events-have-key-events -- see above
                <div
                  aria-describedby={item.description === undefined ? undefined : `${id}-${item.key}-description`}
                  aria-labelledby={`${id}-${item.key}-label`}
                  className={styles.item}
                  key={item.key}
                  onClick={() => {
                    choose(item);
                  }}
                  onFocus={() => {
                    setActive(index);
                  }}
                  ref={(element) => {
                    itemsRef.current[index] = element;
                  }}
                  role="menuitem"
                  tabIndex={index === active ? 0 : -1}
                >
                  <span id={`${id}-${item.key}-label`}>{item.label}</span>
                  {item.description === undefined ? null : (
                    <span className={styles.description} id={`${id}-${item.key}-description`}>
                      {item.description}
                    </span>
                  )}
                </div>
              ))}
            </div>,
            document.body,
          )
        : null}
    </>
  );
};
