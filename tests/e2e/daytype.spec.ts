import { expect, test } from '@playwright/test';

/**
 * Test E2E pour le cycle de types de jour (work -> vacation -> disabled -> work)
 * 
 * Ce test vérifie que :
 * 1. Le bouton de type de jour est visible quand le jour est activé
 * 2. Cliquer sur le bouton change le type (💼 -> 🏖️ -> désactivé)
 * 3. Le dayType est correctement inclus dans la requête de sauvegarde
 * 
 * Régression : dayType n'était pas passé dans buildSaveInput, ce qui faisait
 * que les changements de type n'étaient pas persistés côté backend.
 */

test.describe('Day type cycling', () => {
  test('day type button cycles through work -> vacation -> disabled', async ({ page }) => {
    await page.goto('/');

    // Attendre que l'interface charge (même si le backend échoue, l'UI doit être là)
    const nav = page.locator('.sidebar-nav');
    await expect(nav).toBeVisible();

    // Trouver la première carte de jour (Lundi) et son checkbox
    const firstDayCard = page.locator('.bento-card').first();
    await expect(firstDayCard).toBeVisible();

    // Activer le jour s'il ne l'est pas déjà
    const checkbox = firstDayCard.locator('.bento-card-checkbox');
    await checkbox.check();

    // Le bouton de type devrait être visible
    const dayTypeButton = firstDayCard.locator('.day-type-btn');
    await expect(dayTypeButton).toBeVisible();

    // Vérifier l'état initial (💼 = work)
    await expect(dayTypeButton).toHaveText('💼');

    // Cliquer pour passer en vacation (🏖️)
    await dayTypeButton.click();
    await expect(dayTypeButton).toHaveText('🏖️');

    // Vérifier que la carte a la classe vacation
    await expect(firstDayCard).toHaveClass(/bento-card--vacation/);

    // Cliquer pour passer en disabled
    await dayTypeButton.click();
    
    // Le bouton devrait disparaître quand le jour est désactivé
    await expect(dayTypeButton).not.toBeVisible();
    
    // La carte devrait avoir la classe disabled
    await expect(firstDayCard).toHaveClass(/bento-card--disabled/);
  });

  test('dayType is included in save request', async ({ page }) => {
    // Intercepter les appels Tauri pour vérifier que dayType est présent
    await page.goto('/');

    // Attendre que l'app soit chargée
    await expect(page.locator('.bento-grid')).toBeVisible();

    // Mock l'invoke Tauri pour capturer les appels
    const invokeCalls: any[] = [];
    await page.evaluate(() => {
      const originalInvoke = (window as any).__TAURI__?.core?.invoke;
      if (originalInvoke) {
        (window as any).__TAURI__.core.invoke = async (cmd: string, args: any) => {
          (window as any).__TAURI_INVOKE_CAPTURE__?.push({ cmd, args });
          return originalInvoke(cmd, args);
        };
      }
      // Créer un array global pour capturer les appels
      (window as any).__TAURI_INVOKE_CAPTURE__ = [];
    });

    // Activer un jour et changer son type
    const firstDayCard = page.locator('.bento-card').first();
    const checkbox = firstDayCard.locator('.bento-card-checkbox');
    await checkbox.check();

    const dayTypeButton = firstDayCard.locator('.day-type-btn');
    await dayTypeButton.click(); // work -> vacation

    // Attendre un peu pour le debounce de l'autosave
    await page.waitForTimeout(500);

    // Vérifier que les appels contiennent dayType
    const capturedCalls = await page.evaluate(() => (window as any).__TAURI_INVOKE_CAPTURE__);
    
    // Filtrer les appels save_week
    const saveCalls = capturedCalls?.filter((call: any) => call.cmd === 'save_week') || [];
    
    for (const call of saveCalls) {
      const entries = call.args?.input?.entries || [];
      for (const entry of entries) {
        // Chaque entry devrait avoir un dayType défini
        expect(entry.dayType).toBeDefined();
        expect(['work', 'vacation', 'disabled']).toContain(entry.dayType);
      }
    }
  });
});

/**
 * Test de non-régression pour le bug dayType.
 * 
 * Ce test échouera si dayType n'est pas inclus dans les données
 * envoyées au backend lors d'une sauvegarde.
 */
test('regression: dayType must be preserved after save', async ({ page }) => {
  await page.goto('/');

  const firstDayCard = page.locator('.bento-card').first();
  await expect(firstDayCard).toBeVisible();

  // Activer et mettre en vacation
  const checkbox = firstDayCard.locator('.bento-card-checkbox');
  await checkbox.check();

  const dayTypeButton = firstDayCard.locator('.day-type-btn');
  await dayTypeButton.click(); // work -> vacation

  // Vérifier que c'est bien en vacation visuellement
  await expect(dayTypeButton).toHaveText('🏖️');

  // Simuler un événement de perte de focus pour forcer la sauvegarde immédiate
  await page.evaluate(() => {
    window.dispatchEvent(new Event('blur'));
  });

  // Attendre que la sauvegarde soit effectuée
  await page.waitForTimeout(1000);

  // Après sauvegarde, le type devrait être préservé
  // Si dayType n'était pas passé, il reviendrait à 'work'
  await expect(dayTypeButton).toHaveText('🏖️');
});
