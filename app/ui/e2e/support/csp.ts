import { expect, type Page } from '@playwright/test';

type Watched = { readonly __cspViolations__?: readonly string[] };

// Records every `securitypolicyviolation` of the page from its first script on: the e2e build is served with the app's
// production policy (`style-src 'self'`: no <style> element, no style attribute), so a breach shows up here.
export const failOnCspViolation = async (page: Page): Promise<void> => {
  await page.addInitScript(() => {
    const violations: string[] = [];

    Object.assign(window, { __cspViolations__: violations });
    document.addEventListener('securitypolicyviolation', (event) => {
      violations.push(`${event.violatedDirective} blocked ${event.blockedURI || 'inline'}`);
    });
  });
};

// Fails the test when the page broke its content security policy. A visual spec calls it before its screenshot:
// in WebKit the policy also blocks the <style> Playwright injects to take one.
export const expectNoCspViolation = async (page: Page): Promise<void> => {
  const violations = await page.evaluate(() => (window as Watched).__cspViolations__ ?? []);

  expect(violations, 'the page broke its content security policy').toStrictEqual([]);
};
