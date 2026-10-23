//! Le pagine dell'app, costruite dalle stesse chiamate InnerTube che usa Limusic: Home a scaffali,
//! Libreria, ricerca completa con filtri, playlist, album e artista con la loro testata.

use anyhow::Result;
use innertube::{BrowseItem, InnerTube, Rating, SongItem, YouTubeClient};
use std::path::PathBuf;

/// Viste della GUI (property `page` in Slint).
pub const VIEW_HOME: i32 = 0;
pub const VIEW_SEARCH: i32 = 1;
pub const VIEW_LIBRARY: i32 = 2;
pub const VIEW_LIKED: i32 = 3;
pub const VIEW_LOCAL: i32 = 4;
pub const VIEW_SETTINGS: i32 = 5;
pub const VIEW_DETAIL: i32 = 6;

#[derive(Clone)]
pub struct Entry {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub subtitle: String,
    pub duration: String,
    pub thumb: Option<String>,
    /// Copertina 240 px (schede, testate).
    pub cover: Option<PathBuf>,
    /// Copertina 96 px (righe, coda, barra laterale): niente aliasing nel rimpicciolire.
    pub small: Option<PathBuf>,
    pub song: Option<SongItem>,
    pub liked: bool,
}
impl Entry {
    pub fn song(s: SongItem) -> Self {
        Self {
            id: s.video_id.clone(),
            kind: "song".into(),
            title: s.title.clone(),
            subtitle: s.artists.clone(),
            duration: s.duration.clone().unwrap_or_default(),
            thumb: s.thumbnail.clone(),
            cover: None,
            small: None,
            liked: s.rating == Some(Rating::Like),
            song: Some(s),
        }
    }
    pub fn card(c: BrowseItem) -> Self {
        if c.kind == "song" {
            return Self::song(SongItem {
                video_id: c.id,
                title: c.title,
                artists: c.subtitle.unwrap_or_default(),
                duration: c.duration,
                thumbnail: c.thumbnail,
                album_id: c.album_id,
                artist_runs: c.artist_runs,
                ..Default::default()
            });
        }
        Self {
            id: c.id,
            kind: c.kind.into(),
            title: c.title,
            subtitle: c.subtitle.unwrap_or_default(),
            duration: String::new(),
            thumb: c.thumbnail,
            cover: None,
            small: None,
            song: None,
            liked: false,
        }
    }
}

/// Uno scaffale: schede (album, playlist, artisti) oppure, se `songs`, un elenco di brani.
#[derive(Clone)]
pub struct Shelf {
    pub title: String,
    pub songs: bool,
    pub items: Vec<Entry>,
}
impl Shelf {
    fn of(title: impl Into<String>, items: Vec<BrowseItem>) -> Option<Self> {
        let items: Vec<Entry> = items.into_iter().map(Entry::card).collect();
        if items.is_empty() {
            return None;
        }
        Some(Self {
            title: title.into(),
            songs: items.iter().all(|e| e.song.is_some()),
            items,
        })
    }
    fn cards(title: impl Into<String>, items: Vec<BrowseItem>) -> Option<Self> {
        let mut s = Self::of(title, items)?;
        s.songs = false;
        Some(s)
    }
}

/// Testata di playlist, album e artista.
#[derive(Clone, Default)]
pub struct Hero {
    /// "PLAYLIST", "ALBUM • 2026", "ARTISTA".
    pub label: String,
    pub title: String,
    pub subtitle: String,
    pub detail: String,
    pub thumb: Option<String>,
    pub cover: Option<PathBuf>,
    /// Avatar rotondo (artisti).
    pub round: bool,
}

#[derive(Clone, Default)]
pub struct Page {
    pub view: i32,
    pub title: String,
    pub subtitle: String,
    pub hero: Option<Hero>,
    pub shelves: Vec<Shelf>,
    /// Elenco principale dei brani (playlist, album, brani piaciuti, filtro Brani della ricerca).
    pub rows: Vec<Entry>,
    /// I brani di `rows` si riproducono come lista (coda = elenco) e non come radio.
    pub rows_are_list: bool,
    pub continuation: Option<String>,
    pub append: bool,
    /// Ricerca: testo e filtro attivo ("all", "songs", "videos", "albums", "artists", "playlists").
    pub query: String,
    pub filter: String,
    /// C'è una pagina precedente (freccia Indietro attiva). Lo imposta il backend.
    pub can_back: bool,
}

fn songs(items: Vec<SongItem>) -> Vec<Entry> {
    items.into_iter().map(Entry::song).collect()
}

