import type { JSX } from 'react';

import type { Build } from '../../ipc/bindings.generated';

import { t } from '../../i18n';
import styles from './build-banner.module.css';

export type BuildBannerProps = { readonly build: Build };

// A thin, quiet strip above the window in a development build: which sandbox it lives in, or that it removes nothing
// outside one. Nothing in a release build.
export const BuildBanner = ({ build }: BuildBannerProps): JSX.Element | null => {
  switch (build.kind) {
    case 'development':
      return (
        <p className={styles.banner} role="note">
          {t('ui.build-banner.development')}
        </p>
      );
    case 'release':
      return null;
    case 'sandbox':
      return (
        <p className={styles.banner} role="note">
          {t('ui.build-banner.sandbox', { path: { kind: 'text', value: build.path } })}
        </p>
      );
  }
};
