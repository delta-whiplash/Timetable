//! Construction de la feuille de temps exportable (modèle pur, sans I/O).

use crate::domain::{
    errors::ValidationError,
    logic::{calculate_day_minutes, minutes_to_label, signed_minutes_to_label, summarize_week, threshold_percentage},
    types::WeekSheet,
};

/// Feuille de temps prête à sérialiser : titre, métadonnées, tableau des
/// jours et ligne de totaux. Structure volontairement plate pour être
/// testée unitairement avant toute écriture XLSX.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSheet {
    pub title: String,
    pub meta: Vec<(String, String)>,
    pub header: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
    pub footer: Vec<(String, String)>,
}

const NO_TIME: &str = "--:--";

/// Index de la colonne « Total (min) » — écrite en nombre pour rester
/// sommable/pivotable dans Excel.
const MINUTES_COLUMN: usize = 5;

/// Sérialise la feuille en classeur XLSX (mapping 1:1, sans logique de calcul).
pub fn sheet_to_xlsx(sheet: &ExportSheet) -> Vec<u8> {
    use rust_xlsxwriter::{Color, Format, Workbook};

    let bold = Format::new().set_bold();
    let title_format = Format::new().set_bold().set_font_size(14);
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xDCE6F1));

    let mut workbook = Workbook::new();

    // Propriétés figées : deux exports du meme état produisent des bytes
    // identiques (indispensable pour l'audit). Sans ça, docProps/core.xml
    // embarque Utc::now() et fait varier le hash à chaque export.
    use chrono::TimeZone;
    let fixed_epoch = chrono::Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
    let properties = rust_xlsxwriter::DocProperties::new()
        .set_title(&sheet.title)
        .set_author("Timetable Desktop")
        .set_creation_datetime(&fixed_epoch);
    workbook.set_properties(&properties);

    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Semaine").expect("nom de feuille");

    // Titre fusionné sur la largeur du tableau
    let last_col = (sheet.header.len() as u16).saturating_sub(1);
    worksheet
        .merge_range(0, 0, 0, last_col, &sheet.title, &title_format)
        .expect("titre");

    for (index, (key, value)) in sheet.meta.iter().enumerate() {
        let row = 1 + index as u32;
        worksheet.write_with_format(row, 0, key.as_str(), &bold).expect("meta clé");
        worksheet.write(row, 1, value.as_str()).expect("meta valeur");
    }

    let header_row = 2 + sheet.meta.len() as u32;
    for (col, heading) in sheet.header.iter().enumerate() {
        worksheet
            .write_with_format(header_row, col as u16, *heading, &header_format)
            .expect("en-tête");
    }

    for (row_offset, row) in sheet.rows.iter().enumerate() {
        for (col, cell) in row.iter().enumerate() {
            let cell_ref = (header_row + 1 + row_offset as u32, col as u16);
            if col == MINUTES_COLUMN {
                worksheet
                    .write(cell_ref.0, cell_ref.1, cell.parse::<u32>().unwrap_or(0))
                    .expect("cellule minutes");
            } else {
                worksheet.write(cell_ref.0, cell_ref.1, cell.as_str()).expect("cellule");
            }
        }
    }

    let footer_start = header_row + 1 + sheet.rows.len() as u32 + 1;
    for (index, (label, value)) in sheet.footer.iter().enumerate() {
        let row = footer_start + index as u32;
        worksheet.write_with_format(row, 0, label.as_str(), &bold).expect("pied libellé");
        worksheet.write(row, 1, value.as_str()).expect("pied valeur");
    }

    worksheet.set_column_width(0, 24).expect("largeur colonne");
    for col in 1..=last_col {
        worksheet.set_column_width(col, 12).expect("largeur colonne");
    }

    workbook.save_to_buffer().expect("classeur xlsx")
}

