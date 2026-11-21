import { describe, expect, it } from 'vitest';

import type { ModuleInfo } from '../../ipc/bindings.generated';

import { DEMO_MODULES } from '../../test/demo-scan';
import { moduleProgress } from './module-progress';

const LOGS: ModuleInfo = { categories: [], choices: [], group: 'development', id: 'logs', settings: ['logs.enabled'] };

describe('moduleProgress', () => {
  describe('negative cases', () => {
    it('has no chips before the core lists its modules', () => {
      expect(moduleProgress(null, null, [])).toStrictEqual([]);
    });

    it('a module the scan does not run has no chip', () => {
      expect(moduleProgress(['demo'], [...DEMO_MODULES, LOGS], [])).toStrictEqual([
        { group: 'debug', module: 'demo', state: { kind: 'running' } },
      ]);
    });
  });

  describe('positive cases', () => {
    it('a chip names the group of its module', () => {
      expect(moduleProgress(null, [...DEMO_MODULES, LOGS], []).map(({ group }) => group)).toStrictEqual([
        'debug',
        'development',
      ]);
    });

    it('a module still running shows running, a finished one its outcome', () => {
      const finished = [{ group: 'development', module: 'logs', outcome: { kind: 'completed' } }] as const;

      expect(moduleProgress(['demo', 'logs'], [...DEMO_MODULES, LOGS], finished)).toStrictEqual([
        { group: 'debug', module: 'demo', state: { kind: 'running' } },
        { group: 'development', module: 'logs', state: { kind: 'completed' } },
      ]);
    });

    it('a scan learned of from an overview only shows every module the core lists', () => {
      expect(moduleProgress(null, [...DEMO_MODULES, LOGS], [])).toStrictEqual([
        { group: 'debug', module: 'demo', state: { kind: 'running' } },
        { group: 'development', module: 'logs', state: { kind: 'running' } },
      ]);
    });
  });
});
