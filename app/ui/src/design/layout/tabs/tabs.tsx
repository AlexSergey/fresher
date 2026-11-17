import { type JSX, type KeyboardEvent, type ReactNode, useId, useRef } from 'react';

import type { Forced } from '../../primitives/forced';

import styles from './tabs.module.css';

export type TabsProps<T extends string> = Forced & {
  // The panel of the selected tab.
  readonly children: ReactNode;
  // The tab list's accessible name.
  readonly label: string;
  readonly onChange: (value: T) => void;
  readonly tabs: readonly Tab<T>[];
  readonly value: T;
};

// A tab's label, with its count after it when there is one to show.
type Tab<T extends string> = { readonly count?: number; readonly label: string; readonly value: T };

// The tab a key moves to from `index`, or `null` for a key that does not move the selection. Left and Right wrap.
const targetOf = (key: string, index: number, count: number): null | number => {
  switch (key) {
    case 'ArrowLeft': {
      return (index - 1 + count) % count;
    }
    case 'ArrowRight': {
      return (index + 1) % count;
    }
    case 'End': {
      return count - 1;
    }
    case 'Home': {
      return 0;
    }
    default: {
      return null;
    }
  }
};

// Tabs over one panel: the selected tab is the only tab stop, Left and Right (Home and End) select another tab and
// move to it, and the panel is named by its tab.
export const Tabs = <T extends string>({
  children,
  label,
  onChange,
  tabs,
  value,
  ...forced
}: TabsProps<T>): JSX.Element => {
  const baseId = useId();
  const buttonsRef = useRef<(HTMLButtonElement | null)[]>([]);
  const selected = Math.max(
    tabs.findIndex((tab) => tab.value === value),
    0,
  );
  const tabId = (index: number): string => `${baseId}-tab-${String(index)}`;

  const onKeyDown = (event: KeyboardEvent<HTMLButtonElement>): void => {
    const target = targetOf(event.key, selected, tabs.length);

    if (target === null) {
      return;
    }

    event.preventDefault();
    const tab = tabs[target];

    if (tab !== undefined && target !== selected) {
      onChange(tab.value);
      buttonsRef.current[target]?.focus();
    }
  };

  return (
    <div>
      <div aria-label={label} className={styles.list} role="tablist">
        {tabs.map((tab, index) => (
          <button
            key={tab.value}
            {...(index === selected ? forced : {})}
            aria-controls={`${baseId}-panel`}
            aria-selected={index === selected}
            className={styles.tab}
            id={tabId(index)}
            onClick={() => {
              onChange(tab.value);
            }}
            onKeyDown={onKeyDown}
            ref={(element) => {
              buttonsRef.current[index] = element;
            }}
            role="tab"
            tabIndex={index === selected ? 0 : -1}
            type="button"
          >
            {tab.count === undefined || tab.count === 0 ? tab.label : `${tab.label} · ${String(tab.count)}`}
          </button>
        ))}
      </div>
      <div aria-labelledby={tabId(selected)} className={styles.panel} id={`${baseId}-panel`} role="tabpanel">
        {children}
      </div>
    </div>
  );
};
