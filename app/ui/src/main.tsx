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

createRoot(root).render(
  <StrictMode>
    <App store={createCoreStore()} />
  </StrictMode>,
);
