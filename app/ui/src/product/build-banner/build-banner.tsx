import type { JSX } from 'react';

import type { UiKey } from '../../i18n/types';
import type { Build } from '../../ipc/bindings.generated';

import { t } from '../../i18n';
import styles from './build-banner.module.css';

export type BuildBannerProps = { readonly build: Build };

type SandboxBuild = Extract<Build, { readonly kind: 'sandbox' }>;

// What the sandbox's banner says is simulated, as the core reports it.
const sandboxText = (build: SandboxBuild): UiKey => {
  if (build.simulatedRunning && build.simulatedSpotlight) {
    return 'ui.build-banner.sandbox-simulated-running-spotlight';
  }
  if (build.simulatedRunning) {
    return 'ui.build-banner.sandbox-simulated-running';
  }

  return build.simulatedSpotlight ? 'ui.build-banner.sandbox-simulated-spotlight' : 'ui.build-banner.sandbox';
};

// A thin, quiet strip above the window in a development build: which sandbox it lives in (and that its open apps and
// Spotlight are the sandbox's, when the core says so), or that it removes nothing outside one. Nothing in a release
// build.
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
          {t(sandboxText(build), { path: { kind: 'text', value: build.path } })}
        </p>
      );
  }
};
