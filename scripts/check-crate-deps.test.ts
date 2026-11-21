import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
  findModuleViolations,
  findUnsafeViolations,
  findViolations,
  type MetadataDependency,
  type MetadataPackage,
  type MetadataTarget,
  type SourceFile,
  type Violation,
} from './check-crate-deps.ts';

const crate = (name: string, dependencies: readonly MetadataDependency[] = []): MetadataPackage => ({
  dependencies,
  name,
  targets: [],
});

const withTargets = (name: string, targets: readonly MetadataTarget[]): MetadataPackage => ({
  dependencies: [],
  name,
  targets,
});

const target = (kind: string, path: string): MetadataTarget => ({ kind: [kind], path });

const FORBIDDING = '//! A crate.\n\n#![forbid(unsafe_code)]\n\nmod a;\n';

// Every source forbids unsafe code but the ones named in `without`.
const sources =
  (...without: readonly string[]) =>
  (path: string): string =>
    without.includes(path) ? '//! A crate.\n\nmod a;\n' : FORBIDDING;

const normal = (name: string): MetadataDependency => ({ kind: null, name });
const dev = (name: string): MetadataDependency => ({ kind: 'dev', name });

const LIBRARIES = ['fresher-core', 'fresher-macos', 'fresher-rules', 'fresher-store', 'fresher-engine'].map((name) =>
  crate(name),
);

const check = (...packages: readonly MetadataPackage[]): Violation[] =>
  findViolations({ packages: [...LIBRARIES, ...packages] });

describe('findViolations', () => {
  describe('negative cases', () => {
    it('fails on a module depending on another module', () => {
      const violations = check(
        crate('fresher-module-demo'),
        crate('fresher-module-docker', [normal('fresher-core'), normal('fresher-module-demo')]),
      );

      assert.deepEqual(violations, [
        { crate: 'fresher-module-docker', reason: 'a module never depends on another module (fresher-module-demo)' },
      ]);
    });

    it('fails on a module depending on anything but the core and the rules', () => {
      assert.equal(check(crate('fresher-module-demo', [normal('fresher-store')])).length, 1);
    });

    it('fails on the engine depending on the store', () => {
      const violations = findViolations({
        packages: [crate('fresher-core'), crate('fresher-store'), crate('fresher-engine', [normal('fresher-store')])],
      });

      assert.equal(violations.length, 1);
      assert.match(violations[0]?.reason ?? '', /normal dependency on fresher-store/);
    });

    it('fails on the core depending on anything in the workspace', () => {
      const violations = findViolations({
        packages: [
          crate('fresher-macos'),
          crate('fresher-core', [normal('fresher-macos'), { kind: 'build', name: 'fresher-macos' }]),
        ],
      });

      assert.equal(violations.length, 2);
    });

    it('fails on an unknown workspace crate', () => {
      const violations = check(crate('fresher-misc'));

      assert.equal(violations.length, 1);
      assert.match(violations[0]?.reason ?? '', /not in the dependency table/);
    });

    it('fails on a dev-dependency outside the table', () => {
      const violations = check(crate('fresher-store', [dev('fresher-engine')]));

      assert.equal(violations.length, 1);
      assert.match(violations[0]?.reason ?? '', /dev dependency on fresher-engine/);
    });

    it('fails on fresher-testing as a normal dependency of a library', () => {
      assert.equal(check(crate('fresher-testing'), crate('fresher-rules', [normal('fresher-testing')])).length, 1);
    });

    it('fails on fresher-sandbox as a dependency of anything but the app and the tests', () => {
      const violations = check(
        crate('fresher-sandbox'),
        crate('fresher-cleaner', [normal('fresher-sandbox')]),
        crate('fresher-module-demo', [dev('fresher-sandbox')]),
      );

      assert.equal(violations.length, 2);
    });

    it('fails on fresher-sandbox depending on the engine', () => {
      assert.equal(check(crate('fresher-sandbox', [normal('fresher-engine')])).length, 1);
    });
  });

  describe('positive cases', () => {
    it('passes a workspace shaped like the current one', () => {
      assert.deepEqual(
        findViolations({
          packages: [
            crate('fresher-app', [normal('tauri'), { kind: 'build', name: 'tauri-build' }]),
            crate('fresher-cleaner'),
            crate('fresher-core', [normal('serde'), dev('proptest')]),
            crate('fresher-e2e', [dev('fresher-testing')]),
            crate('fresher-testing', [normal('fresher-core'), normal('serde')]),
            crate('fresher-rules'),
            crate('fresher-scanner'),
            crate('fresher-store'),
          ],
        }),
        [],
      );
    });

    it('passes every edge of the table', () => {
      assert.deepEqual(
        check(
          crate('fresher-cleaner', [normal('fresher-core'), normal('fresher-macos')]),
          crate('fresher-scanner', [normal('fresher-core'), normal('fresher-macos')]),
          crate('fresher-macos', [normal('fresher-core')]),
          crate('fresher-module-demo', [normal('fresher-core'), normal('fresher-rules')]),
        ),
        [],
      );
    });

    it('allows fresher-testing as a dev-dependency anywhere', () => {
      assert.deepEqual(
        check(
          crate('fresher-testing', [normal('fresher-core')]),
          crate('fresher-module-demo', [dev('fresher-testing')]),
        ),
        [],
      );
    });

    it('allows the app and e2e to depend on everything', () => {
      const everything = ['fresher-core', 'fresher-store', 'fresher-engine', 'fresher-module-demo'].map(normal);

      assert.deepEqual(
        check(
          crate('fresher-module-demo'),
          crate('fresher-app', everything),
          crate(
            'fresher-e2e',
            everything.map((d) => dev(d.name)),
          ),
        ),
        [],
      );
    });

    it('allows fresher-sandbox its edges, and the app and e2e to depend on it', () => {
      assert.deepEqual(
        check(
          crate('fresher-store', [normal('fresher-core')]),
          crate('fresher-testing', [normal('fresher-core')]),
          crate('fresher-sandbox', [
            normal('fresher-core'),
            normal('fresher-macos'),
            normal('fresher-store'),
            normal('fresher-testing'),
          ]),
          crate('fresher-app', [normal('fresher-sandbox')]),
          crate('fresher-e2e', [dev('fresher-sandbox')]),
        ),
        [],
      );
    });

    it('ignores dependencies outside the workspace', () => {
      assert.deepEqual(check(crate('fresher-module-demo', [normal('serde'), dev('tempfile')])), []);
    });
  });
});

