// Where the tooltip goes, in pixels from the window's top left corner.
export type TooltipPlace = { readonly left: number; readonly side: TooltipSide; readonly top: number };

export type TooltipSide = 'bottom' | 'top';

type Placing = {
  readonly anchor: Pick<DOMRect, 'bottom' | 'left' | 'top' | 'width'>;
  readonly preferred: TooltipSide;
  readonly tip: Size;
  readonly viewport: Size;
};

type Size = { readonly height: number; readonly width: number };

// The space between the child and the tooltip, and the least space kept from the window's edges.
const GAP = 6;
const MARGIN = 8;

// Centred over or under the anchor, on the preferred side unless the tooltip would leave the window there, and kept
// inside the window's width.
export const placeTooltip = ({ anchor, preferred, tip, viewport }: Placing): TooltipPlace => {
  const above = anchor.top - GAP - tip.height;
  const below = anchor.bottom + GAP;
  const fits = { bottom: below + tip.height <= viewport.height - MARGIN, top: above >= MARGIN };
  const other = preferred === 'top' ? 'bottom' : 'top';
  // When neither side fits, the preferred one stays.
  const side = fits[preferred] || !fits[other] ? preferred : other;
  const centred = anchor.left + anchor.width / 2 - tip.width / 2;
  const left = Math.max(MARGIN, Math.min(centred, viewport.width - MARGIN - tip.width));

  return { left: Math.round(left), side, top: Math.round(side === 'top' ? above : below) };
};