/// Construit la feuille exportable d'une semaine, solde cumulé inclus.
/// Échoue si la semaine contient des données invalides (validées à la
/// sauvegarde, donc en pratique jamais vues ici).
pub fn build_export_sheet(
    week: &WeekSheet,
    cumulative_balance: i32,
) -> Result<ExportSheet, ValidationError> {
    let summary = summarize_week(week)?;
    let sunday = week.week_start.0 + chrono::Duration::days(6);

    let rows = week
        .entries
        .iter()
        .map(|entry| {
            let total = calculate_day_minutes(entry, week.travel_deduction_minutes, week.vacation_day_hours)?;
            Ok(vec![
                entry.label.0.clone(),
                if entry.enabled { "Oui" } else { "Non" }.to_string(),
                entry
                    .interval
                    .as_ref()
                    .map(|interval| interval.start.to_hhmm())
                    .unwrap_or_else(|| NO_TIME.to_string()),
                entry
                    .interval
                    .as_ref()
                    .map(|interval| interval.end.to_hhmm())
                    .unwrap_or_else(|| NO_TIME.to_string()),
                crate::domain::types::TimeOfDay(entry.break_minutes.0).to_hhmm(),
                total.to_string(),
                minutes_to_label(total),
            ])
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ExportSheet {
        title: format!(
            "Feuille de temps — Semaine du {} au {}",
            week.week_start.0.format("%d/%m/%Y"),
            sunday.format("%d/%m/%Y")
        ),
        meta: vec![
            (
                "Application".to_string(),
                format!("Timetable Desktop v{}", env!("CARGO_PKG_VERSION")),
            ),
            (
                "Identifiant semaine".to_string(),
                week.week_id
                    .as_ref()
                    .map(|id| id.0.clone())
                    .unwrap_or_else(|| "Non sauvegardée".to_string()),
            ),
            (
                "Seuil heures supplémentaires".to_string(),
                minutes_to_label(week.overtime_threshold.0),
            ),
        ],
        header: vec!["Jour", "Activé", "Début", "Fin", "Pause", "Total (min)", "Total"],
        rows,
        footer: vec![
            ("Total semaine".to_string(), minutes_to_label(summary.total_minutes)),
            ("Jours travaillés".to_string(), summary.worked_days.to_string()),
            (
                "Heures supplémentaires".to_string(),
                minutes_to_label(summary.total_minutes.saturating_sub(week.overtime_threshold.0)),
            ),
            (
                "Objectif atteint".to_string(),
                format!(
                    "{} %",
                    threshold_percentage(summary.total_minutes, week.overtime_threshold.0)
                ),
            ),
            (
                "Solde cumulé".to_string(),
                signed_minutes_to_label(cumulative_balance),
            ),
        ],
    })
}

/// Construit un classeur XLSX multi-feuilles contenant toutes les semaines.
/// Chaque semaine a sa propre feuille nommée avec la date de début.
/// Retourne le classeur sérialisé en bytes.
pub fn build_all_weeks_xlsx(
    sheets: Vec<(String, ExportSheet)>,
) -> Vec<u8> {
    use rust_xlsxwriter::{Color, Format, Workbook};

    let bold = Format::new().set_bold();
    let title_format = Format::new().set_bold().set_font_size(14);
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xDCE6F1));

    let mut workbook = Workbook::new();

    // Propriétés fixes pour l'audit (byte-identical exports)
    use chrono::TimeZone;
    let fixed_epoch = chrono::Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
    let properties = rust_xlsxwriter::DocProperties::new()
        .set_title("Export complet - Timetable Desktop")
        .set_author("Timetable Desktop")
        .set_creation_datetime(&fixed_epoch);
    workbook.set_properties(&properties);

    for (sheet_name, sheet) in sheets {
        // Tronquer le nom de feuille à 31 caractères (limite Excel)
        let worksheet_name = if sheet_name.len() > 31 {
            format!("Sem {}", &sheet_name[..27])
        } else {
            sheet_name.clone()
        };

        let worksheet = workbook.add_worksheet();
        worksheet.set_name(&worksheet_name).expect("nom de feuille");

        // Titre fusionné sur la largeur du tableau
        let last_col = (sheet.header.len() as u16).saturating_sub(1);
        worksheet
            .merge_range(0, 0, 0, last_col, &sheet.title, &title_format)
            .expect("titre");

        for (index, (key, value)) in sheet.meta.iter().enumerate() {
            let row = 1 + index as u32;
            worksheet.write_with_format(row, 0, key.as_str(), &bold).expect("meta clé");
            worksheet.write(row, 1, value.as_str()).expect("meta valeur");
        }

        let header_row = 2 + sheet.meta.len() as u32;
        for (col, heading) in sheet.header.iter().enumerate() {
            worksheet
                .write_with_format(header_row, col as u16, *heading, &header_format)
                .expect("en-tête");
        }

        for (row_offset, row) in sheet.rows.iter().enumerate() {
            for (col, cell) in row.iter().enumerate() {
                let cell_ref = (header_row + 1 + row_offset as u32, col as u16);
                if col == MINUTES_COLUMN {
                    worksheet
                        .write(cell_ref.0, cell_ref.1, cell.parse::<u32>().unwrap_or(0))
                        .expect("cellule minutes");
                } else {
                    worksheet.write(cell_ref.0, cell_ref.1, cell.as_str()).expect("cellule");
                }
            }
        }

        let footer_start = header_row + 1 + sheet.rows.len() as u32 + 1;
        for (index, (label, value)) in sheet.footer.iter().enumerate() {
            let row = footer_start + index as u32;
            worksheet.write_with_format(row, 0, label.as_str(), &bold).expect("pied libellé");
            worksheet.write(row, 1, value.as_str()).expect("pied valeur");
        }

        worksheet.set_column_width(0, 24).expect("largeur colonne");
        for col in 1..=last_col {
            worksheet.set_column_width(col, 12).expect("largeur colonne");
        }
    }

    workbook.save_to_buffer().expect("classeur xlsx")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::logic::{default_settings, minutes_to_label, signed_minutes_to_label};
    use crate::domain::types::{
        BreakMinutes, DayEntry, DayId, DayLabel, DayType, OvertimeThresholdMinutes, TimeOfDay,
        TravelDeductionMinutes, WeekId, WeekStartDate, WorkInterval,
    };

    fn day(day_id: u8, label: &str, start: Option<(u16, u16)>, break_minutes: u16, enabled: bool) -> DayEntry {
        DayEntry {
            day_id: DayId(day_id),
            label: DayLabel(label.to_string()),
            interval: start.map(|(start, end)| WorkInterval {
                start: TimeOfDay(start),
                end: TimeOfDay(end),
            }),
            break_minutes: BreakMinutes(break_minutes),
            enabled,
            has_departure_deduction: false,
            has_return_deduction: false,
            day_type: if enabled { DayType::Work } else { DayType::Disabled },
        }
    }

    fn fixture_week() -> WeekSheet {
        let h = |hh: u16, mm: u16| hh * 60 + mm;
        WeekSheet {
            week_id: Some(WeekId("fixture".to_string())),
            week_start: WeekStartDate::parse("2026-09-07").expect("lundi valide"),
            entries: vec![
                day(0, "Lundi", Some((h(8, 0), h(18, 0))), 60, true),
                day(1, "Mardi", Some((h(8, 0), h(17, 30))), 30, true),
                day(2, "Mercredi", Some((h(9, 0), h(17, 0))), 60, true),
                day(3, "Jeudi", Some((h(8, 0), h(18, 0))), 60, true),
                day(4, "Vendredi", Some((h(8, 0), h(16, 0))), 30, true),
                // Samedi désactivé mais horaires préservés (comme en base)
                day(5, "Samedi", Some((h(8, 0), h(18, 0))), 60, false),
                // Dimanche désactivé sans horaire
                day(6, "Dimanche", None, 0, false),
            ],
            overtime_threshold: OvertimeThresholdMinutes(35 * 60),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            vacation_day_hours: 468,
            updated_at: String::new(),
        }
    }

    fn sheet(balance: i32) -> ExportSheet {
        build_export_sheet(&fixture_week(), balance).expect("semaine valide")
    }

    #[test]
    fn le_titre_couvre_toute_la_semaine() {
        assert_eq!(
            sheet(150).title,
            "Feuille de temps — Semaine du 07/09/2026 au 13/09/2026"
        );
    }

    #[test]
    fn les_meta_mentionnent_le_seuil_heures_sup() {
        assert!(sheet(150)
            .meta
            .contains(&("Seuil heures supplémentaires".to_string(), minutes_to_label(35 * 60))));
    }

    #[test]
    fn les_en_tetes_sont_stables_pour_la_retrocompat() {
        assert_eq!(
            sheet(150).header,
            vec!["Jour", "Activé", "Début", "Fin", "Pause", "Total (min)", "Total"]
        );
    }

    #[test]
    fn sept_lignes_une_par_jour() {
        assert_eq!(sheet(150).rows.len(), 7);
    }

    #[test]
    fn jour_actif_format_complet() {
        assert_eq!(
            sheet(150).rows[0],
            vec!["Lundi", "Oui", "08:00", "18:00", "01:00", "540", "9h00"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn jour_desactive_conserve_ses_horaires() {
        assert_eq!(
            sheet(150).rows[5],
            vec!["Samedi", "Non", "08:00", "18:00", "01:00", "0", "0h00"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn jour_sans_horaire_affiche_tirets() {
        assert_eq!(
            sheet(150).rows[6],
            vec!["Dimanche", "Non", "--:--", "--:--", "00:00", "0", "0h00"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn les_totaux_micromanagement_grade() {
        // 540 + 540 + 420 + 540 + 450 = 2490 min = 41h30
        let footer = sheet(150).footer;
        assert!(footer.contains(&("Total semaine".to_string(), "41h30".to_string())));
        assert!(footer.contains(&("Jours travaillés".to_string(), "5".to_string())));
        assert!(footer.contains(&("Heures supplémentaires".to_string(), "6h30".to_string())));
        assert!(footer.contains(&("Objectif atteint".to_string(), "118 %".to_string())));
        assert!(footer.contains(&("Solde cumulé".to_string(), signed_minutes_to_label(150))));
    }

    #[test]
    fn solde_cumulé_négatif_s_affiche_avec_signe() {
        assert!(sheet(-60)
            .footer
            .contains(&("Solde cumulé".to_string(), "-1h00".to_string())));
    }

    #[test]
    fn pause_nulle_sur_jour_sans_horaire() {
        // La pause par défaut ne doit pas fuir sur le dimanche sans horaire
        assert_eq!(sheet(0).rows[6][4], "00:00");
        assert_ne!(default_settings().default_break_minutes, BreakMinutes(0));
    }

    #[test]
    fn le_classeur_est_un_zip_xlsx_valide() {
        let bytes = sheet_to_xlsx(&sheet(150));
        assert_eq!(&bytes[..4], b"PK\x03\x04", "un xlsx est un zip");
        assert!(bytes.len() > 500, "classeur suspiciousement maigre: {} o", bytes.len());
    }

    #[test]
    fn deux_semaines_differentes_donnent_deux_classeurs_differents() {
        let bytes_a = sheet_to_xlsx(&sheet(150));
        let mut autre = sheet(150);
        autre.rows[0][6] = "8h00".to_string();
        let bytes_b = sheet_to_xlsx(&autre);
        assert_ne!(bytes_a, bytes_b);
    }

    #[test]
    fn le_meme_export_deux_fois_produit_le_meme_fichier() {
        // Pour l'audit : deux exports du meme etat doivent etre byte-identiques.
        // Sinon le hash change et ressemble a de la falsification.
        let first = sheet_to_xlsx(&sheet(150));
        std::thread::sleep(std::time::Duration::from_secs(2));
        let second = sheet_to_xlsx(&sheet(150));
        assert_eq!(first, second, "deux exports a 2s d'intervalle doivent etre byte-identiques");
    }

    #[test]
    fn les_meta_incluent_la_version_et_l_identifiant() {
        let sheet = sheet(150);
        assert!(sheet.meta.iter().any(|(k, v)| k == "Application" && v.starts_with("Timetable Desktop v")));
        assert!(sheet.meta.iter().any(|(k, _)| k == "Identifiant semaine"));
    }
}
