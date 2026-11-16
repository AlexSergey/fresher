import { type CSSProperties, type JSX, useSyncExternalStore } from 'react';

import styles from './gallery.module.css';
import { type Story, storyId } from './story';

type Route =
  | { readonly component: string; readonly kind: 'component' }
  | { readonly kind: 'list' }
  | { readonly kind: 'story'; readonly story: Story };

const decode = (part: string): null | string => {
  try {
    return decodeURIComponent(part);
  } catch {
    return null;
  }
};

// `#/<component>` shows one component, `#/<component>/<state>` one story alone; anything else shows the list.
const routeOf = (hash: string, stories: readonly Story[]): Route => {
  const [component = null, state = null] = hash.replace(/^#\/?/u, '').split('/').map(decode);
  const story = stories.find((candidate) => candidate.component === component && candidate.state === state);

  if (story !== undefined) {
    return { kind: 'story', story };
  }

  if (state === null && component !== null && stories.some((candidate) => candidate.component === component)) {
    return { component, kind: 'component' };
  }

  return { kind: 'list' };
};

const hrefOf = (...parts: readonly string[]): string => `#/${parts.map((part) => encodeURIComponent(part)).join('/')}`;

const subscribe = (onChange: () => void): (() => void) => {
  window.addEventListener('hashchange', onChange);

  return () => {
    window.removeEventListener('hashchange', onChange);
  };
};

const readHash = (): string => window.location.hash;

const StoryFrame = ({ story }: { readonly story: Story }): JSX.Element => (
  <figure
    aria-label={storyId(story)}
    className={`${styles.frame ?? ''} ${styles[story.surface ?? 'window'] ?? ''}`}
    data-story={storyId(story)}
    style={story.width === undefined ? undefined : ({ '--story-width': `${story.width}px` } as CSSProperties)}
  >
    {story.render()}
  </figure>
);

// The dev-only gallery: every component in every state, on the design's base.
export const Gallery = ({ stories }: { readonly stories: readonly Story[] }): JSX.Element => {
  const route = routeOf(useSyncExternalStore(subscribe, readHash), stories);

  if (route.kind === 'story') {
    return (
      <main className={styles.alone}>
        <StoryFrame story={route.story} />
      </main>
    );
  }

  const components = [...new Set(stories.map(({ component }) => component))];
  const selected = route.kind === 'component' ? route.component : null;

  return (
    <div className={styles.gallery}>
      <nav aria-label="Components" className={styles.sidebar}>
        <ul className={styles.components}>
          {components.map((component) => (
            <li key={component}>
              <a
                aria-current={component === selected ? 'page' : undefined}
                className={styles.link}
                href={hrefOf(component)}
              >
                {component}
              </a>
            </li>
          ))}
        </ul>
      </nav>
      <main className={styles.pane}>
        {selected === null ? (
          <p className={styles.hint}>Pick a component to see its states.</p>
        ) : (
          stories
            .filter(({ component }) => component === selected)
            .map((story) => (
              <section aria-label={story.state} className={styles.state} key={story.state}>
                <h2 className={styles.title}>
                  <a className={styles.link} href={hrefOf(story.component, story.state)}>
                    {story.state}
                  </a>
                </h2>
                <StoryFrame story={story} />
              </section>
            ))
        )}
      </main>
    </div>
  );
};
