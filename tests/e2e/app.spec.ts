import { expect, test } from '@playwright/test';

/**
 * Smoke test de l'UI (mode dev web, sans runtime Tauri).
 * Sans backend Tauri, invoke() échoue : l'app doit quand même
 * démarrer et afficher la coquille + la navigation.
 */

test('app boots to the shell with sidebar navigation', async ({ page }) => {
  await page.goto('/');

  await expect(page).toHaveTitle('Timetable Desktop');

  // La navigation latérale rend les 4 onglets
  const nav = page.locator('.sidebar-nav');
  await expect(nav).toBeVisible();
  for (const label of ['Feuille de temps', 'Historique', 'Analytiques', 'Configuration']) {
    await expect(nav.getByText(label)).toBeVisible();
  }
});

test('shows the error panel when the Tauri backend is unavailable', async ({ page }) => {
  await page.goto('/');

  // Sans runtime Tauri, le bootstrap échoue et l'alerte d'erreur s'affiche
  const alert = page.locator('.alert');
  await expect(alert).toBeVisible();
});
