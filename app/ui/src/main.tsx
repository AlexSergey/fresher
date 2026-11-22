import './design/index.css';

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './app';
import { inApp } from './ipc';
import { createCoreStore } from './state/store';

const root = document.querySelector('#root');

if (root === null) {
  throw new Error('The #root element is missing from index.html');
}

// The native sidebar material shows through a transparent page only in the app; elsewhere the window stays solid.
if (inApp()) {
  document.documentElement.dataset.window = 'vibrant';
}

const store = createCoreStore();

createRoot(root).render(
  <StrictMode>
    <App store={store} />
  </StrictMode>,
);

// Debug builds only: the release bundle has no player, the chunk is not even imported.
if (import.meta.env.DEV && inApp()) {
  void import('./demo/start').then(({ startDemo }) => startDemo(store));
}
