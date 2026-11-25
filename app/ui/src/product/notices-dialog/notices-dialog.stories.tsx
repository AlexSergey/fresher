import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { type Notices, NoticesDialog } from './notices-dialog';
import styles from './notices-dialog.stories.module.css';

const ignore = (): void => undefined;

// A fixed sample, not the generated file: the picture does not change with every dependency update.
const NOTICES: Notices = {
  licences: [
    {
      id: 'Apache-2.0',
      name: 'Apache License 2.0',
      text: 'Apache License\nVersion 2.0, January 2004',
      usedBy: ['serde 1.0.0', 'tauri 2.0.0'],
    },
    {
      id: 'MIT',
      name: 'MIT License',
      text: 'Permission is hereby granted, free of charge, to any person obtaining a copy of this software.',
      usedBy: ['react 19.0.0'],
    },
    {
      id: 'MPL-2.0',
      name: 'Mozilla Public License 2.0',
      text: 'Mozilla Public License Version 2.0',
      usedBy: ['selectors 0.1.0'],
    },
  ],
  sources: [
    { licence: 'MPL-2.0', name: 'selectors', url: 'https://crates.io/crates/selectors/0.1.0', version: '0.1.0' },
  ],
};

const Open = (): JSX.Element => (
  <div className={styles.stage}>
    <NoticesDialog notices={NOTICES} onClose={ignore} open />
  </div>
);

const Reading = (): JSX.Element => (
  <div className={styles.stage}>
    <NoticesDialog notices={null} onClose={ignore} open />
  </div>
);

export const stories = storiesOf('NoticesDialog', {
  open: { render: Open, width: 1052 },
  reading: { render: Reading, width: 1052 },
});
