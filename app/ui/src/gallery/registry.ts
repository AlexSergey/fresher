import { type Story, storyId } from './story';

type StoriesModule = { readonly stories?: readonly Story[] };

// Every story of the given `*.stories.tsx` modules, sorted by component and state. A module without its `stories`
// export, or two stories with the same component and state, are refused.
export const collectStories = (modules: Readonly<Record<string, StoriesModule>>): readonly Story[] => {
  const seen = new Set<string>();
  const all = Object.entries(modules).flatMap(([file, module]) => {
    if (module.stories === undefined) {
      throw new Error(`${file} has no \`export const stories\``);
    }

    return module.stories;
  });

  for (const story of all) {
    const id = storyId(story);

    if (seen.has(id)) {
      throw new Error(`Two stories are named ${id}`);
    }

    seen.add(id);
  }

  return all.toSorted(
    (first, second) => first.component.localeCompare(second.component) || first.state.localeCompare(second.state),
  );
};

// Every story of the app, found by its file name: adding a component edits no shared file.
export const REGISTRY: readonly Story[] = collectStories(
  import.meta.glob<StoriesModule>('../**/*.stories.tsx', { eager: true }),
);