describe('findUnsafeViolations', () => {
  describe('negative cases', () => {
    it('fails on a crate root without #![forbid(unsafe_code)]', () => {
      const violations = findUnsafeViolations(
        { packages: [withTargets('fresher-store', [target('lib', 'crates/store/src/lib.rs')])] },
        sources('crates/store/src/lib.rs'),
      );

      assert.deepEqual(violations, [
        { crate: 'fresher-store', reason: 'the crate root crates/store/src/lib.rs lacks #![forbid(unsafe_code)]' },
      ]);
    });

    it('fails on each root of a crate with a library and a binary', () => {
      const app = withTargets('fresher-app', [
        target('staticlib', 'app/tauri/src/lib.rs'),
        target('bin', 'app/tauri/src/main.rs'),
      ]);

      assert.equal(findUnsafeViolations({ packages: [app] }, sources('app/tauri/src/main.rs')).length, 1);
    });

    it('does not take the line in a comment for the attribute', () => {
      const commented = (): string => '// #![forbid(unsafe_code)] is missing\nmod a;\n';

      assert.equal(
        findUnsafeViolations({ packages: [withTargets('fresher-core', [target('lib', 'core.rs')])] }, commented).length,
        1,
      );
    });
  });

  describe('positive cases', () => {
    it('passes fresher-macos without the line', () => {
      const macos = withTargets('fresher-macos', [target('lib', 'crates/macos/src/lib.rs')]);

      assert.deepEqual(findUnsafeViolations({ packages: [macos] }, sources('crates/macos/src/lib.rs')), []);
    });

    it('passes crate roots that forbid unsafe code', () => {
      const packages = [
        withTargets('fresher-core', [target('lib', 'crates/core/src/lib.rs')]),
        withTargets('fresher-app', [target('rlib', 'app/tauri/src/lib.rs'), target('bin', 'app/tauri/src/main.rs')]),
      ];

      assert.deepEqual(findUnsafeViolations({ packages }, sources()), []);
    });

    it('ignores tests, benches, examples and build scripts', () => {
      const store = withTargets('fresher-store', [
        target('lib', 'crates/store/src/lib.rs'),
        target('test', 'crates/store/tests/files.rs'),
        target('bench', 'crates/store/benches/save.rs'),
        target('example', 'crates/store/examples/demo.rs'),
        target('custom-build', 'crates/store/build.rs'),
      ]);

      assert.deepEqual(
        findUnsafeViolations(
          { packages: [store] },
          sources(
            'crates/store/tests/files.rs',
            'crates/store/benches/save.rs',
            'crates/store/examples/demo.rs',
            'crates/store/build.rs',
          ),
        ),
        [],
      );
    });
  });
});

