import type { JSX } from 'react';

import type { SettingsFile } from '../../ipc/bindings.generated';

import { Notice } from '../../design/feedback/notice/notice';
import { t, tText } from '../../i18n';

// The settings file cannot be changed: it was written by a newer version, or it could not be read.
export const isReadOnly = (file: SettingsFile): boolean => file === 'newer-version' || file === 'unreadable';

// Why the defaults are in use, when the settings file was not loaded; nothing when it was.
export const FileNotice = ({ file }: { readonly file: SettingsFile }): JSX.Element | null => {
  switch (file) {
    case 'broken':
      return (
        <Notice title={t('ui.screen.settings.file.broken.title')} tone="warning">
          {t('ui.screen.settings.file.broken.text')}
        </Notice>
      );
    case 'loaded':
      return null;
    case 'newer-version':
      return <ReadOnlyNotice title={t('ui.screen.settings.file.newer-version.title')} />;
    case 'unreadable':
      return <ReadOnlyNotice title={t('ui.screen.settings.file.unreadable.title')} />;
  }
};

const ReadOnlyNotice = ({ title }: { readonly title: string }): JSX.Element => (
  <Notice title={title} tone="warning">
    {tText({ args: {}, key: 'core.error.settings-read-only' })}
  </Notice>
);
