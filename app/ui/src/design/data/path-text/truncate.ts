const ELLIPSIS = '…';

// A path cut in the middle to fit `fit` characters: its start and its last segment stay, an ellipsis takes the place
// of what is cut. The last segment is never cut; a path that cannot fit even then is clipped by its container.
export const truncateMiddle = (path: string, fit: number): string => {
  const cut = path.lastIndexOf('/');

  if (path.length <= fit || cut <= 0) {
    return path;
  }

  const tail = path.slice(cut);
  const room = fit - tail.length - ELLIPSIS.length;

  return room <= 0 ? `${ELLIPSIS}${tail}` : `${path.slice(0, room)}${ELLIPSIS}${tail}`;
};
