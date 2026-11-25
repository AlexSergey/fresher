import { useEffect, useState } from 'react';

import type { Notices } from '../../product/notices-dialog/notices-dialog';

// The licence notices, read once when first wanted: the generated file is a chunk of its own, loaded from the app's
// bundle (never the network), so the window opens without it.
export const useNotices = (wanted: boolean): Notices | null => {
  const [notices, setNotices] = useState<Notices | null>(null);

  useEffect(() => {
    if (!wanted || notices !== null) {
      return;
    }
    let live = true;
    void import('./notices.generated.json').then((module) => {
      if (live) {
        setNotices(module.default);
      }
    });

    return (): void => {
      live = false;
    };
  }, [notices, wanted]);

  return notices;
};
