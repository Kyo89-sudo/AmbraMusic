use crate::comments::{self as cm, Comments};
use crate::discord::{self, DiscordHandle};
use crate::login::{self, Mode, Outcome};
use crate::pages::{self, Page};
pub use crate::pages::{Entry, Shelf};
use crate::session::{self, Store};
use anyhow::{anyhow, Result};
use innertube::{Clients, Healing, InnerTube, Locale, Rating, Session, SongItem, YouTubeClient};
use player::{Player, PlayerEvent};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::mpsc::Sender,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// Un rinnovo della sessione al massimo ogni 5 minuti: una sessione scaduta produce una raffica
/// di 401, e ognuno chiederebbe un nuovo rinnovo.
const HEAL_COOLDOWN: Duration = Duration::from_secs(5 * 60);

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
    pub volume: i64,
    pub discord: bool,
    /// "dark" oppure "light".
    pub theme: String,
    /// La ✕ della finestra la riduce nell'area di notifica (predefinito come in Limusic).
    pub close_to_tray: bool,
    /// Playlist dell'account nel pannello di destra invece che nella barra laterale.
    pub playlists_right: bool,
    /// Apri la vista "In riproduzione" quando parte un brano.
    pub open_player: bool,
    /// A fine coda continua con brani simili (radio), come la riproduzione automatica di Limusic.
    pub autoplay: bool,
    /// Scheda "Commenti" nel pannello di destra.
    pub comments: bool,
    /// Nome dell'account collegato, mostrato subito all'avvio mentre l'accesso salvato si
    /// verifica in rete. Solo il nome visualizzato: i cookie restano in `session.dat`.
    pub account_name: Option<String>,
    pub favorites: Vec<SongItem>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            volume: 70,
            discord: true,
            theme: "dark".into(),
            close_to_tray: true,
            playlists_right: false,
            open_player: false,
            autoplay: true,
            comments: true,
            account_name: None,
            favorites: vec![],
        }
    }
}
impl Settings {
    pub fn dark(&self) -> bool {
        self.theme != "light"
    }
}
pub struct Stream {
    pub url: String,
    pub headers: HashMap<String, String>,
}

/// Da dove arriva un cookie da verificare: cambia solo i messaggi e cosa fare se fallisce.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Incollato a mano (Avanzate).
    Paste,
    /// Dalla finestra "Accedi con Google".
    Window,
    /// Letto da `session.dat` all'avvio.
    Restore,
    /// Rinnovato dalla finestra invisibile dopo un ripristino fallito.
    RestoreRetry,
}

pub enum Command {
    Go(i32),
    Search(String),
    /// Filtro della pagina Cerca: "all", "songs", "videos", "albums", "artists", "playlists".
    SearchFilter(String),
    /// Testo digitato nel campo di ricerca (suggerimenti, con attesa breve).
    Suggest(String),
    /// Scelta di un risultato rapido dei suggerimenti.
    SuggestPick(usize),
    /// Riga dell'elenco principale.
    Open(usize),
    /// Elemento `i` dello scaffale `s`.
    OpenShelf(usize, usize),
    /// Playlist della barra laterale.
    OpenLibrary(usize),
    /// Riproduci tutto (`true` = in ordine casuale) dalla testata.
    PlayAll(bool),
    More,
    /// Torna alla pagina precedente.
    Back,
    /// Riproduzione casuale dei brani successivi (come Limusic: l'ordine originale si ripristina).
    Shuffle(bool),
    /// 0 = disattivata, 1 = ripeti tutto, 2 = ripeti brano.
    Repeat(i32),
    Add(usize),
    AddShelf(usize, usize),
    Like(usize),
    LikeShelf(usize, usize),
    LikeCurrent,
    PlayQueue(usize),
    Remove(usize),
    Skip(i32),
    Toggle,
    Seek(f64),
    Volume(i64),
    Discord(bool),
    Theme(bool),
    /// Interruttori delle impostazioni: "close_to_tray", "playlists_right", "open_player", "autoplay".
    Setting(String, bool),
    ClearCache,
    /// Cookie header incollato.
    Login(String),
    /// `true` = "Cambia account" (svuota prima la sessione Google della finestra).
    GoogleLogin(bool),
    CancelLogin,
    Logout,
    Shutdown,
    PageReady(u64, Result<Page>),
    SuggestReady(u64, Result<(Vec<String>, Vec<Entry>)>),
    LibraryReady(u64, Result<Vec<Entry>>),
    RadioReady(u64, bool, Result<Vec<SongItem>>),
    Resolved(u64, SongItem, Result<Stream>),
    LoginReady(
        u64,
        InnerTube,
        Result<innertube::AccountInfo, innertube::Error>,
        Origin,
    ),
    WindowDone(u64, Outcome),
    RestoreRefreshed(u64, String, Outcome),
    Healed(u64, Outcome, Healing),
    RateReady(String, Rating, Result<()>),
    /// Commenti: scheda aperta / scheda del pannello scelta (0 coda, 1 playlist, 2 commenti).
    CommentsOpen,
    PanelTab(i32),
    CommentsSort(bool),
    CommentsMore,
    CommentLike(usize),
    CommentReplies(usize),
    /// Testo e riga a cui rispondere (-1 = commento nuovo).
    CommentSend(String, i32),
    /// Pagina di commenti (`true` = sostituisce l'elenco) e account con cui è stata letta.
    CommentsReady(u64, bool, Result<(innertube::CommentsPage, Option<String>)>),
    RepliesReady(u64, String, Result<innertube::CommentReplies>),
    VoteReady(u64, String, innertube::CommentAction, Result<()>),
    SentReady(
        u64,
        Option<String>,
        Result<Option<innertube::CommentThread>>,
    ),
    /// Foto degli autori scaricate.
    CommentAvatars(u64),
    /// Copertine della coda scaricate: si rimanda la coda alla GUI.
    QueueCovers,
}
pub enum Update {
    Page(Page),
    Queue(Vec<Entry>, i32),
    /// Intestazione della coda ("<titolo> Radio", nome della playlist…).
    QueueSource(String),
    Library(Vec<Entry>),
    Suggestions(Vec<String>, Vec<Entry>),
    Status(String, bool),
    Account(String, bool),
    /// Foto del profilo Google (rotonda nella GUI); `None` = iniziale/icona.
    Avatar(Option<PathBuf>),
    LoginPending(bool),
    SessionSaved(bool),
    Track(Entry),
    /// Apri la vista "In riproduzione" (impostazione "Mostra la copertina grande all'avvio di un brano").
    OpenPlayer,
    /// Copertina grande del brano `id` (per la vista "In riproduzione") e colore dominante.
    Art(String, Option<PathBuf>, [u8; 3]),
    Position(f64),
    Duration(f64),
    Playing(bool),
    Favorites(Vec<String>),
    Rating(String, bool),
    /// Stato del pulsante riproduzione casuale (si spegne quando cambia la coda).
    Shuffle(bool),
    Comments(CommentsUpdate),
    /// Commento o risposta pubblicati: la GUI svuota il campo e chiude la risposta in corso.
    CommentSent,
}

/// La scheda "Commenti" com'è ora.
pub struct CommentsUpdate {
    pub rows: Vec<cm::Row>,
    /// `comments-state` della GUI; -1 = nessun brano.
    pub view: i32,
    pub count: String,
    pub newest: bool,
    pub has_more: bool,
    pub can_write: bool,
    pub sending: bool,
    /// Elenco nuovo (altro brano o altro ordine): la GUI annulla la risposta in corso.
    pub reset: bool,
}

fn locale() -> Locale {
    Locale {
        gl: "IT".into(),
        hl: "it".into(),
    }
}
fn anonymous() -> InnerTube {
    InnerTube::new(
        Session {
            locale: locale(),
            ..Default::default()
        },
        None,
    )
    .unwrap()
}

