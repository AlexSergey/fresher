import { describe, expect, it } from 'vitest';

import type { ModuleInfo } from '../../ipc/bindings.generated';

import { DEMO_MODULES } from '../../test/demo-scan';
import { moduleProgress } from './module-progress';

const LOGS: ModuleInfo = { categories: [], id: 'logs', settings: ['logs.enabled'] };

describe('moduleProgress', () => {
  describe('negative cases', () => {
    it('has no chips before the core lists its modules', () => {
      expect(moduleProgress(null, null, [])).toStrictEqual([]);
    });

    it('a module the scan does not run has no chip', () => {
      expect(moduleProgress(['demo'], [...DEMO_MODULES, LOGS], [])).toStrictEqual([
        { module: 'demo', state: { kind: 'running' } },
      ]);
    });
  });

  describe('positive cases', () => {
    it('a module still running shows running, a finished one its outcome', () => {
      const finished = [{ module: 'logs', outcome: { kind: 'completed' } }] as const;

      expect(moduleProgress(['demo', 'logs'], [...DEMO_MODULES], finished)).toStrictEqual([
        { module: 'demo', state: { kind: 'running' } },
        { module: 'logs', state: { kind: 'completed' } },
      ]);
    });

    it('a scan learned of from an overview only shows every module the core lists', () => {
      expect(moduleProgress(null, [...DEMO_MODULES, LOGS], [])).toStrictEqual([
        { module: 'demo', state: { kind: 'running' } },
        { module: 'logs', state: { kind: 'running' } },
      ]);
    });
  });
});
