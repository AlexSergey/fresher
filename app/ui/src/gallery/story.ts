import type { JSX } from 'react';

// One state of one component in the gallery, photographed by the gallery spec.
export type Story = {
  readonly component: string;
  readonly render: () => JSX.Element;
  readonly state: string;
  readonly surface?: Surface;
  // Pixels; the story's own width otherwise.
  readonly width?: number;
};

type StateDefinition = (() => JSX.Element) | (StoryOptions & { readonly render: () => JSX.Element });

type StoryOptions = Pick<Story, 'surface' | 'width'>;

// The surface a story is shown on, as in the app.
type Surface = 'overlay' | 'raised' | 'window';

// The stories of one component, by state; a `*.stories.tsx` file exports them as `export const stories`.
export const stories = (component: string, states: Readonly<Record<string, StateDefinition>>): readonly Story[] =>
  Object.entries(states).map(([state, definition]) =>
    typeof definition === 'function' ? { component, render: definition, state } : { component, state, ...definition },
  );

// The id of a story: its `data-story` attribute, its hash and its screenshot's name.
export const storyId = ({ component, state }: Pick<Story, 'component' | 'state'>): string => `${component}/${state}`;
