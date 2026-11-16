import type { JSX } from 'react';

import { Notice } from '../../design/feedback/notice/notice';
import { t } from '../../i18n';
import { useCore } from '../../state/store';
import { useScanActions } from '../use-scan-actions';

// "Rescan to apply": a setting that applies at the next scan changed since the last scan. The app never rescans on its
// own; the notice offers the scan. Nothing while a scan runs or when the last scan is current.
export const RescanNotice = (): JSX.Element | null => {
  const { scan } = useScanActions();
  const changed = useCore((state) => state.overview?.lastScan?.settingsChanged ?? false);
  const scanning = useCore((state) => state.status.state === 'scanning');

  return changed && !scanning ? (
    <Notice
      action={{ label: t('ui.settings.rescan.action'), onClick: scan }}
      title={t('ui.settings.rescan.title')}
      tone="info"
    >
      {t('ui.settings.rescan.text')}
    </Notice>
  ) : null;
};
