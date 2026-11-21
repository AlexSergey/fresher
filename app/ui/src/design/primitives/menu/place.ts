// Where the menu goes, in pixels from the window's top left corner.
export type MenuPlace = { readonly left: number; readonly side: 'above' | 'below'; readonly top: number };

type Placing = {
  readonly anchor: Pick<DOMRect, 'bottom' | 'right' | 'top'>;
  readonly menu: Size;
  readonly viewport: Size;
};

type Size = { readonly height: number; readonly width: number };

// The space between the trigger and the menu, and the least space kept from the window's edges.
const GAP = 4;
const MARGIN = 8;

// Under the anchor with its right edge on the anchor's right edge; above it when it does not fit below and does fit
// above; kept within the window's width.
export const placeMenu = ({ anchor, menu, viewport }: Placing): MenuPlace => {
  const below = anchor.bottom + GAP;
  const above = anchor.top - GAP - menu.height;
  const fitsBelow = below + menu.height <= viewport.height - MARGIN;
  const side = !fitsBelow && above >= MARGIN ? 'above' : 'below';
  const left = Math.max(MARGIN, Math.min(anchor.right - menu.width, viewport.width - MARGIN - menu.width));

  return { left: Math.round(left), side, top: Math.round(side === 'above' ? above : below) };
};
