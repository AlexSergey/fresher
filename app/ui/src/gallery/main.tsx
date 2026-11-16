import '../design/index.css';

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { Gallery } from './gallery';
import { REGISTRY } from './registry';
import { storyId } from './story';

const root = document.querySelector('#root');

if (root === null) {
  throw new Error('The #root element is missing from gallery.html');
}

// The gallery spec reads the list of stories from the page.
Object.assign(window, { __stories__: REGISTRY.map((story) => storyId(story)) });

createRoot(root).render(
  <StrictMode>
    <Gallery stories={REGISTRY} />
  </StrictMode>,
);
