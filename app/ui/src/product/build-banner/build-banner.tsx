import type { JSX } from 'react';

import type { UiKey } from '../../i18n/types';
import type { Build, Simulated } from '../../ipc/bindings.generated';

import { t, tText } from '../../i18n';
import styles from './build-banner.module.css';

export type BuildBannerProps = { readonly build: Build };

type SandboxBuild = Extract<Build, { readonly kind: 'sandbox' }>;

const SIMULATED_TEXT: Readonly<Record<Simulated, UiKey>> = {
  access: 'ui.build-banner.simulated.access',
  installs: 'ui.build-banner.simulated.installs',
  running: 'ui.build-banner.simulated.running',
  spotlight: 'ui.build-banner.simulated.spotlight',
  tools: 'ui.build-banner.simulated.tools',
};

// The sandbox's banner: its folder, the core's sentence of where its cleanups go and, when the core reports any, what
// is simulated, in the core's order.
const sandboxText = (build: SandboxBuild): string => {
  const path = { kind: 'text', value: build.path } as const;
  const trash = { kind: 'text', value: tText(build.trash) } as const;
  if (build.simulated.length === 0) {
    return t('ui.build-banner.sandbox', { path, trash });
  }
  const parts = build.simulated.map((part) => t(SIMULATED_TEXT[part])).join(', ');

  return t('ui.build-banner.sandbox-simulated', { parts: { kind: 'text', value: parts }, path, trash });
};

// A thin, quiet strip above the window in a development build: which sandbox it lives in, where its cleanups go (and
// what of the Mac its world simulates, when the core says so), or that it removes nothing outside one. Nothing in a
// release build.
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
          {sandboxText(build)}
        </p>
      );
  }
};
