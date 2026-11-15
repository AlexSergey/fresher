import './styles.css';

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './app';
import { createCoreStore } from './state/store';

const root = document.querySelector('#root');

if (root === null) {
  throw new Error('The #root element is missing from index.html');
}

createRoot(root).render(
  <StrictMode>
    <App store={createCoreStore()} />
  </StrictMode>,
);