/// "1 h 12 min" / "38 min" dalla somma delle durate "m:ss" o "h:mm:ss".
pub fn total_duration(rows: &[Entry]) -> Option<String> {
    let mut secs = 0u64;
    let mut any = false;
    for r in rows {
        let Ok(parts) = r
            .duration
            .split(':')
            .map(|p| p.trim().parse::<u64>())
            .collect::<std::result::Result<Vec<u64>, _>>()
        else {
            continue; // brano senza durata: non si conta
        };
        if parts.is_empty() || parts.len() > 3 {
            continue;
        }
        secs += parts.iter().fold(0, |acc, p| acc * 60 + p);
        any = true;
    }
    if !any {
        return None;
    }
    let (h, m) = (secs / 3600, (secs % 3600) / 60);
    Some(if h > 0 {
        format!("{h} h {m} min")
    } else {
        format!("{} min", m.max(1))
    })
}

fn count_line(rows: &[Entry], more: bool) -> String {
    let n = rows.len();
    let mut s = if n == 1 {
        "1 brano".to_owned()
    } else {
        format!("{n}{} brani", if more { "+" } else { "" })
    };
    if let Some(d) = total_duration(rows) {
        s.push_str(" • ");
        s.push_str(&d);
        if more {
            s.push('+');
        }
    }
    s
}

pub async fn home(tube: &InnerTube, client: &YouTubeClient) -> Result<Page> {
    let p = tube.home(client, None).await?;
    let shelves = p
        .sections
        .into_iter()
        .filter_map(|s| Shelf::of(s.title, s.items))
        .collect();
    Ok(Page {
        view: VIEW_HOME,
        title: greeting(),
        subtitle: "Selezioni da YouTube Music".into(),
        shelves,
        ..Default::default()
    })
}

/// Saluto della Home come in Limusic (Buongiorno / Buon pomeriggio / Buonasera / Buonanotte).
fn greeting() -> String {
    // Senza una libreria per i fusi orari si usa l'ora UTC + 2 (ora legale italiana): per un
    // saluto basta.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let hour = ((secs / 3600) + 2) % 24;
    match hour {
        5..=12 => "Buongiorno",
        13..=17 => "Buon pomeriggio",
        18..=22 => "Buonasera",
        _ => "Buonanotte",
    }
    .into()
}

pub async fn library(tube: &InnerTube, client: &YouTubeClient) -> Result<Page> {
    let playlists = tube.library_playlists(client).await?;
    let albums = tube.library_albums(client).await.unwrap_or_default();
    let artists = tube.library_artists(client).await.unwrap_or_default();
    let shelves = [
        Shelf::cards("Playlist", playlists),
        Shelf::cards("Album", albums),
        Shelf::cards("Artisti", artists),
    ]
    .into_iter()
    .flatten()
    .collect();
    Ok(Page {
        view: VIEW_LIBRARY,
        title: "Libreria".into(),
        subtitle: "Playlist, album e artisti del tuo account".into(),
        shelves,
        ..Default::default()
    })
}

pub async fn playlist(
    tube: &InnerTube,
    client: &YouTubeClient,
    id: &str,
    fallback: &str,
    view: i32,
) -> Result<Page> {
    let p = tube.playlist(client, id, None).await?;
    let rows = songs(p.items);
    let more = p.continuation.is_some();
    Ok(Page {
        view,
        title: p.title.clone().unwrap_or_else(|| fallback.to_owned()),
        hero: Some(Hero {
            label: "PLAYLIST".into(),
            title: p.title.unwrap_or_else(|| fallback.to_owned()),
            subtitle: p.subtitle.unwrap_or_default(),
            detail: count_line(&rows, more),
            thumb: p
                .thumbnail
                .or_else(|| rows.first().and_then(|r| r.thumb.clone())),
            ..Default::default()
        }),
        rows,
        rows_are_list: true,
        continuation: p.continuation,
        ..Default::default()
    })
}

pub async fn album(
    tube: &InnerTube,
    client: &YouTubeClient,
    id: &str,
    fallback: &str,
) -> Result<Page> {
    let p = tube.album(client, id).await?;
    let rows = songs(p.items);
    let shelves = p
        .sections
        .into_iter()
        .filter_map(|c| Shelf::cards(c.title, c.items))
        .collect();
    let detail = p
        .second_subtitle
        .clone()
        .unwrap_or_else(|| count_line(&rows, false));
    Ok(Page {
        view: VIEW_DETAIL,
        title: p.title.clone().unwrap_or_else(|| fallback.to_owned()),
        hero: Some(Hero {
            label: p.subtitle.unwrap_or_else(|| "Album".into()).to_uppercase(),
            title: p.title.unwrap_or_else(|| fallback.to_owned()),
            subtitle: p.artist.unwrap_or_default(),
            detail,
            thumb: p.thumbnail,
            ..Default::default()
        }),
        rows,
        rows_are_list: true,
        shelves,
        continuation: p.continuation,
        ..Default::default()
    })
}

