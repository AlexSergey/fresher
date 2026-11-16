// The arithmetic of a virtual list of fixed-height rows: which rows are in view, where a key moves the focus, how far
// to scroll to show a row. Indices are 0-based; a range is inclusive.

export type Range = { readonly first: number; readonly last: number };

export type Viewport = {
  readonly count: number;
  readonly height: number;
  readonly rowHeight: number;
  readonly scrollTop: number;
};

const clamp = (value: number, low: number, high: number): number => Math.min(Math.max(value, low), high);

// The rows the viewport shows, even partly; null when there are none.
export const rowsInView = ({ count, height, rowHeight, scrollTop }: Viewport): null | Range => {
  if (count === 0) {
    return null;
  }

  const first = clamp(Math.floor(scrollTop / rowHeight), 0, count - 1);
  const last = clamp(Math.ceil((scrollTop + height) / rowHeight) - 1, first, count - 1);

  return { first, last };
};

// The range widened by `overscan` rows above and below, inside the list.
export const withOverscan = ({ first, last }: Range, overscan: number, count: number): Range => ({
  first: Math.max(0, first - overscan),
  last: Math.min(count - 1, last + overscan),
});

// Where a key moves the focus from `index`, or null for a key that does not move it. A page is the rows that fit.
export const moveFocus = (key: string, index: number, count: number, pageSize: number): null | number => {
  const moves: Readonly<Record<string, number>> = {
    ArrowDown: index + 1,
    ArrowUp: index - 1,
    End: count - 1,
    Home: 0,
    PageDown: index + pageSize,
    PageUp: index - pageSize,
  };
  const target = moves[key];

  return target === undefined ? null : clamp(target, 0, count - 1);
};

// The scroll offset that shows the row at `index` whole, moving as little as possible.
export const scrollToShow = (index: number, { height, rowHeight, scrollTop }: Omit<Viewport, 'count'>): number => {
  const top = index * rowHeight;

  if (top < scrollTop) {
    return top;
  }

  if (top + rowHeight > scrollTop + height) {
    return top + rowHeight - height;
  }

  return scrollTop;
};