pub fn start(
    settings: Settings,
    data: PathBuf,
    demo: bool,
    tx: Sender<Update>,
) -> (UnboundedSender<Command>, std::thread::JoinHandle<()>) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let to_backend = sender.clone();
    let worker = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async move {
            crate::audioproxy::start();
            let tube = anonymous();
            let clients = Clients::bundled();
            let client = clients.get("WEB_REMIX").unwrap().clone();
            let _ = std::fs::create_dir_all(data.join("audio-cache"));
            let mut player = Player::new(data.join("audio-cache").to_string_lossy().as_ref()).ok();
            if let Some(p) = &player {
                let _ = p.set_volume(settings.volume);
            }
            let events = player.as_mut().and_then(Player::take_events);
            let mut app = Engine {
                tube,
                client,
                clients,
                player,
                events,
                discord: discord::spawn(settings.discord),
                settings,
                store: Store::new(&data),
                data,
                tx,
                cmd: to_backend,
                rows: vec![],
                rows_are_list: false,
                shelves: vec![],
                library: vec![],
                suggestions: vec![],
                query: String::new(),
                filter: "all".into(),
                title: String::new(),
                shown: None,
                history: vec![],
                unshuffled: None,
                repeat: 0,
                queue: vec![],
                current: None,
                generation: 0,
                page_gen: 0,
                auth_gen: 0,
                window_gen: 0,
                suggest_gen: 0,
                library_gen: 0,
                resolver: None,
                browser_task: None,
                login_task: None,
                window_task: None,
                heal_task: None,
                suggest_task: None,
                radio_task: None,
                last_heal: None,
                saved_jar: None,
                continuation: None,
                demo,
                comments: Comments::default(),
                comments_active: false,
                comments_busy: false,
            };
            if app.player.is_none() {
                app.status("Motore audio non disponibile. Controlla libmpv.", false);
            }
            if demo {
                app.show_demo();
            } else if let Some(cookie) = app.store.load() {
                // Accesso salvato: lo si verifica prima di caricare la Home, così la prima pagina
                // è già quella dell'account.
                app.begin_login(cookie, Origin::Restore);
            } else {
                app.browse(pages::VIEW_HOME);
            }
            app.run(receiver).await;
        });
    });
    (sender, worker)
}
struct Engine {
    tube: InnerTube,
    client: YouTubeClient,
    clients: Clients,
    player: Option<Player>,
    events: Option<UnboundedReceiver<PlayerEvent>>,
    discord: Option<DiscordHandle>,
    settings: Settings,
    store: Store,
    data: PathBuf,
    tx: Sender<Update>,
    cmd: UnboundedSender<Command>,
    rows: Vec<Entry>,
    rows_are_list: bool,
    shelves: Vec<Shelf>,
    /// Playlist dell'account (barra laterale).
    library: Vec<Entry>,
    /// Risultati rapidi dei suggerimenti attualmente mostrati.
    suggestions: Vec<Entry>,
    query: String,
    filter: String,
    /// Titolo della pagina aperta (intestazione della coda quando la si riproduce).
    title: String,
    /// Pagina mostrata e pagine precedenti (pulsante Indietro).
    shown: Option<Page>,
    history: Vec<Page>,
    /// Ordine della coda prima della riproduzione casuale.
    unshuffled: Option<Vec<SongItem>>,
    repeat: i32,
    queue: Vec<SongItem>,
    current: Option<usize>,
    generation: u64,
    page_gen: u64,
    auth_gen: u64,
    window_gen: u64,
    suggest_gen: u64,
    library_gen: u64,
    resolver: Option<tokio::task::JoinHandle<()>>,
    browser_task: Option<tokio::task::JoinHandle<()>>,
    login_task: Option<tokio::task::JoinHandle<()>>,
    window_task: Option<tokio::task::JoinHandle<()>>,
    heal_task: Option<tokio::task::JoinHandle<()>>,
    suggest_task: Option<tokio::task::JoinHandle<()>>,
    radio_task: Option<tokio::task::JoinHandle<()>>,
    last_heal: Option<Instant>,
    /// Ultimo cookie scritto su disco, per non riscriverlo a ogni risposta identica.
    saved_jar: Option<String>,
    continuation: Option<String>,
    demo: bool,
    comments: Comments,
    /// La scheda "Commenti" è quella aperta: si caricano solo allora.
    comments_active: bool,
    /// Una pagina (altri commenti, risposte) in arrivo: niente richieste doppie.
    comments_busy: bool,
}
impl Engine {
    fn send(&self, u: Update) {
        let _ = self.tx.send(u);
    }
    fn status(&self, s: impl Into<String>, busy: bool) {
        self.send(Update::Status(s.into(), busy));
    }
    fn persist(&self) {
        if let Ok(s) = serde_json::to_vec_pretty(&self.settings) {
            if std::fs::write(self.data.join("settings.json"), s).is_err() {
                self.status("Impossibile salvare le preferenze locali.", false);
            }
        }
    }
    fn metadata_client(&self) -> YouTubeClient {
        self.clients
            .get(innertube::METADATA_CLIENT)
            .cloned()
            .unwrap_or_else(|| self.client.clone())
    }
    fn entry(&self, song: SongItem) -> Entry {
        let local = self
            .settings
            .favorites
            .iter()
            .any(|s| s.video_id == song.video_id);
        let mut entry = Entry::song(song);
        if !self.tube.is_logged_in() {
            entry.liked = local;
        }
        entry
    }
    fn queue_update(&self) {
        self.send_queue(true);
    }
    /// Manda la coda alla GUI con le copertine già in cache. Le mancanti si scaricano in
    /// background (`fetch`) e poi la coda si rimanda una volta sola, senza riscaricare.
    fn send_queue(&self, fetch: bool) {
        let cache = self.data.join("covers");
        let mut missing = Vec::new();
        let list: Vec<Entry> = self
            .queue
            .iter()
            .cloned()
            .map(|s| {
                let mut e = self.entry(s);
                if let Some(url) = e.thumb.clone() {
                    let (big, small) = cover_files(&cache, &url);
                    if big.exists() && small.exists() {
                        e.cover = Some(big);
                        e.small = Some(small);
                    } else {
                        missing.push(url);
                    }
                }
                e
            })
            .collect();
        self.send(Update::Queue(list, self.current.map_or(-1, |i| i as i32)));
        if fetch && !missing.is_empty() && !self.demo {
            let data = self.data.clone();
            let tx = self.cmd.clone();
            tokio::spawn(async move {
                covers_for(missing, &data).await;
                let _ = tx.send(Command::QueueCovers);
            });
        }
    }
    fn show_demo(&mut self) {
        let names = [
            ("First light", "Sessione acustica", "3:42"),
            ("Orange skies", "Studio session", "4:08"),
            ("Slow morning", "Piano e atmosfere", "2:56"),
            ("After hours", "Electronic sessions", "3:24"),
            ("Quiet streets", "Acoustic collective", "4:12"),
        ];
        let rows: Vec<_> = names
            .into_iter()
            .map(|(title, artist, dur)| {
                Entry::song(SongItem {
                    video_id: "demo".into(),
                    title: title.into(),
                    artists: artist.into(),
                    duration: Some(dur.into()),
                    ..Default::default()
                })
            })
            .collect();
        self.rows = rows.clone();
        self.shelves = vec![Shelf {
            title: "Scelte rapide".into(),
            songs: false,
            items: rows.iter().take(4).cloned().collect(),
        }];
        self.queue = rows.iter().take(3).filter_map(|r| r.song.clone()).collect();
        self.queue_update();
        self.send(Update::Page(Page {
            view: pages::VIEW_HOME,
            title: "Buonasera".into(),
            subtitle: "Anteprima dell’interfaccia · dati dimostrativi".into(),
            shelves: self.shelves.clone(),
            rows,
            ..Default::default()
        }));
        self.status(
            "Modalità demo: ricerca, account e riproduzione online disattivati.",
            false,
        );
    }
    fn spawn_page<F>(&mut self, f: F)
    where
        F: std::future::Future<Output = Result<Page>> + Send + 'static,
    {
        self.page_gen += 1;
        let gen = self.page_gen;
        if let Some(t) = self.browser_task.take() {
            t.abort();
        }
        let tx = self.cmd.clone();
        let data = self.data.clone();
        self.status("Caricamento…", true);
        self.browser_task = Some(tokio::spawn(async move {
            let result = match f.await {
                Ok(mut p) => {
                    load_covers(&mut p, &data).await;
                    Ok(p)
                }
                Err(e) => Err(e),
            };
            let _ = tx.send(Command::PageReady(gen, result));
        }));
    }
    // Le richieste di pagina non usano `without_healing`: se YouTube rifiuta la sessione, il
    // trasporto aspetta il rinnovo (vedi `heal`) e riprova da solo una volta.
    fn browse(&mut self, view: i32) {
        if self.demo {
            self.show_demo();
            return;
        }
        let tube = self.tube.clone();
        let client = self.client.clone();
        match view {
            pages::VIEW_SETTINGS => {}
            pages::VIEW_LOCAL => {
                let rows = self
                    .settings
                    .favorites
                    .iter()
                    .cloned()
                    .map(Entry::song)
                    .collect();
                self.spawn_page(async move {
                    Ok(Page {
                        view: pages::VIEW_LOCAL,
                        title: "Preferiti locali".into(),
                        subtitle: "Salvati su questo computer".into(),
                        rows,
                        rows_are_list: true,
                        ..Default::default()
                    })
                });
            }
            pages::VIEW_SEARCH => {
                if self.query.is_empty() {
                    self.spawn_page(async move {
                        Ok(Page {
                            view: pages::VIEW_SEARCH,
                            title: "Cerca".into(),
                            subtitle: "Scrivi un titolo, un artista o un album.".into(),
                            filter: "all".into(),
                            ..Default::default()
                        })
                    });
                } else {
                    let (q, f) = (self.query.clone(), self.filter.clone());
                    self.spawn_page(async move { pages::search(&tube, &client, &q, &f).await });
                }
            }
            pages::VIEW_LIBRARY | pages::VIEW_LIKED if !self.tube.is_logged_in() => {
                self.status(
                    "Accedi con il tuo account per vedere la libreria e i brani che ti piacciono.",
                    false,
                );
            }
            pages::VIEW_LIBRARY => {
                self.spawn_page(async move { pages::library(&tube, &client).await });
            }
            pages::VIEW_LIKED => {
                self.spawn_page(async move {
                    pages::playlist(&tube, &client, "VLLM", "I tuoi mi piace", pages::VIEW_LIKED)
                        .await
                });
            }
            _ => {
                self.spawn_page(async move { pages::home(&tube, &client).await });
            }
        }
    }
    fn search(&mut self, query: String) {
        let query = query.trim().to_owned();
        if query.is_empty() {
            return;
        }
        if self.demo {
            self.status("Riavvia senza --demo per usare la ricerca reale.", false);
            return;
        }
        self.cancel_suggestions();
        self.query = query;
        self.filter = "all".into();
        self.browse(pages::VIEW_SEARCH);
    }
    fn search_filter(&mut self, filter: String) {
        if self.query.is_empty() || self.demo {
            return;
        }
        self.filter = filter;
        self.browse(pages::VIEW_SEARCH);
    }
    fn more(&mut self) {
        let Some(token) = self.continuation.take() else {
            return;
        };
        let tube = self.tube.clone();
        let client = self.client.clone();
        self.spawn_page(async move { pages::more(&tube, &client, &token).await });
    }
    /// Apre una scheda: album, playlist o artista diventano una pagina, un brano parte con la sua radio.
    fn open_entry(&mut self, e: Entry) {
        let Entry {
            id,
            kind,
            title,
            song,
            ..
        } = e;
        if let Some(song) = song {
            self.play_song(song);
            return;
        }
        if self.demo {
            return;
        }
        let tube = self.tube.clone();
        let client = self.client.clone();
        match kind.as_str() {
            "album" => {
                self.spawn_page(async move { pages::album(&tube, &client, &id, &title).await })
            }
            "artist" => {
                self.spawn_page(async move { pages::artist(&tube, &client, &id, &title).await })
            }
            "playlist" => self.spawn_page(async move {
                pages::playlist(&tube, &client, &id, &title, pages::VIEW_DETAIL).await
            }),
            _ => self.status("Questo elemento non si può aprire.", false),
        }
    }
    /// Riga dell'elenco principale: nelle playlist/album la coda diventa l'elenco (come in
    /// Limusic `playFrom`), altrove il brano parte con la sua radio (`play_song`).
    fn open_row(&mut self, i: usize) {
        let Some(e) = self.rows.get(i).cloned() else {
            return;
        };
        let Some(song) = e.song.clone() else {
            self.open_entry(e);
            return;
        };
        if !self.rows_are_list {
            self.play_song(song);
            return;
        }
        self.queue = self.rows.iter().filter_map(|e| e.song.clone()).collect();
        self.new_context();
        let index = self.rows[..i].iter().filter(|e| e.song.is_some()).count();
        self.send(Update::QueueSource(self.title.clone()));
        self.play(index);
    }
    fn open_shelf(&mut self, s: usize, i: usize) {
        let Some(shelf) = self.shelves.get(s).cloned() else {
            return;
        };
        let Some(e) = shelf.items.get(i).cloned() else {
            return;
        };
        self.open_entry(e);
    }
    fn play_all(&mut self, shuffle: bool) {
        let mut songs: Vec<SongItem> = self.rows.iter().filter_map(|e| e.song.clone()).collect();
        if songs.is_empty() {
            songs = self
                .shelves
                .iter()
                .filter(|s| s.songs)
                .flat_map(|s| s.items.iter().filter_map(|e| e.song.clone()))
                .collect();
        }
        if songs.is_empty() {
            return;
        }
        if shuffle {
            use rand::seq::SliceRandom;
            songs.shuffle(&mut rand::thread_rng());
        }
        self.queue = songs;
        self.new_context();
        self.send(Update::QueueSource(self.title.clone()));
        self.play(0);
    }
    /// Un brano singolo (ricerca, Home, suggerimenti): parte subito e la coda si riempie con la
    /// sua radio `RDAMVM<id>`, senza doppioni. Stessa logica di `AppState::play_song` in Limusic.
    fn play_song(&mut self, song: SongItem) {
        self.send(Update::QueueSource(if self.settings.autoplay {
            format!("{} Radio", song.title)
        } else {
            String::new()
        }));
        self.queue = vec![song];
        self.new_context();
        self.play(0);
        if self.settings.autoplay {
            self.fetch_radio(false);
        }
    }
    /// Radio del brano attuale da accodare. `then_play`: a fine coda, parti con il primo nuovo brano.
    fn fetch_radio(&mut self, then_play: bool) {
        let Some(seed) = self.current.and_then(|i| self.queue.get(i)).cloned() else {
            return;
        };
        if self.demo || seed.video_id == "demo" {
            return;
        }
        if let Some(t) = self.radio_task.take() {
            t.abort();
        }
        let gen = self.generation;
        let tube = self.tube.clone();
        let client = self.metadata_client();
        let tx = self.cmd.clone();
        self.radio_task = Some(tokio::spawn(async move {
            let radio = format!("RDAMVM{}", seed.video_id);
            let r = tube
                .next(&client, Some(seed.video_id.as_str()), Some(radio.as_str()))
                .await
                .map(|n| n.items)
                .map_err(Into::into);
            let _ = tx.send(Command::RadioReady(gen, then_play, r));
        }));
    }
    fn radio_ready(&mut self, gen: u64, then_play: bool, r: Result<Vec<SongItem>>) {
        if gen != self.generation {
            return; // nel frattempo è partito altro
        }
        let Ok(items) = r else {
            if then_play {
                self.status("Fine della coda.", false);
            }
            return;
        };
        let mut seen: HashSet<String> = self.queue.iter().map(|s| s.video_id.clone()).collect();
        let before = self.queue.len();
        for item in items {
            if seen.insert(item.video_id.clone()) {
                if let Some(orig) = self.unshuffled.as_mut() {
                    orig.push(item.clone());
                }
                self.queue.push(item);
            } else if let Some(have) = self.queue.iter_mut().find(|s| s.video_id == item.video_id) {
                // Le schede di ricerca e Home non hanno l'album; la riga della radio sì.
                have.album = have.album.take().or(item.album);
            }
        }
        self.queue_update();
        if then_play {
            if self.queue.len() > before {
                self.play(before);
            } else {
                self.status("Fine della coda.", false);
            }
        }
    }
    fn page_ready(&mut self, gen: u64, result: Result<Page>) {
        if gen != self.page_gen {
            return;
        }
        match result {
            Ok(mut p) => {
                let local = !self.tube.is_logged_in();
                let favs: HashSet<&str> = self
                    .settings
                    .favorites
                    .iter()
                    .map(|s| s.video_id.as_str())
                    .collect();
                if local {
                    for e in p
                        .rows
                        .iter_mut()
                        .chain(p.shelves.iter_mut().flat_map(|s| s.items.iter_mut()))
                    {
                        e.liked = favs.contains(e.id.as_str());
                    }
                }
                self.continuation = p.continuation.clone();
                if p.append {
                    self.rows.extend(p.rows.clone());
                    p.rows = self.rows.clone();
                    p.view = -1; // resta sulla pagina attuale
                } else {
                    self.rows = p.rows.clone();
                    self.rows_are_list = p.rows_are_list;
                    self.title = p
                        .hero
                        .as_ref()
                        .map(|h| h.title.clone())
                        .unwrap_or_else(|| p.title.clone());
                    self.shelves = p.shelves.clone();
                    // Cronologia per "Indietro": la pagina precedente va in pila.
                    if let Some(prev) = self.shown.replace(p.clone()) {
                        self.history.push(prev);
                        if self.history.len() > 30 {
                            self.history.remove(0);
                        }
                    }
                }
                p.can_back = !self.history.is_empty();
                self.send(Update::Page(p));
                self.status("Pronto", false);
            }
            Err(e) => {
                let expired = e
                    .downcast_ref::<innertube::Error>()
                    .is_some_and(|e| matches!(e, innertube::Error::SessionExpired));
                self.status(
                    if expired {
                        "La sessione YouTube è scaduta: accedi di nuovo dalle Impostazioni."
                    } else {
                        "Caricamento non riuscito. Controlla la connessione o ricollega l’account."
                    },
                    false,
                )
            }
        }
    }
    // ----- Suggerimenti di ricerca (come SearchSuggest.svelte) -----
    fn cancel_suggestions(&mut self) {
        self.suggest_gen += 1;
        if let Some(t) = self.suggest_task.take() {
            t.abort();
        }
        self.suggestions.clear();
        self.send(Update::Suggestions(vec![], vec![]));
    }
    fn suggest(&mut self, text: String) {
        let text = text.trim().to_owned();
        if self.demo || text.chars().count() < 2 {
            self.cancel_suggestions();
            return;
        }
        self.suggest_gen += 1;
        let gen = self.suggest_gen;
        if let Some(t) = self.suggest_task.take() {
            t.abort();
        }
        let tube = self.tube.clone();
        let client = self.client.clone();
        let data = self.data.clone();
        let tx = self.cmd.clone();
        self.suggest_task = Some(tokio::spawn(async move {
            // Si aspetta che l'utente smetta di digitare un attimo.
            tokio::time::sleep(Duration::from_millis(220)).await;
            let r = match pages::suggestions(&tube, &client, &text).await {
                Ok((q, mut items)) => {
                    load_entry_covers(&mut items, &data).await;
                    Ok((q, items))
                }
                Err(e) => Err(e),
            };
            let _ = tx.send(Command::SuggestReady(gen, r));
        }));
    }
    fn suggest_ready(&mut self, gen: u64, r: Result<(Vec<String>, Vec<Entry>)>) {
        if gen != self.suggest_gen {
            return;
        }
        if let Ok((queries, items)) = r {
            self.suggestions = items.clone();
            self.send(Update::Suggestions(queries, items));
        }
    }
    fn suggest_pick(&mut self, i: usize) {
        let Some(e) = self.suggestions.get(i).cloned() else {
            return;
        };
        self.cancel_suggestions();
        self.open_entry(e);
    }
    // ----- Libreria nella barra laterale -----
    fn load_library(&mut self) {
        if self.demo || !self.tube.is_logged_in() {
            return;
        }
        self.library_gen += 1;
        let gen = self.library_gen;
        let tube = self.tube.clone();
        let client = self.client.clone();
        let data = self.data.clone();
        let tx = self.cmd.clone();
        tokio::spawn(async move {
            let r = match tube.library_playlists(&client).await {
                Ok(items) => {
                    let mut list: Vec<Entry> = items.into_iter().map(Entry::card).collect();
                    load_entry_covers(&mut list, &data).await;
                    Ok(list)
                }
                Err(e) => Err(e.into()),
            };
            let _ = tx.send(Command::LibraryReady(gen, r));
        });
    }
    fn library_ready(&mut self, gen: u64, r: Result<Vec<Entry>>) {
        if gen != self.library_gen || !self.tube.is_logged_in() {
            return;
        }
        if let Ok(list) = r {
            self.library = list.clone();
            self.send(Update::Library(list));
        }
    }
    fn clear_cache(&mut self) {
        let mut bytes = 0u64;
        for dir in ["covers", "art"] {
            let path = self.data.join(dir);
            if let Ok(entries) = std::fs::read_dir(&path) {
                for e in entries.flatten() {
                    bytes += e.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
            let _ = std::fs::remove_dir_all(&path);
        }
        self.status(
            format!(
                "Cache delle copertine svuotata ({:.1} MB liberati).",
                bytes as f64 / 1_048_576.0
            ),
            false,
        );
    }
    fn play(&mut self, i: usize) {
        if self.demo {
            self.status(
                "Questa è un’anteprima grafica: i brani sono dimostrativi.",
                false,
            );
            return;
        }
        let Some(song) = self.queue.get(i).cloned() else {
            return;
        };
        if self.player.is_none() {
            self.status("libmpv non disponibile.", false);
            return;
        }
        self.current = Some(i);
        self.generation += 1;
        if let Some(t) = self.resolver.take() {
            t.abort();
        }
        if let Some(p) = &self.player {
            let _ = p.stop();
        }
        if let Some(d) = &self.discord {
            // Cambio di brano, non una pausa: Discord tiene la scheda finché parte il nuovo.
            d.set_switching();
        }
        self.send(Update::Playing(false));
        self.send(Update::Position(0.0));
        self.send(Update::Duration(0.0));
        self.send(Update::Track(self.entry(song.clone())));
        self.comments_load();
        if self.settings.open_player {
            self.send(Update::OpenPlayer);
        }
        if let Some(url) = song.thumbnail.clone() {
            let tx = self.tx.clone();
            let data = self.data.clone();
            let id = song.video_id.clone();
            tokio::spawn(async move {
                let (path, tint) = match fetch_art(&url, &data).await {
                    Some((p, t)) => (Some(p), t),
                    None => (None, crate::art::tint(&[])),
                };
                let _ = tx.send(Update::Art(id, path, tint));
            });
        }
        self.queue_update();
        self.status("Preparazione del flusso audio…", true);
        let gen = self.generation;
        let tube = self.tube.clone();
        let clients = self.clients.clone();
        let tx = self.cmd.clone();
        self.resolver = Some(tokio::spawn(async move {
            let result =
                tokio::time::timeout(Duration::from_secs(50), resolve(&tube, &clients, &song))
                    .await
                    .unwrap_or_else(|_| Err(anyhow!("Tempo scaduto. Riprova il brano.")));
            let _ = tx.send(Command::Resolved(gen, song, result));
        }));
    }
    fn stop_playback(&mut self) {
        self.generation += 1;
        if let Some(t) = self.resolver.take() {
            t.abort();
        }
        if let Some(t) = self.radio_task.take() {
            t.abort();
        }
        if let Some(p) = &self.player {
            let _ = p.stop();
        }
        if let Some(d) = &self.discord {
            d.set_playing(false);
        }
        self.current = None;
        self.send(Update::Playing(false));
        self.send(Update::Position(0.0));
        self.send(Update::Duration(0.0));
        self.send(Update::Track(Entry::song(SongItem::default())));
        self.comments_load();
    }
    fn remove(&mut self, i: usize) {
        if i >= self.queue.len() {
            return;
        }
        let removed = self.queue.remove(i);
        // Con la riproduzione casuale attiva l'ordine originale va tenuto allineato, o
        // spegnendola il brano tolto tornerebbe in coda.
        if let Some(orig) = self.unshuffled.as_mut() {
            if let Some(j) = orig.iter().position(|s| s.video_id == removed.video_id) {
                orig.remove(j);
            }
        }
        if self.current == Some(i) {
            self.stop_playback();
        } else if let Some(cur) = self.current {
            if i < cur {
                self.current = Some(cur - 1);
            }
        }
        self.queue_update();
    }
    fn enqueue(&mut self, song: SongItem) {
        // Anche nell'ordine originale: spegnendo la casuale il brano aggiunto non deve sparire.
        if let Some(orig) = self.unshuffled.as_mut() {
            orig.push(song.clone());
        }
        self.queue.push(song);
        self.queue_update();
        self.status("Messo in coda.", false);
    }
    fn back(&mut self) {
        let Some(page) = self.history.pop() else {
            return;
        };
        self.page_gen += 1; // una pagina in caricamento non deve sovrascrivere quella ripresa
        if let Some(t) = self.browser_task.take() {
            t.abort();
        }
        self.rows = page.rows.clone();
        self.rows_are_list = page.rows_are_list;
        self.shelves = page.shelves.clone();
        self.continuation = page.continuation.clone();
        self.query = page.query.clone();
        if !page.filter.is_empty() {
            self.filter = page.filter.clone();
        }
        self.title = page
            .hero
            .as_ref()
            .map(|h| h.title.clone())
            .unwrap_or_else(|| page.title.clone());
        let mut page = page;
        page.can_back = !self.history.is_empty();
        page.view = if page.view < 0 {
            pages::VIEW_DETAIL
        } else {
            page.view
        };
        self.shown = Some(page.clone());
        self.send(Update::Page(page));
        self.status("Pronto", false);
    }
    /// Una nuova coda (brano, playlist, album): la riproduzione casuale della precedente non vale più.
    fn new_context(&mut self) {
        if self.unshuffled.take().is_some() {
            self.send(Update::Shuffle(false));
        }
    }
    fn shuffle(&mut self, on: bool) {
        let start = self.current.map_or(0, |i| i + 1);
        if on {
            if self.unshuffled.is_none() {
                self.unshuffled = Some(self.queue.clone());
            }
            if start < self.queue.len() {
                use rand::seq::SliceRandom;
                self.queue[start..].shuffle(&mut rand::thread_rng());
            }
        } else if let Some(orig) = self.unshuffled.take() {
            // Torna all'ordine originale, mantenendo il brano in riproduzione.
            let playing = self
                .current
                .and_then(|i| self.queue.get(i))
                .map(|s| s.video_id.clone());
            self.current = playing
                .as_ref()
                .and_then(|id| orig.iter().position(|s| &s.video_id == id))
                .or(self.current.filter(|&i| i < orig.len()));
            self.queue = orig;
        }
        self.queue_update();
    }
    fn track_ended(&mut self) {
        let Some(i) = self.current else {
            return;
        };
        if self.repeat == 2 {
            self.play(i);
        } else if i + 1 < self.queue.len() {
            self.play(i + 1);
        } else if self.repeat == 1 && !self.queue.is_empty() {
            self.play(0);
        } else if self.settings.autoplay {
            // Riproduzione automatica: la radio dell'ultimo brano continua la coda.
            self.fetch_radio(true);
        } else {
            self.status("Fine della coda.", false);
        }
    }
    fn resolved(&mut self, gen: u64, song: SongItem, r: Result<Stream>) {
        if gen != self.generation {
            return;
        }
        match r {
            Ok(stream) => {
                let url = crate::audioproxy::register(&stream.url, &stream.headers)
                    .unwrap_or(stream.url);
                let headers = if url.starts_with("http://127.0.0.1:") {
                    HashMap::new()
                } else {
                    stream.headers
                };
                if let Some(p) = &self.player {
                    match p.load(&url, &headers).and_then(|_| p.play()) {
                        Ok(_) => {
                            if let Some(d) = &self.discord {
                                d.set_track(&song);
                            }
                            self.status("Riproduzione", false);
                        }
                        Err(_) => self.status("Impossibile avviare l’audio.", false),
                    }
                }
            }
            Err(_) => self.status(
                "Stream YouTube non disponibile. Questa versione non include ancora l’estrazione web completa.",
                false,
            ),
        }
    }
    fn toggle_favorite(&mut self, song: SongItem) {
        if let Some(i) = self
            .settings
            .favorites
            .iter()
            .position(|s| s.video_id == song.video_id)
        {
            self.settings.favorites.remove(i);
        } else {
            self.settings.favorites.push(song);
        }
        self.persist();
        self.send(Update::Favorites(
            self.settings
                .favorites
                .iter()
                .map(|s| s.video_id.clone())
                .collect(),
        ));
        self.status("Preferiti locali aggiornati.", false);
    }
    fn rate(&mut self, song: SongItem) {
        if self.demo {
            self.status("Preferiti disattivati nella demo.", false);
            return;
        }
        if !self.tube.is_logged_in() {
            self.toggle_favorite(song);
            return;
        }
        let rating = if song.rating == Some(Rating::Like) {
            Rating::Indifferent
        } else {
            Rating::Like
        };
        let tube = self.tube.clone();
        let client = self.client.clone();
        let tx = self.cmd.clone();
        tokio::spawn(async move {
            let r = tube
                .rate(&client, &song.video_id, rating)
                .await
                .map_err(Into::into);
            let _ = tx.send(Command::RateReady(song.video_id, rating, r));
        });
    }
    fn rate_ready(&mut self, id: String, rating: Rating, r: Result<()>) {
        match r {
            Ok(_) => {
                for e in self
                    .rows
                    .iter_mut()
                    .chain(self.shelves.iter_mut().flat_map(|s| s.items.iter_mut()))
                {
                    if e.id == id {
                        e.liked = rating == Rating::Like;
                        if let Some(s) = &mut e.song {
                            s.rating = Some(rating);
                        }
                    }
                }
                for s in &mut self.queue {
                    if s.video_id == id {
                        s.rating = Some(rating);
                    }
                }
                self.send(Update::Rating(id, rating == Rating::Like));
                self.status("Valutazione aggiornata su YouTube Music.", false);
            }
            Err(_) => self.status(
                "YouTube Music non ha confermato la modifica. Riapri la pagina prima di riprovare.",
                false,
            ),
        }
    }
    fn setting(&mut self, key: &str, on: bool) {
        match key {
            "close_to_tray" => self.settings.close_to_tray = on,
            "playlists_right" => self.settings.playlists_right = on,
            "open_player" => self.settings.open_player = on,
            "autoplay" => self.settings.autoplay = on,
            "comments" => {
                self.settings.comments = on;
                if !on {
                    self.comments_active = false;
                    self.comments.clear();
                }
            }
            _ => return,
        }
        self.persist();
    }

    // ----- Account -----

    /// Verifica un cookie (incollato, dalla finestra Google o salvato) e, se valido, lo adotta.
    fn begin_login(&mut self, cookie: String, origin: Origin) {
        if self.demo {
            self.status("Account disattivati nella demo.", false);
            return;
        }
        self.auth_gen += 1;
        let gen = self.auth_gen;
        if let Some(t) = self.login_task.take() {
            t.abort();
        }
        let cookie = session::normalize(&cookie);
        if innertube::cookie_sapisid(&cookie).is_none() {
            self.send(Update::LoginPending(false));
            self.status(
                "Cookie non valido: manca SAPISID o __Secure-3PAPISID.",
                false,
            );
            return;
        }
        let mut s = Session {
            locale: locale(),
            ..Default::default()
        };
        s.cookie = Some(cookie);
        let tube = InnerTube::new(s, None).unwrap();
        let client = self.client.clone();
        let tx = self.cmd.clone();
        self.status(
            match origin {
                Origin::Restore | Origin::RestoreRetry => "Ripristino dell’accesso salvato…",
                _ => "Verifica dell’account…",
            },
            true,
        );
        self.login_task = Some(tokio::spawn(async move {
            // La verifica vuole la risposta vera di YouTube, non un rinnovo.
            let r = innertube::without_healing(tube.account_menu(&client)).await;
            let _ = tx.send(Command::LoginReady(gen, tube, r, origin));
        }));
    }
    fn login_ready(
        &mut self,
        gen: u64,
        tube: InnerTube,
        r: Result<innertube::AccountInfo, innertube::Error>,
        origin: Origin,
    ) {
        if gen != self.auth_gen {
            return;
        }
        let offline = matches!(&r, Err(innertube::Error::Http(e)) if e.status().is_none());
        match r {
            Ok(account) if account.name.is_some() => {
                tube.set_data_sync_id(account.data_sync_id);
                tube.set_visitor_data(account.visitor_data);
                self.tube = tube;
                self.saved_jar = None;
                let saved = self.save_session();
                let name = account.name.unwrap_or_default();
                if saved && self.settings.account_name.as_deref() != Some(name.as_str()) {
                    self.settings.account_name = Some(name.clone());
                    self.persist();
                }
                self.send(Update::Account(name, true));
                // I token dei commenti appartengono all'account con cui sono stati letti.
                self.comments.clear();
                self.comments_load();
                if let Some(url) = account.thumbnail.clone() {
                    let tx = self.tx.clone();
                    let data = self.data.clone();
                    tokio::spawn(async move {
                        let _ = tx.send(Update::Avatar(fetch_avatar(&url, &data).await));
                    });
                }
                self.send(Update::LoginPending(false));
                self.status(
                    match (origin, saved) {
                        (Origin::Restore | Origin::RestoreRetry, _) => {
                            "Bentornato: accesso ripristinato."
                        }
                        (_, true) => "Account collegato e salvato su questo PC.",
                        (_, false) => {
                            "Account collegato, ma non è stato possibile salvarlo su disco."
                        }
                    },
                    false,
                );
                self.browse(0);
                self.load_library();
            }
            _ => {
                self.send(Update::LoginPending(false));
                match origin {
                    Origin::Restore if offline => {
                        // La sessione resta salvata, ma senza verifica non si mostra collegati.
                        self.send(Update::Account("Accedi".into(), false));
                        self.status(
                            "Nessuna connessione: l’accesso salvato verrà riprovato al prossimo avvio.",
                            false,
                        );
                        self.browse(0);
                    }
                    Origin::Restore if login::SUPPORTED => {
                        // I cookie salvati sono scaduti: si prova a rinnovarli dal profilo della
                        // finestra Google, che ricorda ancora l’accesso.
                        self.restore_refresh(tube.cookie().unwrap_or_default());
                    }
                    Origin::Restore | Origin::RestoreRetry => {
                        self.expired_saved_session();
                        self.browse(0);
                    }
                    _ if offline => self.status(
                        "Nessuna connessione: impossibile verificare l’account.",
                        false,
                    ),
                    _ => self.status(
                        "Accesso non riuscito: YouTube Music non ha riconosciuto la sessione.",
                        false,
                    ),
                }
            }
        }
    }
    fn restore_refresh(&mut self, stored: String) {
        self.auth_gen += 1;
        let gen = self.auth_gen;
        let data = self.data.clone();
        let tx = self.cmd.clone();
        self.status("Rinnovo dell’accesso salvato…", true);
        self.login_task = Some(tokio::spawn(async move {
            let out = login::run_helper(Mode::Refresh, &data).await;
            let _ = tx.send(Command::RestoreRefreshed(gen, stored, out));
        }));
    }
    fn restore_refreshed(&mut self, gen: u64, stored: String, out: Outcome) {
        if gen != self.auth_gen {
            return;
        }
        match out {
            Outcome::Cookie(c) if session::same_account(&c, &stored) => {
                self.begin_login(c, Origin::RestoreRetry)
            }
            _ => {
                self.expired_saved_session();
                self.browse(0);
            }
        }
    }
    fn forget_account_name(&mut self) {
        if self.settings.account_name.take().is_some() {
            self.persist();
        }
    }
    fn expired_saved_session(&mut self) {
        self.store.clear();
        self.saved_jar = None;
        self.forget_account_name();
        self.send(Update::Account("Accedi".into(), false));
        self.send(Update::SessionSaved(false));
        self.status(
            "L’accesso salvato è scaduto: accedi di nuovo da Account e opzioni.",
            false,
        );
    }
    fn google_login(&mut self, add_account: bool) {
        if self.demo {
            self.status("Account disattivati nella demo.", false);
            return;
        }
        if !login::SUPPORTED {
            self.status(
                "Il login integrato è disponibile solo su Windows: usa il Cookie header.",
                false,
            );
            return;
        }
        if let Some(t) = self.window_task.take() {
            t.abort();
        }
        self.window_gen += 1;
        let gen = self.window_gen;
        let data = self.data.clone();
        let tx = self.cmd.clone();
        let mode = if add_account {
            Mode::AddAccount
        } else {
            Mode::SignIn
        };
        self.send(Update::LoginPending(true));
        self.status("Completa l’accesso nella finestra di Google…", true);
        self.window_task = Some(tokio::spawn(async move {
            let out = login::run_helper(mode, &data).await;
            let _ = tx.send(Command::WindowDone(gen, out));
        }));
    }
    fn window_done(&mut self, gen: u64, out: Outcome) {
        if gen != self.window_gen {
            return;
        }
        self.window_task = None;
        match out {
            Outcome::Cookie(c) => self.begin_login(c, Origin::Window),
            Outcome::Cancelled => {
                self.send(Update::LoginPending(false));
                self.status("Accesso annullato.", false);
            }
            Outcome::Failed(e) => {
                self.send(Update::LoginPending(false));
                self.status(e, false);
            }
        }
    }
    fn cancel_login(&mut self) {
        self.window_gen += 1;
        // `kill_on_drop`: interrompere il task chiude anche la finestra di Google.
        if let Some(t) = self.window_task.take() {
            t.abort();
        }
        self.send(Update::LoginPending(false));
        self.status("Accesso annullato.", false);
    }
    /// Scrive su disco il cookie attuale (cifrato su Windows). `true` se è salvato.
    fn save_session(&mut self) -> bool {
        let Some(cookie) = self.tube.cookie() else {
            return false;
        };
        if self
            .saved_jar
            .as_deref()
            .is_some_and(|s| session::same_jar(s, &cookie))
        {
            return true;
        }
        let ok = self.store.save(&cookie).is_ok();
        if ok {
            self.saved_jar = Some(cookie);
        }
        self.send(Update::SessionSaved(ok));
        ok
    }
    /// YouTube ha rifiutato la sessione: si rinnovano i cookie dal profilo della finestra Google,
    /// in background. Le richieste in attesa ripartono quando il `Healing` viene rilasciato.
    fn heal(&mut self) {
        let can_heal = !self.demo && self.tube.is_logged_in() && login::SUPPORTED;
        if can_heal && self.heal_task.as_ref().is_some_and(|t| !t.is_finished()) {
            return; // un rinnovo è già in corso: le richieste aspettano quello
        }
        if !can_heal || self.last_heal.is_some_and(|t| t.elapsed() < HEAL_COOLDOWN) {
            // Nessun rinnovo possibile: si chiude subito un "rinnovo vuoto", così chi aspetta
            // riprova una volta e riceve l’errore senza restare fermo per 90 secondi.
            drop(self.tube.begin_heal());
            return;
        }
        self.last_heal = Some(Instant::now());
        let guard = self.tube.begin_heal();
        let gen = self.auth_gen;
        let data = self.data.clone();
        let tx = self.cmd.clone();
        self.heal_task = Some(tokio::spawn(async move {
            let out = login::run_helper(Mode::Refresh, &data).await;
            let _ = tx.send(Command::Healed(gen, out, guard));
        }));
    }
    fn healed(&mut self, gen: u64, out: Outcome, guard: Healing) {
        if gen == self.auth_gen && self.tube.is_logged_in() {
            let current = self.tube.cookie().unwrap_or_default();
            match out {
                Outcome::Cookie(c) if session::same_account(&c, &current) => {
                    if !session::same_jar(&c, &current) {
                        self.tube.set_cookie(Some(c));
                    }
                    self.save_session();
                    self.status("Sessione YouTube rinnovata.", false);
                }
                Outcome::Cookie(_) => self.status(
                    "La finestra di accesso è collegata a un altro account: accedi di nuovo da Account e opzioni.",
                    false,
                ),
                _ => self.status(
                    "La sessione YouTube è scaduta: accedi di nuovo da Account e opzioni.",
                    false,
                ),
            }
        }
        // Rilasciato dopo aver impostato il nuovo cookie: le richieste in attesa ripartono con quello.
        drop(guard);
    }
    fn logout(&mut self) {
        self.stop_playback();
        self.queue.clear();
        self.queue_update();
        self.auth_gen += 1;
        self.page_gen += 1;
        self.window_gen += 1;
        for t in [
            self.login_task.take(),
            self.browser_task.take(),
            self.window_task.take(),
            self.heal_task.take(),
        ]
        .into_iter()
        .flatten()
        {
            t.abort();
        }
        self.tube = anonymous();
        self.store.clear();
        self.saved_jar = None;
        self.forget_account_name();
        self.send(Update::SessionSaved(false));
        self.send(Update::LoginPending(false));
        self.send(Update::Account("Accedi".into(), false));
        self.comments.clear();
        self.comments_load();
        self.send(Update::Avatar(None));
        let _ = std::fs::remove_file(self.data.join("avatar.png"));
        self.rows.clear();
        self.shelves.clear();
        self.library.clear();
        self.send(Update::Library(vec![]));
        self.send(Update::Page(Page {
            view: pages::VIEW_HOME,
            title: "Account disconnesso".into(),
            subtitle: "Puoi continuare a cercare e ascoltare senza account.".into(),
            ..Default::default()
        }));
        self.status(
            "Disconnesso: l’accesso è stato rimosso da questo PC.",
            false,
        );
    }

    async fn run(&mut self, mut receiver: UnboundedReceiver<Command>) {
        loop {
            // Segnali del trasporto dell’account attuale (cambiano a ogni login/logout).
            let rejected = self.tube.session_rejected();
            let rotated = self.tube.cookie_changed();
            tokio::select! {
                msg = receiver.recv() => {
                    let Some(msg) = msg else { break };
                    match msg {
                        Command::Shutdown => break,
                        Command::Go(view) => self.browse(view),
                        Command::Search(q) => self.search(q),
                        Command::SearchFilter(f) => self.search_filter(f),
                        Command::Suggest(t) => self.suggest(t),
                        Command::SuggestPick(i) => self.suggest_pick(i),
                        Command::SuggestReady(gen, r) => self.suggest_ready(gen, r),
                        Command::Open(i) => self.open_row(i),
                        Command::OpenShelf(s, i) => self.open_shelf(s, i),
                        Command::OpenLibrary(i) => {
                            if let Some(e) = self.library.get(i).cloned() {
                                self.open_entry(e);
                            }
                        }
                        Command::LibraryReady(gen, r) => self.library_ready(gen, r),
                        Command::PlayAll(shuffle) => self.play_all(shuffle),
                        Command::More => self.more(),
                        Command::Back => self.back(),
                        Command::Shuffle(on) => self.shuffle(on),
                        Command::Repeat(mode) => self.repeat = mode.clamp(0, 2),
                        Command::PageReady(gen, result) => self.page_ready(gen, result),
                        Command::Add(i) => match self.rows.get(i).cloned() {
                            Some(Entry { song: Some(s), .. }) => self.enqueue(s),
                            Some(e) => self.open_entry(e),
                            None => {}
                        },
                        Command::AddShelf(s, i) => {
                            let e = self.shelves.get(s).and_then(|sh| sh.items.get(i)).cloned();
                            match e {
                                Some(Entry { song: Some(song), .. }) => self.enqueue(song),
                                Some(e) => self.open_entry(e),
                                None => {}
                            }
                        }
                        Command::RadioReady(gen, then_play, r) => self.radio_ready(gen, then_play, r),
                        Command::PlayQueue(i) => self.play(i),
                        Command::Remove(i) => self.remove(i),
                        Command::Skip(delta) => {
                            if !self.queue.is_empty() {
                                let i = self.current.map_or(0, |i| {
                                    (i as i32 + delta).clamp(0, self.queue.len() as i32 - 1) as usize
                                });
                                if delta > 0 && self.current == Some(self.queue.len() - 1) && self.settings.autoplay {
                                    self.fetch_radio(true);
                                } else {
                                    self.play(i);
                                }
                            }
                        }
                        Command::Toggle => {
                            if self.current.is_none() {
                                if !self.queue.is_empty() {
                                    self.play(0);
                                }
                            } else if let Some(p) = &self.player {
                                if p.toggle().is_err() {
                                    self.status("Riproduzione non disponibile. Seleziona nuovamente il brano.", false);
                                }
                            }
                        }
                        Command::Seek(s) => {
                            if s.is_finite() {
                                if let Some(p) = &self.player {
                                    let _ = p.seek(s.max(0.0));
                                }
                            }
                        }
                        Command::Volume(v) => {
                            self.settings.volume = v.clamp(0, 100);
                            if let Some(p) = &self.player {
                                let _ = p.set_volume(self.settings.volume);
                            }
                            self.persist();
                        }
                        Command::Discord(on) => {
                            self.settings.discord = on;
                            if let Some(d) = &self.discord {
                                d.set_enabled(on);
                            }
                            self.persist();
                        }
                        Command::Theme(dark) => {
                            self.settings.theme = if dark { "dark" } else { "light" }.into();
                            self.persist();
                        }
                        Command::Setting(key, on) => self.setting(&key, on),
                        Command::ClearCache => self.clear_cache(),
                        Command::Like(i) => {
                            if let Some(song) = self.rows.get(i).and_then(|r| r.song.clone()) {
                                self.rate(song);
                            }
                        }
                        Command::LikeShelf(s, i) => {
                            let song = self.shelves.get(s).and_then(|sh| sh.items.get(i)).and_then(|e| e.song.clone());
                            if let Some(song) = song {
                                self.rate(song);
                            }
                        }
                        Command::LikeCurrent => {
                            if let Some(song) = self.current.and_then(|i| self.queue.get(i)).cloned() {
                                self.rate(song);
                            }
                        }
                        Command::RateReady(id, rating, r) => self.rate_ready(id, rating, r),
                        Command::QueueCovers => self.send_queue(false),
                        Command::CommentsOpen => {
                            self.comments_active = true;
                            self.comments_load();
                        }
                        Command::PanelTab(t) => {
                            self.comments_active = t == 2;
                            self.comments_load();
                        }
                        Command::CommentsSort(newest) => self.comments_sort(newest),
                        Command::CommentsMore => self.comments_more(),
                        Command::CommentLike(i) => self.comment_like(i),
                        Command::CommentReplies(i) => self.comment_replies(i),
                        Command::CommentSend(text, to) => self.comment_send(text, to),
                        Command::CommentsReady(gen, replace, r) => self.comments_ready(gen, replace, r),
                        Command::RepliesReady(gen, id, r) => self.replies_ready(gen, id, r),
                        Command::VoteReady(gen, id, action, r) => self.vote_ready(gen, id, action, r),
                        Command::SentReady(gen, to, r) => self.sent_ready(gen, to, r),
                        Command::CommentAvatars(gen) => {
                            if gen == self.comments.gen {
                                self.comments_ui(false);
                            }
                        }
                        Command::Login(cookie) => self.begin_login(cookie, Origin::Paste),
                        Command::GoogleLogin(add) => self.google_login(add),
                        Command::CancelLogin => self.cancel_login(),
                        Command::LoginReady(gen, tube, r, origin) => self.login_ready(gen, tube, r, origin),
                        Command::WindowDone(gen, out) => self.window_done(gen, out),
                        Command::RestoreRefreshed(gen, stored, out) => self.restore_refreshed(gen, stored, out),
                        Command::Healed(gen, out, guard) => self.healed(gen, out, guard),
                        Command::Logout => self.logout(),
                        Command::Resolved(gen, song, r) => self.resolved(gen, song, r),
                    }
                },
                _ = rejected.notified() => self.heal(),
                _ = rotated.notified() => {
                    // Google ha ruotato i cookie `*SIDTS` in una risposta: senza riscriverli su
                    // disco, al prossimo avvio la sessione salvata sarebbe già scaduta.
                    if self.tube.is_logged_in() {
                        self.save_session();
                    }
                },
                event = async { match self.events.as_mut() { Some(rx) => rx.recv().await, None => std::future::pending().await } } => {
                    match event {
                        Some(PlayerEvent::Position(s)) => {
                            self.send(Update::Position(s));
                            if let Some(d) = &self.discord {
                                d.set_position(s);
                            }
                        }
                        Some(PlayerEvent::Duration(s)) => {
                            self.send(Update::Duration(s));
                            if let Some(d) = &self.discord {
                                d.set_duration(s);
                            }
                        }
                        Some(PlayerEvent::Playing(on)) => {
                            self.send(Update::Playing(on));
                            if let Some(d) = &self.discord {
                                d.set_playing(on);
                            }
                        }
                        Some(PlayerEvent::TrackEnded) => self.track_ended(),
                        Some(PlayerEvent::TrackFailed(_) | PlayerEvent::Error(_) | PlayerEvent::AudioDeviceLost) => {
                            self.send(Update::Playing(false));
                            if let Some(d) = &self.discord {
                                d.set_playing(false);
                            }
                            self.status("Audio interrotto. Riprova il brano o controlla il dispositivo audio.", false);
                        }
                        None => self.events = None,
                    }
                }
            }
        }
        for t in [
            self.resolver.take(),
            self.browser_task.take(),
            self.login_task.take(),
            self.window_task.take(),
            self.heal_task.take(),
            self.suggest_task.take(),
            self.radio_task.take(),
        ]
        .into_iter()
        .flatten()
        {
            t.abort();
        }
        if let Some(p) = &self.player {
            let _ = p.stop();
        }
        if let Some(d) = &self.discord {
            d.set_enabled(false);
        }
        self.persist();
    }
}

async fn resolve(tube: &InnerTube, clients: &Clients, song: &SongItem) -> Result<Stream> {
    // Phase 0.1: preserve upstream direct-client path and its pure Rust fallback.
    // Web cipher / PoToken pipeline still lives in the original Tauri application.
    if let Some(client) = clients.get("VISIONOS") {
        if let Ok(p) =
            innertube::without_healing(tube.player(client, &song.video_id, None, None, None)).await
        {
            if let Some(format) = p
                .streaming_data
                .as_ref()
                .and_then(|s| innertube::find_format(s, innertube::AudioQuality::High))
            {
                if let Some(url) = format.direct_url() {
                    let mut headers = HashMap::new();
                    headers.insert("User-Agent".into(), client.user_agent.clone());
                    if let Ok(r) = crate::http::client()
                        .get(url)
                        .header("User-Agent", &client.user_agent)
                        .header("Range", "bytes=0-0")
                        .send()
                        .await
                    {
                        if r.status().is_success() {
                            return Ok(Stream {
                                url: url.into(),
                                headers,
                            });
                        }
                    }
                }
            }
        }
    }
    let stream = innertube::rustypipe_fallback::resolve(&song.video_id, true).await?;
    Ok(Stream {
        url: stream.url,
        headers: HashMap::new(),
    })
}
/// Scarica le miniature (solo dagli host immagini di YouTube, mai con i cookie dell'account) e
/// ne salva due versioni: 240 px per schede e testate, 96 px per righe, coda e barra laterale.
/// Rimpicciolire di 5 volte un'immagine in tempo reale produce aliasing; così no.
// ----- Commenti del brano in ascolto (stato in `comments.rs`) -----
impl Engine {
    fn current_video(&self) -> Option<String> {
        self.current
            .and_then(|i| self.queue.get(i))
            .map(|s| s.video_id.clone())
            .filter(|id| !id.is_empty())
    }

    /// Manda la scheda alla GUI, con le foto degli autori già in cache.
    fn comments_ui(&self, reset: bool) {
        let cache = self.data.join("covers");
        let c = &self.comments;
        let rows = c.rows(|url| {
            let (_, small) = cover_files(&cache, url);
            small.exists().then_some(small)
        });
        self.send(Update::Comments(CommentsUpdate {
            rows,
            view: c.view.map_or(-1, cm::View::code),
            count: c.count.clone(),
            newest: c.newest,
            has_more: c.continuation.is_some(),
            can_write: c.create.is_some() && self.account_matches(),
            sending: c.sending,
            reset,
        }));
    }

    /// I token letti con l'account valgono solo se quell'account è ancora quello collegato.
    fn account_matches(&self) -> bool {
        self.comments.as_account
            && self.tube.is_logged_in()
            && self.comments.identity.is_some()
            && self.tube.comments_identity() == self.comments.identity
    }

    /// Come rimandare i token della pagina: `None` se l'account è cambiato (si ricarica).
    fn comments_origin(&mut self) -> Option<bool> {
        if !self.comments.as_account {
            return Some(false);
        }
        if self.account_matches() {
            return Some(true);
        }
        self.comments.clear();
        self.comments_load();
        None
    }

    /// Carica i commenti del brano in ascolto, se la scheda è aperta e non sono già quelli.
    fn comments_load(&mut self) {
        if !self.settings.comments || !self.comments_active {
            return;
        }
        let Some(id) = self.current_video() else {
            if self.comments.video_id.is_some() || self.comments.view.is_some() {
                self.comments.clear();
            }
            self.comments_ui(true);
            return;
        };
        if self.comments.is_for(&id) && self.comments.view != Some(cm::View::Failed) {
            self.comments_ui(false);
            return;
        }
        let gen = self.comments.start(&id);
        self.comments_busy = false;
        if self.demo {
            self.comments.view = Some(cm::View::Empty);
            self.comments_ui(true);
            return;
        }
        self.comments_ui(true);
        let tube = self.tube.clone();
        let client = self.client.clone();
        let tx = self.cmd.clone();
        tokio::spawn(async move {
            let before = tube.comments_identity();
            let r = match tube.comments(&client, &id).await {
                Ok(page) if page.read_as_account => {
                    // Account cambiato durante la lettura: la pagina non è di nessuno.
                    if before.is_some() && before == tube.comments_identity() {
                        Ok((page, before))
                    } else {
                        Err(anyhow!("L'account è cambiato durante il caricamento."))
                    }
                }
                Ok(page) => Ok((page, None)),
                Err(e) => Err(e.into()),
            };
            let _ = tx.send(Command::CommentsReady(gen, true, r));
        });
    }

    fn comments_ready(
        &mut self,
        gen: u64,
        replace: bool,
        r: Result<(innertube::CommentsPage, Option<String>)>,
    ) {
        if gen != self.comments.gen {
            return;
        }
        self.comments_busy = false;
        match r {
            Ok((page, identity)) => {
                if replace {
                    self.comments.as_account = page.read_as_account;
                    self.comments.identity = identity;
                }
                self.comments.apply_page(page, replace);
                self.comments_avatars();
            }
            Err(e) => {
                tracing::debug!(error = %e, "commenti non caricati");
                if replace {
                    self.comments.view = Some(cm::View::Failed);
                } else {
                    self.status("Non riesco a caricare altri commenti.", false);
                }
            }
        }
        self.comments_ui(replace);
    }

    /// Scarica in background le foto degli autori che mancano, poi aggiorna la scheda.
    fn comments_avatars(&self) {
        let cache = self.data.join("covers");
        let missing: Vec<String> = self
            .comments
            .avatar_urls()
            .into_iter()
            .filter(|u| !cover_files(&cache, u).1.exists())
            .collect();
        if missing.is_empty() {
            return;
        }
        let data = self.data.clone();
        let tx = self.cmd.clone();
        let gen = self.comments.gen;
        tokio::spawn(async move {
            covers_for(missing, &data).await;
            let _ = tx.send(Command::CommentAvatars(gen));
        });
    }

    fn comments_more(&mut self) {
        if self.comments_busy {
            return;
        }
        let Some(token) = self.comments.continuation.clone() else {
            return;
        };
        let Some(as_account) = self.comments_origin() else {
            return;
        };
        self.comments_busy = true;
        self.comments_page(token, as_account, false);
    }

    fn comments_sort(&mut self, newest: bool) {
        if newest == self.comments.newest || self.comments_busy {
            return;
        }
        let Some(token) = self.comments.sort_token(newest) else {
            return;
        };
        let Some(as_account) = self.comments_origin() else {
            return;
        };
        self.comments_busy = true;
        self.comments.newest = newest;
        self.comments.items.clear();
        self.comments.continuation = None;
        self.comments.view = Some(cm::View::Loading);
        self.comments_ui(true);
        self.comments_page(token, as_account, true);
    }

    fn comments_page(&self, token: String, as_account: bool, replace: bool) {
        let tube = self.tube.clone();
        let client = self.client.clone();
        let tx = self.cmd.clone();
        let gen = self.comments.gen;
        let identity = self.comments.identity.clone();
        tokio::spawn(async move {
            let r = tube
                .comments_continuation(&client, &token, as_account)
                .await
                .map(|p| (p, identity))
                .map_err(Into::into);
            let _ = tx.send(Command::CommentsReady(gen, replace, r));
        });
    }

    fn comment_replies(&mut self, i: usize) {
        if self.comments_busy {
            return;
        }
        let Some(t) = self.comments.thread_of(i) else {
            return;
        };
        let item = &self.comments.items[t];
        let Some(token) = item.replies_token.clone() else {
            return;
        };
        let id = item.comment.id.clone();
        let Some(as_account) = self.comments_origin() else {
            return;
        };
        self.comments_busy = true;
        let tube = self.tube.clone();
        let client = self.client.clone();
        let tx = self.cmd.clone();
        let gen = self.comments.gen;
        tokio::spawn(async move {
            let r = tube
                .comment_replies(&client, &token, as_account)
                .await
                .map_err(Into::into);
            let _ = tx.send(Command::RepliesReady(gen, id, r));
        });
    }

    fn replies_ready(&mut self, gen: u64, id: String, r: Result<innertube::CommentReplies>) {
        if gen != self.comments.gen {
            return;
        }
        self.comments_busy = false;
        match r {
            Ok(page) => {
                self.comments.apply_replies(&id, page);
                self.comments_avatars();
            }
            Err(e) => {
                tracing::debug!(error = %e, "risposte non caricate");
                self.status("Non riesco a caricare le risposte.", false);
            }
        }
        self.comments_ui(false);
    }

    fn comment_like(&mut self, i: usize) {
        if !self.tube.is_logged_in() {
            self.status("Accedi per mettere mi piace ai commenti.", false);
            return;
        }
        if self.comments_origin() != Some(true) {
            return;
        }
        let Some((id, action, token)) = self.comments.vote_for(i) else {
            return;
        };
        self.comments.voting.push(id.clone());
        let tube = self.tube.clone();
        let client = self.client.clone();
        let tx = self.cmd.clone();
        let gen = self.comments.gen;
        tokio::spawn(async move {
            let r = tube
                .comment_action(&client, &token, action.name())
                .await
                .map_err(Into::into);
            let _ = tx.send(Command::VoteReady(gen, id, action, r));
        });
    }

    fn vote_ready(
        &mut self,
        gen: u64,
        id: String,
        action: innertube::CommentAction,
        r: Result<()>,
    ) {
        if gen != self.comments.gen {
            return;
        }
        if let Err(e) = &r {
            tracing::debug!(error = %e, "mi piace al commento non riuscito");
            self.status("Mi piace non riuscito: riprova.", false);
        }
        self.comments.voted(&id, action, r.is_ok());
        self.comments_ui(false);
    }

    fn comment_send(&mut self, text: String, to: i32) {
        if self.comments.sending {
            return;
        }
        let Some(text) = cm::clean_text(&text) else {
            return;
        };
        if !self.tube.is_logged_in() {
            self.status("Accedi per scrivere un commento.", false);
            return;
        }
        if self.comments_origin() != Some(true) {
            self.status("L'account è cambiato: ricarico i commenti.", false);
            return;
        }
        let (command, target) = if to >= 0 {
            let Some(t) = self.comments.thread_of(to as usize) else {
                return;
            };
            let c = &self.comments.items[t].comment;
            match c.commands.reply.clone() {
                Some(cmd) => (cmd, Some(c.id.clone())),
                None => {
                    self.status("A questo commento non si può rispondere.", false);
                    return;
                }
            }
        } else {
            match self.comments.create.clone() {
                Some(cmd) => (cmd, None),
                None => {
                    self.status("Questo brano non accetta commenti dal tuo account.", false);
                    return;
                }
            }
        };
        self.comments.sending = true;
        self.comments_ui(false);
        let tube = self.tube.clone();
        let client = self.client.clone();
        let tx = self.cmd.clone();
        let gen = self.comments.gen;
        tokio::spawn(async move {
            let r = tube
                .comment_write(&client, &command, Some(text.as_str()), false)
                .await
                .map(|answer| innertube::parse_written_comment(&answer))
                .map_err(Into::into);
            let _ = tx.send(Command::SentReady(gen, target, r));
        });
    }

    fn sent_ready(
        &mut self,
        gen: u64,
        target: Option<String>,
        r: Result<Option<innertube::CommentThread>>,
    ) {
        if gen != self.comments.gen {
            // L'elenco è cambiato nel frattempo (altro brano, account, ordine), ma il commento
            // è comunque partito: il campo va svuotato, o un secondo invio lo duplicherebbe.
            if r.is_ok() {
                self.send(Update::CommentSent);
                self.status("Commento pubblicato.", false);
            }
            return;
        }
        self.comments.sending = false;
        if r.is_ok() {
            self.send(Update::CommentSent);
        }
        match r {
            Ok(Some(thread)) => {
                match &target {
                    Some(id) => self.comments.add_reply(id, thread.comment),
                    None => self.comments.add_thread(thread),
                }
                self.status(
                    if target.is_some() {
                        "Risposta pubblicata."
                    } else {
                        "Commento pubblicato."
                    },
                    false,
                );
                self.comments_avatars();
            }
            Ok(None) => {
                // Pubblicato, ma la risposta di YouTube non lo conteneva: si ricarica l'elenco.
                self.status("Pubblicato.", false);
                self.comments.clear();
                self.comments_load();
                return;
            }
            Err(e) => {
                tracing::debug!(error = %e, "commento non pubblicato");
                self.status("Commento non pubblicato: riprova tra poco.", false);
            }
        }
        self.comments_ui(false);
    }
}

/// File in cache (240 px e 96 px) della copertina di un URL.
fn cover_files(cache: &std::path::Path, url: &str) -> (PathBuf, PathBuf) {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut h);
    let key = h.finish();
    (
        cache.join(format!("v2-{key:x}.png")), // v2: miniature chieste in grande
        cache.join(format!("v2-{key:x}-s.png")),
    )
}

async fn covers_for(
    urls: Vec<String>,
    data: &std::path::Path,
) -> HashMap<String, (PathBuf, PathBuf)> {
    use futures_util::{stream, StreamExt};
    let cache = data.join("covers");
    let _ = std::fs::create_dir_all(&cache);
    let mut urls = urls;
    urls.sort();
    urls.dedup();
    let results: HashMap<String, (PathBuf, PathBuf)> = stream::iter(urls)
        .map(|url| {
            let cache = cache.clone();
            async move {
                let (big, small) = cover_files(&cache, &url);
                if big.exists() && small.exists() {
                    return Some((url, (big, small)));
                }
                let img = if big.exists() {
                    image::open(&big).ok()?
                } else {
                    // Prima la versione grande chiesta al server, poi l'URL originale.
                    let mut got = None;
                    for candidate in crate::art::list_candidates(&url, 320) {
                        if let Some(img) = download_image(&candidate).await {
                            got = Some(img);
                            break;
                        }
                    }
                    let img = got?;
                    // Le miniature video sono 16:9: ritaglio centrale quadrato come le copertine.
                    let side = img.width().min(img.height()).clamp(1, 240);
                    let img = img.resize_to_fill(side, side, image::imageops::FilterType::Lanczos3);
                    img.save(&big).ok()?;
                    img
                };
                img.resize_to_fill(96, 96, image::imageops::FilterType::Lanczos3)
                    .save(&small)
                    .ok()?;
                Some((url, (big, small)))
            }
        })
        .buffer_unordered(8)
        .filter_map(|v| async { v })
        .collect()
        .await;
    // Cache limitata per numero di file (due per copertina).
    if let Ok(entries) = std::fs::read_dir(&cache) {
        let mut paths: Vec<_> = entries
            .flatten()
            .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
            .collect();
        if paths.len() > 1600 {
            paths.sort_by_key(|e| e.0);
            let n = paths.len() - 1600;
            for (_, path) in paths.into_iter().take(n) {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    results
}

/// Scarica e decodifica un'immagine dagli host di YouTube (mai con i cookie dell'account).
async fn download_image(url: &str) -> Option<image::DynamicImage> {
    use futures_util::StreamExt;
    const LIMIT: usize = 4 * 1024 * 1024;
    let host = reqwest::Url::parse(url).ok()?.host_str()?.to_owned();
    if !["ytimg.com", "googleusercontent.com", "ggpht.com"]
        .iter()
        .any(|d| host == *d || host.ends_with(&format!(".{d}")))
    {
        return None;
    }
    let response = crate::http::client()
        .get(url)
        .timeout(Duration::from_secs(6))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    if response.content_length().is_some_and(|n| n > LIMIT as u64) {
        return None;
    }
    let mut bytes = Vec::new();
    let mut chunks = response.bytes_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.ok()?;
        if bytes.len() + chunk.len() > LIMIT {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    let img = image::load_from_memory(&bytes).ok()?;
    // i.ytimg risponde a una dimensione mancante con un grigio 120x90.
    (!crate::art::is_missing_placeholder(img.width(), img.height())).then_some(img)
}

/// Foto del profilo: scaricata da yt3/lh3 (96 px), salvata in `avatar.png` nella cartella dati.
async fn fetch_avatar(url: &str, data: &std::path::Path) -> Option<PathBuf> {
    let mut img = None;
    for candidate in crate::art::list_candidates(url, 96) {
        if let Some(i) = download_image(&candidate).await {
            img = Some(i);
            break;
        }
    }
    let img = img?.resize_to_fill(96, 96, image::imageops::FilterType::Lanczos3);
    let path = data.join("avatar.png");
    img.save(&path).ok()?;
    Some(path)
}

async fn load_covers(p: &mut Page, data: &std::path::Path) {
    let mut urls: Vec<String> = p
        .rows
        .iter()
        .take(500)
        .chain(p.shelves.iter().flat_map(|s| s.items.iter()))
        .filter_map(|e| e.thumb.clone())
        .collect();
    if let Some(h) = p.hero.as_ref().and_then(|h| h.thumb.clone()) {
        urls.push(h);
    }
    let found = covers_for(urls, data).await;
    for e in p
        .rows
        .iter_mut()
        .chain(p.shelves.iter_mut().flat_map(|s| s.items.iter_mut()))
    {
        if let Some((big, small)) = e.thumb.as_ref().and_then(|u| found.get(u)) {
            e.cover = Some(big.clone());
            e.small = Some(small.clone());
        }
    }
    if let Some(h) = p.hero.as_mut() {
        h.cover = h
            .thumb
            .as_ref()
            .and_then(|u| found.get(u))
            .map(|(b, _)| b.clone());
    }
}

async fn load_entry_covers(list: &mut [Entry], data: &std::path::Path) {
    let found = covers_for(list.iter().filter_map(|e| e.thumb.clone()).collect(), data).await;
    for e in list.iter_mut() {
        if let Some((big, small)) = e.thumb.as_ref().and_then(|u| found.get(u)) {
            e.cover = Some(big.clone());
            e.small = Some(small.clone());
        }
    }
}

/// Scarica e mette in cache la copertina grande (720 px, quadrata) e ne calcola il colore.
/// Solo dagli host delle immagini di YouTube, senza cookie dell'account.
async fn fetch_art(url: &str, data: &std::path::Path) -> Option<(PathBuf, [u8; 3])> {
    use futures_util::StreamExt;
    use std::hash::{Hash, Hasher};
    const LIMIT: usize = 8 * 1024 * 1024;
    let dir = data.join("art");
    let _ = std::fs::create_dir_all(&dir);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut h);
    let path = dir.join(format!("{:x}.jpg", h.finish()));
    let colour = |img: &image::DynamicImage| {
        let small = img
            .resize_exact(24, 24, image::imageops::FilterType::Triangle)
            .to_rgb8();
        let px: Vec<[u8; 3]> = small.pixels().map(|p| p.0).collect();
        crate::art::tint(&px)
    };
    if path.exists() {
        if let Ok(img) = image::open(&path) {
            return Some((path, colour(&img)));
        }
    }
    for candidate in crate::art::candidates(url, 720) {
        let Some(host) = reqwest::Url::parse(&candidate)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned))
        else {
            continue;
        };
        if !["ytimg.com", "googleusercontent.com", "ggpht.com"]
            .iter()
            .any(|d| host == *d || host.ends_with(&format!(".{d}")))
        {
            continue;
        }
        let Ok(response) = crate::http::client()
            .get(&candidate)
            .timeout(Duration::from_secs(10))
            .send()
            .await
        else {
            continue;
        };
        let Ok(response) = response.error_for_status() else {
            continue;
        };
        let mut bytes = Vec::new();
        let mut chunks = response.bytes_stream();
        let mut too_big = false;
        while let Some(chunk) = chunks.next().await {
            let Ok(chunk) = chunk else {
                too_big = true;
                break;
            };
            if bytes.len() + chunk.len() > LIMIT {
                too_big = true;
                break;
            }
            bytes.extend_from_slice(&chunk);
        }
        if too_big {
            continue;
        }
        let Ok(img) = image::load_from_memory(&bytes) else {
            continue;
        };
        if crate::art::is_missing_placeholder(img.width(), img.height()) {
            continue;
        }
        // Le miniature video sono 16:9: si ritaglia il centro, come `object-cover` in Limusic.
        let side = img.width().min(img.height()).clamp(1, 720);
        let square = img.resize_to_fill(side, side, image::imageops::FilterType::Lanczos3);
        let tint = colour(&square);
        if image::DynamicImage::ImageRgb8(square.to_rgb8())
            .save(&path)
            .is_err()
        {
            continue;
        }
        // Cache limitata: le copertine grandi più vecchie se ne vanno oltre le 120.
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut files: Vec<_> = entries
                .flatten()
                .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
                .collect();
            if files.len() > 120 {
                files.sort_by_key(|e| e.0);
                let n = files.len() - 120;
                for (_, old) in files.into_iter().take(n) {
                    let _ = std::fs::remove_file(old);
                }
            }
        }
        return Some((path, tint));
    }
    None
}