pub async fn artist(
    tube: &InnerTube,
    client: &YouTubeClient,
    id: &str,
    fallback: &str,
) -> Result<Page> {
    let p = tube.artist(client, id).await?;
    let name = p.name.clone().unwrap_or_else(|| fallback.to_owned());
    let shelves = p
        .sections
        .into_iter()
        .filter_map(|c| Shelf::cards(c.title, c.items))
        .collect();
    let detail = [p.subscribers.clone(), p.monthly_listeners.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" • ");
    Ok(Page {
        view: VIEW_DETAIL,
        title: name.clone(),
        hero: Some(Hero {
            label: "ARTISTA".into(),
            title: name,
            subtitle: if p.top_songs.is_empty() {
                String::new()
            } else {
                "Più ascoltati".into()
            },
            detail,
            thumb: p.avatar.or(p.thumbnail),
            round: true,
            ..Default::default()
        }),
        rows: songs(p.top_songs),
        rows_are_list: true,
        shelves,
        ..Default::default()
    })
}

/// Ricerca come la pagina Cerca di Limusic: "Tutti" = risultato principale, brani, album, artisti,
/// playlist; gli altri filtri mostrano una sola categoria.
pub async fn search(
    tube: &InnerTube,
    client: &YouTubeClient,
    query: &str,
    filter: &str,
) -> Result<Page> {
    let mut page = Page {
        view: VIEW_SEARCH,
        title: format!("Risultati per “{query}”"),
        query: query.to_owned(),
        filter: filter.to_owned(),
        ..Default::default()
    };
    match filter {
        "songs" => {
            page.rows = songs(tube.search_songs(client, query, true).await?.items);
        }
        "videos" => {
            page.rows = songs(tube.search_videos(client, query).await?.items);
        }
        "albums" | "artists" | "playlists" => {
            let title = match filter {
                "albums" => "Album",
                "artists" => "Artisti",
                _ => "Playlist",
            };
            page.shelves = Shelf::cards(title, tube.search_cards(client, query, filter).await?)
                .into_iter()
                .collect();
        }
        _ => {
            let r = tube.search_all(client, query, true).await?;
            page.shelves = [
                Shelf::cards("In evidenza", r.top),
                Shelf::of("Brani", r.songs).map(|mut s| {
                    s.songs = true;
                    s
                }),
                Shelf::cards("Album", r.albums),
                Shelf::cards("Artisti", r.artists),
                Shelf::cards("Playlist", r.playlists),
            ]
            .into_iter()
            .flatten()
            .collect();
        }
    }
    if page.rows.is_empty() && page.shelves.is_empty() {
        page.subtitle = format!("Nessun risultato per “{query}”.");
    }
    Ok(page)
}

pub async fn more(tube: &InnerTube, client: &YouTubeClient, token: &str) -> Result<Page> {
    let p = tube.playlist_continuation(client, token).await?;
    Ok(Page {
        rows: songs(p.items),
        rows_are_list: true,
        continuation: p.continuation,
        append: true,
        ..Default::default()
    })
}

/// Suggerimenti sotto il campo di ricerca: completamenti e alcuni risultati rapidi.
pub async fn suggestions(
    tube: &InnerTube,
    client: &YouTubeClient,
    input: &str,
) -> Result<(Vec<String>, Vec<Entry>)> {
    let s = tube.search_suggestions(client, input).await?;
    let queries = s.queries.into_iter().map(|q| q.text).take(7).collect();
    let items = s.items.into_iter().map(Entry::card).take(6).collect();
    Ok((queries, items))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(d: &str) -> Entry {
        Entry::song(SongItem {
            duration: Some(d.into()),
            ..Default::default()
        })
    }

    #[test]
    fn durations_add_up() {
        assert_eq!(
            total_duration(&[row(""), row("2:00")]).as_deref(),
            Some("2 min")
        );
        assert_eq!(
            total_duration(&[row("3:30"), row("4:30")]).as_deref(),
            Some("8 min")
        );
        assert_eq!(
            total_duration(&[row("1:00:00"), row("12:00")]).as_deref(),
            Some("1 h 12 min")
        );
        assert_eq!(total_duration(&[]), None);
        assert_eq!(count_line(&[row("3:00")], false), "1 brano • 3 min");
        assert_eq!(
            count_line(&[row("3:00"), row("2:00")], true),
            "2+ brani • 5 min+"
        );
    }
}
