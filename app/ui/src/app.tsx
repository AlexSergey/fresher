import type { JSX } from 'react';

import type { CoreStore } from './state/store';

import { AppFrame } from './screens/app-frame';
import { AppProviders } from './screens/app-providers';

export const App = ({ store }: { readonly store: CoreStore }): JSX.Element => (
  <AppProviders store={store}>
    <AppFrame />
  </AppProviders>
);
