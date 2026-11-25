// A path as the user reads it: the home folder (`home`, as the core reports it) shown as `~`. Only a whole folder
// name is replaced: with the home `/Users/a`, `/Users/ab/x` stays as it is. Without a home, the path is unchanged.
export const shortPath = (path: string, home: null | string): string => {
  if (home === null || home === '') {
    return path;
  }

  const root = home.endsWith('/') ? home.slice(0, -1) : home;

  if (root === '' || !path.startsWith(root)) {
    return path;
  }

  const rest = path.slice(root.length);

  return rest === '' || rest.startsWith('/') ? `~${rest}` : path;
};
