import { type JSX, type ReactNode, useMemo } from 'react';

import { type PathForm, PathFormContext } from '../design/data/path-text/path-form';
import { shortPath } from '../product/paths/short-path';
import { useCore } from '../state/store';
import { useAppInfo } from './use-app-info';

const asIs: PathForm = (path) => path;

// While a demo plays, every path on the page is written from the home folder as `~`, in its tooltip and accessible
// text too: the home of a demo is in the sandbox's folder, which a recording must never show (plan 038).
export const DemoPathForm = ({ children }: { readonly children: ReactNode }): JSX.Element => {
  const demo = useCore((state) => state.demo);
  const home = useAppInfo()?.home ?? null;
  const form = useMemo<PathForm>(() => (demo ? (path): string => shortPath(path, home) : asIs), [demo, home]);

  return <PathFormContext value={form}>{children}</PathFormContext>;
};
