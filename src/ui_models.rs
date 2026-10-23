//! Conversione dei dati backend nei modelli Slint e aggiornamenti delle liste.
use crate::{
    art,
    backend::{Entry, Shelf},
    AppWindow, MusicRow, ShelfData,
};
use slint::{Model, ModelRc, SharedString, VecModel};
use std::{path::PathBuf, rc::Rc};

pub(super) fn load(p: &Option<PathBuf>) -> Option<slint::Image> {
    p.as_ref()
        .and_then(|p| slint::Image::load_from_path(p).ok())
}
pub(super) fn colour(rgb: [u8; 3]) -> slint::Color {
    slint::Color::from_rgb_u8(rgb[0], rgb[1], rgb[2])
}
/// Una riga della GUI. Si decodifica **una sola** immagine per elemento: la grande (240 px) per
/// schede e testate (`large`), altrimenti la piccola (96 px) per righe, coda, playlist e
/// suggerimenti. Caricarle entrambe raddoppiava la memoria delle copertine.
pub(super) fn row(e: Entry, large: bool) -> MusicRow {
    let img = if large {
        load(&e.cover).or_else(|| load(&e.small))
    } else {
        load(&e.small).or_else(|| load(&e.cover))
    };
    // Senza immagine la GUI disegna una copertina generata (colore dal titolo), non il logo.
    let has_cover = img.is_some();
    let img = img.unwrap_or_default();
    let key = if e.title.is_empty() { &e.id } else { &e.title };
    let tint = colour(art::placeholder_tint(key));
    MusicRow {
        id: e.id.into(),
        title: e.title.into(),
        subtitle: e.subtitle.into(),
        duration: e.duration.into(),
        // `slint::Image` è condivisa: la stessa immagine in due campi non occupa il doppio.
        cover: img.clone(),
        thumb: img,
        has_cover,
        tint,
        kind: e.kind.into(),
        liked: e.liked,
    }
}
pub(super) fn rows(entries: Vec<Entry>, large: bool) -> ModelRc<MusicRow> {
    Rc::new(VecModel::from(
        entries
            .into_iter()
            .map(|e| row(e, large))
            .collect::<Vec<_>>(),
    ))
    .into()
}
pub(super) fn shelf(index: usize, s: Shelf) -> ShelfData {
    ShelfData {
        index: index as i32,
        title: s.title.into(),
        songs: s.songs,
        // Le schede mostrano copertine grandi, gli elenchi di brani miniature.
        items: rows(s.items, !s.songs),
    }
}
pub(super) fn time(s: f64) -> SharedString {
    let s = if s.is_finite() { s.max(0.0) as u64 } else { 0 };
    format!("{}:{:02}", s / 60, s % 60).into()
}
/// Aggiorna il cuore di un brano in tutti gli elenchi visibili.
fn set_liked(model: &ModelRc<MusicRow>, f: &dyn Fn(&MusicRow) -> Option<bool>) {
    for i in 0..model.row_count() {
        if let Some(mut r) = model.row_data(i) {
            if let Some(on) = f(&r) {
                if r.liked != on {
                    r.liked = on;
                    model.set_row_data(i, r);
                }
            }
        }
    }
}
pub(super) fn for_each_list(ui: &AppWindow, f: &dyn Fn(&MusicRow) -> Option<bool>) {
    set_liked(&ui.get_rows(), f);
    set_liked(&ui.get_queue(), f);
    set_liked(&ui.get_lead_songs().items, f);
    let shelves = ui.get_shelves();
    for i in 0..shelves.row_count() {
        if let Some(s) = shelves.row_data(i) {
            set_liked(&s.items, f);
        }
    }
}

/// Mantiene la priorità originale (righe, coda, risultato principale, scaffali),
/// interrompendo la visita appena viene trovato il primo ID corrispondente.
pub(super) fn find_track(ui: &AppWindow, id: &str, has_thumbnail: bool) -> Option<MusicRow> {
    if !has_thumbnail {
        return None;
    }
    let find = |model: ModelRc<MusicRow>| {
        (0..model.row_count())
            .filter_map(|i| model.row_data(i))
            .find(|row| row.id == id)
    };
    if let Some(row) = find(ui.get_rows()) {
        return Some(row);
    }
    if let Some(row) = find(ui.get_queue()) {
        return Some(row);
    }
    if let Some(row) = find(ui.get_lead_songs().items) {
        return Some(row);
    }
    let shelves = ui.get_shelves();
    for i in 0..shelves.row_count() {
        if let Some(shelf) = shelves.row_data(i) {
            if let Some(row) = find(shelf.items) {
                return Some(row);
            }
        }
    }
    None
}
