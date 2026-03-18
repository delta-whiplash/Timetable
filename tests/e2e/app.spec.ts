import { expect, test } from '@playwright/test';

/**
 * Tests E2E pour Timetable Desktop
 *
 * Scénarios couverts:
 * 1. Saisie d'une semaine et vérification des calculs
 * 2. Changement de thème (clair/sombre)
 * 3. Navigation entre semaines
 */

test.describe('Timetable Desktop - Scénarios Principaux', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('1. Saisie d\'une semaine et vérification des calculs', async ({ page }) => {
    // Attendre que l'application soit chargée
    await expect(page.getByText('Application de calcul d\'heures Windows')).toBeVisible();
    await expect(page.getByText('Horaires hebdomadaires')).toBeVisible();

    // Vérifier que les 7 jours sont présents (Lundi à Dimanche)
    const dayLabels = ['Lundi', 'Mardi', 'Mercredi', 'Jeudi', 'Vendredi', 'Samedi', 'Dimanche'];
    for (const label of dayLabels) {
      await expect(page.getByText(label).first()).toBeVisible();
    }

    // Saisir des heures pour Lundi (jour 0)
    // 09:00 à 17:00 avec 60 min de pause = 7h travaillées = 420 minutes
    const lundiSection = page.locator('.day-card').filter({ hasText: 'Lundi' });

    await lundiSection.getByLabel('Début').fill('09:00');
    await lundiSection.getByLabel('Fin').fill('17:00');
    await lundiSection.getByLabel('Pause').fill('01:00');

    // Attendre la mise à jour (auto-save)
    await page.waitForTimeout(500);

    // Vérifier que le total du jour est affiché
    await expect(lundiSection.getByText('07h 00')).toBeVisible();

    // Saisir des heures pour Mardi (jour 1)
    // 08:30 à 16:30 avec 45 min de pause = 7h45 travaillées = 465 minutes
    const mardiSection = page.locator('.day-card').filter({ hasText: 'Mardi' });

    await mardiSection.getByLabel('Début').fill('08:30');
    await mardiSection.getByLabel('Fin').fill('16:30');
    await mardiSection.getByLabel('Pause').fill('00:45');
    await page.waitForTimeout(500);

    // Vérifier le total affiché pour Mardi
    await expect(mardiSection.getByText('07h 45')).toBeVisible();

    // Vérifier le résumé hebdomadaire
    // Total: 420 + 465 = 885 minutes = 14h 45
    const summarySection = page.locator('.summary-cards, section[class*="summary"]');
    await expect(summarySection.getByText(/14h/)).toBeVisible();
  });

  test('2. Changement de thème (clair vers sombre)', async ({ page }) => {
    // Attendre le chargement
    await expect(page.getByText('Paramètres')).toBeVisible();

    // Vérifier le thème initial (clair par défaut)
    const htmlElement = page.locator('html');
    await expect(htmlElement).toHaveAttribute('data-theme', 'light');

    // Cliquer sur le bouton thème sombre
    const settingsPanel = page.locator('section[aria-label="Paramètres"]');
    await settingsPanel.getByRole('button', { name: 'Sombre' }).click();

    // Vérifier que le thème a changé
    await expect(htmlElement).toHaveAttribute('data-theme', 'dark');

    // Revenir au thème clair
    await settingsPanel.getByRole('button', { name: 'Clair' }).click();

    // Vérifier le retour au thème clair
    await expect(htmlElement).toHaveAttribute('data-theme', 'light');
  });

  test('3. Navigation entre semaines via le sélecteur de date', async ({ page }) => {
    // Attendre le chargement
    await expect(page.getByText('Semaine')).toBeVisible();

    // Récupérer la date actuelle affichée dans le sélecteur
    const weekInput = page.getByLabel('Semaine').locator('input[type="date"]');
    const initialDate = await weekInput.inputValue();

    // Sélectionner une semaine différente (semaine précédente)
    const previousWeek = new Date();
    previousWeek.setDate(previousWeek.getDate() - 7);
    const previousWeekStr = previousWeek.toISOString().split('T')[0];

    await weekInput.fill(previousWeekStr);
    await weekInput.press('Enter'); // Déclencher le changement
    await page.waitForTimeout(1000); // Attendre le chargement

    // Vérifier que la valeur a changé
    const newDate = await weekInput.inputValue();
    expect(newDate).not.toBe(initialDate);

    // Revenir à la semaine courante
    const today = new Date();
    const todayStr = today.toISOString().split('T')[0];
    await weekInput.fill(todayStr);
    await weekInput.press('Enter');
    await page.waitForTimeout(1000);

    const finalDate = await weekInput.inputValue();
    expect(finalDate).toBe(todayStr);
  });

  test('4. Activation/désactivation d\'un jour de travail', async ({ page }) => {
    // Attendre le chargement
    await expect(page.getByText('Horaires hebdomadaires')).toBeVisible();

    // Trouver la carte Samedi (généralement inactive par défaut)
    const samediSection = page.locator('.day-card').filter({ hasText: 'Samedi' });

    // Vérifier l'état initial (inactif)
    const toggleLabel = samediSection.getByText('Inactif');
    await expect(toggleLabel).toBeVisible();

    // Activer le samedi
    await samediSection.getByRole('checkbox').check();

    // Vérifier que le label a changé
    await expect(samediSection.getByText('Actif')).toBeVisible();

    // Vérifier que les champs sont maintenant activés
    const startInput = samediSection.getByLabel('Début');
    await expect(startInput).toBeEnabled();

    // Désactiver le samedi
    await samediSection.getByRole('checkbox').uncheck();

    // Vérifier le retour à l'état inactif
    await expect(samediSection.getByText('Inactif')).toBeVisible();
  });

  test('5. Modification du seuil d\'heures supplémentaires', async ({ page }) => {
    // Attendre le chargement
    await expect(page.getByText('Paramètres')).toBeVisible();

    const settingsPanel = page.locator('section[aria-label="Paramètres"]');

    // Récupérer la valeur actuelle du seuil
    const thresholdInput = settingsPanel.getByLabel('Seuil d\'heures sup (minutes)');
    const initialValue = await thresholdInput.inputValue();

    // Modifier le seuil à 1515 minutes (25h15 au lieu de 35h par défaut)
    await thresholdInput.fill('1515');
    await page.waitForTimeout(200);

    // Cliquer sur le bouton Enregistrer
    await settingsPanel.getByRole('button', { name: 'Enregistrer' }).click();

    // Attendre la sauvegarde
    await page.waitForTimeout(500);

    // Vérifier que la valeur a été sauvegardée (le champ conserve sa valeur)
    const newValue = await thresholdInput.inputValue();
    expect(newValue).toBe('1515');

    // Restaurer la valeur initiale
    await thresholdInput.fill(initialValue.toString());
    await settingsPanel.getByRole('button', { name: 'Enregistrement...' }).click();
  });
});
