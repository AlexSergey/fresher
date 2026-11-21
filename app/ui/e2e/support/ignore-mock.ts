import type { Page } from '@playwright/test';

import type {
  Finding,
  HiddenModule,
  IgnoredEntry,
  IgnoreTarget,
  ModuleInfo,
  Overview,
  SettingsView,
} from '../../src/ipc/bindings.generated';

import { COMMANDS } from '../../src/ipc';
import { CLEANUP_HOOKS } from './cleanup-mock';

// The ignore commands of the mocked core: the module, group and path lists kept in the tab's session storage (so a reload of
// the page keeps them), the settings and the overview answered with them, `setting-changed` sent after each change.
export const IGNORE_HOOK = '__fresherIgnoreInvoke';

// The page's name of the function that answers the ignored paths, for `get_findings` to leave them out.
export const IGNORED_PATHS_HOOK = '__fresherIgnoredPaths';

const STORE = 'fresher.mock.ignore';

type Lists = { categories: string[]; groups: string[]; modules: string[]; paths: string[] };

type Setup = {
  readonly commands: typeof COMMANDS;
  // The path of every finding that has one, by its id: what an Ignore of a finding records.
  readonly findingPaths: Readonly<Record<number, string>>;
  readonly hook: string;
  readonly pathsHook: string;
  readonly send: string;
  readonly store: string;
};

// Adds the ignore commands to the mocked core of `core-mock.ts`, which calls the hook with the answers it holds.
export const mockIgnore = async (page: Page, findings: readonly Finding[]): Promise<void> => {
  const findingPaths = Object.fromEntries(
    findings.flatMap((finding) => (finding.subject.kind === 'path' ? [[finding.id, finding.subject.value]] : [])),
  );

  await page.addInitScript(
    (setup: Setup) => {
      const scope = globalThis as unknown as Record<string, unknown>;

      const load = (): Lists => {
        try {
          const saved = JSON.parse(sessionStorage.getItem(setup.store) ?? 'null') as null | Partial<Lists>;

          return {
            categories: saved?.categories ?? [],
            groups: saved?.groups ?? [],
            modules: saved?.modules ?? [],
            paths: saved?.paths ?? [],
          };
        } catch {
          return { categories: [], groups: [], modules: [], paths: [] };
        }
      };
      const lists = load();
      const save = (): void => {
        try {
          sessionStorage.setItem(setup.store, JSON.stringify(lists));
        } catch {
          // The page keeps the lists in memory.
        }
      };
      const changed = (key: string, kind: 'ids' | 'paths', value: readonly string[]): void => {
        const send = scope[setup.send] as ((events: readonly unknown[]) => void) | undefined;

        send?.([{ key, type: 'setting-changed', value: { kind, value } }]);
      };
      const toggle = (list: string[], key: string, id: string, on: boolean, kind: 'ids' | 'paths' = 'ids'): void => {
        const next = on ? [...list.filter((one) => one !== id), id] : list.filter((one) => one !== id);

        list.splice(0, list.length, ...next);
        save();
        changed(key, kind, list);
      };
      scope[setup.pathsHook] = (): readonly string[] => lists.paths;

      const settingsOf = (view: SettingsView): SettingsView => ({
        ...view,
        values: {
          ...view.values,
          'core.ignore.categories': { kind: 'ids', value: lists.categories },
          'core.ignore.groups': { kind: 'ids', value: lists.groups },
          'core.ignore.modules': { kind: 'ids', value: lists.modules },
          'core.ignore.paths': { kind: 'paths', value: lists.paths },
        },
      });

      const overviewOf = (base: Overview, modules: readonly ModuleInfo[]): Overview => {
        const hiddenOf = (module: ModuleInfo): HiddenModule[] => {
          if (lists.groups.includes(module.group)) {
            return [{ by: 'group', module: module.id }];
          }

          return lists.modules.includes(module.id) ? [{ by: 'module', module: module.id }] : [];
        };
        const hidden = modules.flatMap(hiddenOf);
        const totals = base.totals.filter(
          ({ category }) =>
            !lists.categories.includes(category) && !hidden.some(({ module }) => category.startsWith(`${module}.`)),
        );
        const sum = (field: 'allocated' | 'logical' | 'reclaimable'): number =>
          totals.reduce((bytes, total) => bytes + total.sizes[field], 0);

        return {
          ...base,
          found: {
            ...base.found,
            allocated: sum('allocated'),
            logical: sum('logical'),
            reclaimable: sum('reclaimable'),
          },
          hiddenModules: hidden,
          ignored: {
            ...base.ignored,
            categories: lists.categories.length,
            groups: lists.groups.length,
            modules: lists.modules.length,
            paths: lists.paths.length,
          },
          totals,
        };
      };

      const change = (entry: IgnoredEntry | IgnoreTarget, on: boolean): void => {
        if (entry.kind === 'category') {
          toggle(lists.categories, 'core.ignore.categories', entry.category, on);
        } else if (entry.kind === 'group') {
          toggle(lists.groups, 'core.ignore.groups', entry.group, on);
        } else if (entry.kind === 'module') {
          toggle(lists.modules, 'core.ignore.modules', entry.module, on);
        } else if (entry.kind === 'path') {
          toggle(lists.paths, 'core.ignore.paths', entry.path, on, 'paths');
        } else {
          const path = setup.findingPaths[entry.finding];

          if (path !== undefined) {
            toggle(lists.paths, 'core.ignore.paths', path, on, 'paths');
          }
        }
      };

      scope[setup.hook] = (
        command: string,
        args: Record<string, unknown>,
        answers: Record<string, unknown>,
      ): Promise<unknown> | undefined => {
        switch (command) {
          case setup.commands.getOverview:
            return Promise.resolve(
              overviewOf(answers[command] as Overview, answers[setup.commands.getModules] as ModuleInfo[]),
            );
          case setup.commands.getSettings:
            return Promise.resolve(settingsOf(answers[command] as SettingsView));
          case setup.commands.ignore:
            change(args.target as IgnoreTarget, true);

            return Promise.resolve(null);
          case setup.commands.restoreIgnored:
            change(args.entry as IgnoredEntry, false);

            return Promise.resolve(null);
          default:
            return undefined;
        }
      };
    },
    {
      commands: COMMANDS,
      findingPaths,
      hook: IGNORE_HOOK,
      pathsHook: IGNORED_PATHS_HOOK,
      send: CLEANUP_HOOKS.send,
      store: STORE,
    } satisfies Setup,
  );
};
