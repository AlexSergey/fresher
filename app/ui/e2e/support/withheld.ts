import type { CategoryId, CategoryTotal, Event, ModuleInfo, Overview, Sizes } from '../../src/ipc/bindings.generated';
import type { CoreScript } from '../core-mock';

// What a core that withholds High produces from the same home (mirrors the engine's rule, docs/restrictions): no finding
// whose own level or whose category is High, no total, found size or left-out of those, no High category in a module,
// no insight part at High, no Everything choice on an app's page, no part at High in an Apps row nor its bytes, and
// `high: 'withheld'` in the app info.

const lessBy = (sizes: Sizes, gone: readonly Sizes[]): Sizes => ({
  ...sizes,
  allocated: gone.reduce((left, one) => left - one.allocated, sizes.allocated),
  logical: gone.reduce((left, one) => left - one.logical, sizes.logical),
  reclaimable: gone.reduce((left, one) => left - one.reclaimable, sizes.reclaimable),
});

const withheldOverview = (overview: Overview): Overview => {
  const gone = overview.totals.filter(({ highestDanger }) => highestDanger === 'high');

  return {
    ...overview,
    found: lessBy(
      overview.found,
      gone.map(({ sizes }) => sizes),
    ),
    totals: overview.totals.filter((total: CategoryTotal) => !gone.includes(total)),
  };
};

const withheldModules = (modules: unknown): unknown =>
  Array.isArray(modules)
    ? (modules as readonly ModuleInfo[]).map((module) => ({
        ...module,
        categories: module.categories.filter(({ descriptor }) => descriptor.danger !== 'high'),
      }))
    : modules;

export const withheld = (script: CoreScript): CoreScript => {
  const highCategories = new Set<CategoryId>(
    Array.isArray(script.answers?.getModules)
      ? (script.answers.getModules as readonly ModuleInfo[]).flatMap(({ categories }) =>
          categories.filter(({ descriptor }) => descriptor.danger === 'high').map(({ descriptor }) => descriptor.id),
        )
      : [],
  );
  const withheldEvent = (event: Event): Event => {
    if (event.type === 'findings-added') {
      return {
        ...event,
        findings: event.findings.filter(({ category, danger }) => danger !== 'high' && !highCategories.has(category)),
      };
    }

    return event.type === 'module-finished'
      ? {
          ...event,
          insights: event.insights.map((insight) => ({
            ...insight,
            parts: insight.parts.filter(({ danger }) => danger !== 'high'),
          })),
          leftOut: event.leftOut.filter(({ category }) => !highCategories.has(category)),
        }
      : event;
  };

  const { answers, apps, scan, scanned, scanning, stop } = script;

  return {
    ...script,
    ...(answers === undefined ? {} : { answers: { ...answers, getModules: withheldModules(answers.getModules) } }),
    ...(apps === undefined ? {} : { apps: { ...apps, withheld: true } }),
    high: 'withheld',
    ...(scan === undefined ? {} : { scan: scan.map(withheldEvent) }),
    ...(scanned === undefined ? {} : { scanned: withheldOverview(scanned) }),
    ...(scanning === undefined ? {} : { scanning: withheldOverview(scanning) }),
    ...(stop === undefined
      ? {}
      : { stop: { events: stop.events.map(withheldEvent), overview: withheldOverview(stop.overview) } }),
  };
};