const moduleWith = (...files: readonly SourceFile[]): Violation[] =>
  findModuleViolations(
    {
      packages: [
        withTargets('fresher-module-demo', [target('lib', '/m/demo/src/lib.rs')]),
        withTargets('fresher-core', [target('lib', '/c/src/lib.rs')]),
      ],
    },
    (folder) => (folder === '/m/demo/src' ? files : [{ path: '/c/src/lib.rs', text: 'use std::process::Command;' }]),
  );

describe('findModuleViolations', () => {
  describe('negative cases', () => {
    it('fails on a module naming std::process', () => {
      const text = 'fn run() { std::process::Command::new("x"); }';

      assert.deepEqual(moduleWith({ path: '/m/demo/src/tool.rs', text }), [
        { crate: 'fresher-module-demo', reason: '/m/demo/src/tool.rs names std::process or std::env' },
      ]);
    });

    it('fails on a module naming std::env', () => {
      assert.equal(moduleWith({ path: '/m/demo/src/a.rs', text: 'let v = std::env::var("HOME");' }).length, 1);
    });

    it('fails on a use that groups std::env or std::process', () => {
      assert.equal(moduleWith({ path: '/m/demo/src/a.rs', text: 'use std::{env, fmt};' }).length, 1);
      assert.equal(moduleWith({ path: '/m/demo/src/b.rs', text: 'use std::{fmt, process::Command};' }).length, 1);
    });

    it('fails on a multi-line and on a nested group', () => {
      assert.equal(moduleWith({ path: '/m/demo/src/a.rs', text: 'use std::{\n    env,\n    fmt,\n};' }).length, 1);
      assert.equal(moduleWith({ path: '/m/demo/src/b.rs', text: 'use std::{path::{Path}, env};' }).length, 1);
    });

    it('reports a violation once for a module with a lib and a bin in one folder', () => {
      const both = findModuleViolations(
        {
          packages: [
            withTargets('fresher-module-demo', [
              target('lib', '/m/demo/src/lib.rs'),
              target('bin', '/m/demo/src/main.rs'),
            ]),
          ],
        },
        () => [{ path: '/m/demo/src/a.rs', text: 'std::env::args();' }],
      );

      assert.equal(both.length, 1);
    });
  });

  describe('positive cases', () => {
    it('ignores a block comment naming std::process', () => {
      assert.deepEqual(
        moduleWith({ path: '/m/demo/src/a.rs', text: '/* std::process::Command\n std::env */ fn a() {}' }),
        [],
      );
    });

    it('passes a module that names neither, and ignores comments and other crates', () => {
      assert.deepEqual(
        moduleWith(
          { path: '/m/demo/src/a.rs', text: '// never std::process::Command\nuse std::path::Path;\nuse std::fmt;' },
          { path: '/m/demo/src/b.rs', text: 'let environment = 1; // std::envelope' },
        ),
        [],
      );
    });
  });
});
