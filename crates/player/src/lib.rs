//! Player audio di AmbraMusic.
//!
//! Un involucro sottile, scritto da noi, sull'API C di libmpv (`include/mpv/client.h`, API 2.x):
//! niente librerie di terzi in mezzo e solo quello che l'app usa — carica, play/pausa, stop,
//! posizione, volume ed eventi. libmpv è thread-safe (client.h: "The client API is generally
//! fully thread-safe"); gli eventi si leggono da un solo thread, quello creato qui.

mod logic;

use logic::{mpv as c, Ended};
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[repr(C)]
struct MpvHandle {
    _private: [u8; 0],
}

#[repr(C)]
struct MpvEvent {
    event_id: c_int,
    error: c_int,
    reply_userdata: u64,
    data: *mut c_void,
}

#[repr(C)]
struct MpvEventProperty {
    name: *const c_char,
    format: c_int,
    data: *mut c_void,
}

#[repr(C)]
struct MpvEventEndFile {
    reason: c_int,
    error: c_int,
    playlist_entry_id: i64,
    playlist_insert_id: i64,
    playlist_insert_num_entries: c_int,
}

#[link(name = "mpv")]
extern "C" {
    fn mpv_create() -> *mut MpvHandle;
    fn mpv_initialize(ctx: *mut MpvHandle) -> c_int;
    fn mpv_terminate_destroy(ctx: *mut MpvHandle);
    fn mpv_set_option_string(
        ctx: *mut MpvHandle,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int;
    fn mpv_set_property_string(
        ctx: *mut MpvHandle,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int;
    fn mpv_get_property_string(ctx: *mut MpvHandle, name: *const c_char) -> *mut c_char;
    fn mpv_command(ctx: *mut MpvHandle, args: *mut *const c_char) -> c_int;
    fn mpv_observe_property(
        ctx: *mut MpvHandle,
        reply_userdata: u64,
        name: *const c_char,
        format: c_int,
    ) -> c_int;
    fn mpv_wait_event(ctx: *mut MpvHandle, timeout: f64) -> *mut MpvEvent;
    fn mpv_wakeup(ctx: *mut MpvHandle);
    fn mpv_error_string(error: c_int) -> *const c_char;
    fn mpv_free(data: *mut c_void);
}

#[derive(Debug)]
pub enum Error {
    /// Codice di errore di libmpv e descrizione.
    Mpv(i32, String),
    /// libmpv non si avvia (DLL mancante o incompatibile).
    Init(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Mpv(code, text) => write!(f, "mpv: {text} ({code})"),
            Error::Init(text) => write!(f, "mpv non disponibile: {text}"),
        }
    }
}
impl std::error::Error for Error {}

fn describe(code: c_int) -> String {
    // SAFETY: mpv_error_string restituisce sempre una stringa statica valida.
    unsafe { CStr::from_ptr(mpv_error_string(code)) }
        .to_string_lossy()
        .into_owned()
}
fn check(code: c_int) -> Result<(), Error> {
    if code >= 0 {
        Ok(())
    } else {
        Err(Error::Mpv(code, describe(code)))
    }
}
fn cstr(s: &str) -> CString {
    // Le stringhe passate a mpv non contengono NUL; se ci fossero, si tagliano lì.
    CString::new(s.split('\0').next().unwrap_or_default()).unwrap_or_default()
}

/// Eventi verso l'app (stessi nomi usati dal backend).
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEvent {
    Position(f64),
    Duration(f64),
    /// In riproduzione (`true`) oppure in pausa / fermo.
    Playing(bool),
    TrackEnded,
    TrackFailed(String),
    AudioDeviceLost,
    Error(String),
}

/// Il puntatore a mpv, distrutto quando non serve più a nessuno.
struct Handle(*mut MpvHandle);
// SAFETY: l'API client di mpv è thread-safe; gli eventi li legge un solo thread.
unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}
impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: ultimo riferimento: nessun altro thread usa più il puntatore.
        unsafe { mpv_terminate_destroy(self.0) }
    }
}

pub struct Player {
    mpv: Arc<Handle>,
    events: Option<UnboundedReceiver<PlayerEvent>>,
    quit: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Player {
    /// Avvia libmpv per il solo audio, con la cache su disco in `cache_dir`.
    pub fn new(cache_dir: &str) -> Result<Self, Error> {
        // SAFETY: mpv_create non ha precondizioni; NULL = libreria non utilizzabile.
        let raw = unsafe { mpv_create() };
        if raw.is_null() {
            return Err(Error::Init("mpv_create ha restituito NULL".into()));
        }
        let mpv = Arc::new(Handle(raw));
        // Opzioni "al meglio": una che questa build di mpv non conosce non deve impedire
        // all'audio di partire. Solo la cartella della cache è indispensabile.
        let set = |name: &str, value: &str| -> Result<(), Error> {
            let (n, v) = (cstr(name), cstr(value));
            // SAFETY: puntatore valido, stringhe NUL-terminate vive per tutta la chiamata.
            check(unsafe { mpv_set_option_string(mpv.0, n.as_ptr(), v.as_ptr()) })
        };
        let soft = |name: &str, value: &str| {
            if let Err(e) = set(name, value) {
                eprintln!("mpv: opzione {name} ignorata ({e})");
            }
        };
        // Solo audio: niente video, niente finestra, niente OSD.
        soft("vid", "no");
        soft("video", "no");
        soft("vo", "null");
        soft("osd-level", "0");
        // Gli URL li risolve l'app: mpv non deve mai chiamare youtube-dl da solo.
        soft("ytdl", "no");
        soft("input-default-bindings", "no");
        soft("input-vo-keyboard", "no");
        soft("terminal", "no");
        // Nome nel mixer di Windows.
        soft("audio-client-name", "AmbraMusic");
        soft("title", "AmbraMusic");
        soft("gapless-audio", "yes");
        // Il demuxer di FFmpeg gestisce meglio il padding finale degli Opus di YouTube
        // (evita lo "schiocco" tra un brano e l'altro).
        soft("demuxer", "lavf");
        // Cache dei pacchetti su disco: la RAM resta bassa anche con brani lunghi.
        soft("cache", "yes");
        soft("cache-on-disk", "yes");
        set("demuxer-cache-dir", cache_dir)?;
        soft("demuxer-max-bytes", "16MiB");
        soft("demuxer-max-back-bytes", "8MiB");
        // Dopo un salto riparte con 0,3 s di audio invece di 1 s.
        soft("cache-pause-wait", "0.3");
        // Rete: pochi tentativi di riconnessione (poi l'errore arriva all'app, che riprova con un
        // nuovo URL) e ogni salto oltre la cache apre una nuova richiesta Range invece di
        // "scaricare" il resto della risposta.
        soft(
            "stream-lavf-o",
            "reconnect=1,reconnect_streamed=1,reconnect_on_network_error=1,reconnect_delay_max=2,reconnect_max_retries=6,reconnect_delay_total_max=30,short_seek_size=1",
        );
        soft("idle", "yes");
        // SAFETY: handle creato sopra e non ancora inizializzato.
        check(unsafe { mpv_initialize(mpv.0) }).map_err(|e| Error::Init(e.to_string()))?;
        for (id, name, format) in [
            (1u64, "time-pos", c::FORMAT_DOUBLE),
            (2, "duration", c::FORMAT_DOUBLE),
            (3, "pause", c::FORMAT_FLAG),
            (4, "idle-active", c::FORMAT_FLAG),
        ] {
            let n = cstr(name);
            // SAFETY: handle inizializzato, nome NUL-terminato.
            check(unsafe { mpv_observe_property(mpv.0, id, n.as_ptr(), format) })?;
        }
        let (tx, rx) = unbounded_channel();
        let quit = Arc::new(AtomicBool::new(false));
        let thread = {
            let mpv = mpv.clone();
            let quit = quit.clone();
            std::thread::Builder::new()
                .name("ambra-mpv-events".into())
                .spawn(move || event_loop(mpv, tx, quit))
                .map_err(|e| Error::Init(e.to_string()))?
        };
        Ok(Self {
            mpv,
            events: Some(rx),
            quit,
            thread: Some(thread),
        })
    }

    /// Gli eventi del player (si possono prendere una volta sola).
    pub fn take_events(&mut self) -> Option<UnboundedReceiver<PlayerEvent>> {
        self.events.take()
    }

    fn command(&self, args: &[&str]) -> Result<(), Error> {
        let owned: Vec<CString> = args.iter().map(|a| cstr(a)).collect();
        let mut ptrs: Vec<*const c_char> = owned.iter().map(|a| a.as_ptr()).collect();
        ptrs.push(std::ptr::null());
        // SAFETY: array NULL-terminato di stringhe vive fino alla fine della chiamata.
        check(unsafe { mpv_command(self.mpv.0, ptrs.as_mut_ptr()) })
    }
    fn set_property(&self, name: &str, value: &str) -> Result<(), Error> {
        let (n, v) = (cstr(name), cstr(value));
        // SAFETY: come sopra.
        check(unsafe { mpv_set_property_string(self.mpv.0, n.as_ptr(), v.as_ptr()) })
    }
    fn get_property(&self, name: &str) -> Option<String> {
        let n = cstr(name);
        // SAFETY: la stringa restituita appartiene a mpv e va liberata con mpv_free.
        unsafe {
            let p = mpv_get_property_string(self.mpv.0, n.as_ptr());
            if p.is_null() {
                return None;
            }
            let s = CStr::from_ptr(p).to_string_lossy().into_owned();
            mpv_free(p as *mut c_void);
            Some(s)
        }
    }

    /// Carica un brano (sostituisce quello attuale) con le intestazioni HTTP indicate.
    pub fn load(&self, url: &str, headers: &HashMap<String, String>) -> Result<(), Error> {
        // Un brano in pausa sostituito di colpo può far sentire un attimo del vecchio quando
        // parte il nuovo: prima lo si ferma e si aspetta che mpv lo abbia davvero chiuso
        // (`path` sparisce solo dopo).
        if self.get_property("pause").as_deref() == Some("yes") {
            self.command(&["stop"])?;
            for _ in 0..50 {
                if self.get_property("path").is_none() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        let (agent, fields) = logic::split_headers(headers);
        // Lo user agent si cambia solo se l'app ne indica uno (come prima); le altre
        // intestazioni si sostituiscono sempre, anche con una lista vuota.
        if let Some(agent) = agent {
            self.set_property("user-agent", &agent)?;
        }
        self.set_property("http-header-fields", &fields)?;
        self.command(&["loadfile", url, "replace"])
    }
    pub fn play(&self) -> Result<(), Error> {
        self.set_property("pause", "no")
    }
    pub fn pause(&self) -> Result<(), Error> {
        self.set_property("pause", "yes")
    }
    pub fn toggle(&self) -> Result<(), Error> {
        self.command(&["cycle", "pause"])
    }
    /// Ferma e svuota: mpv resta inattivo finché non si carica altro.
    pub fn stop(&self) -> Result<(), Error> {
        self.command(&["stop"])
    }
    /// Salto assoluto, in secondi.
    pub fn seek(&self, seconds: f64) -> Result<(), Error> {
        if !seconds.is_finite() {
            return Ok(());
        }
        self.command(&["seek", &format!("{:.3}", seconds.max(0.0)), "absolute"])
    }
    /// Volume del cursore, 0–100 (curva in dB, vedi `logic::mpv_volume`).
    pub fn set_volume(&self, percent: i64) -> Result<(), Error> {
        self.set_property("volume", &format!("{:.3}", logic::mpv_volume(percent)))
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.quit.store(true, Ordering::SeqCst);
        // SAFETY: handle ancora vivo (lo tiene anche il thread degli eventi).
        unsafe { mpv_wakeup(self.mpv.0) };
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// Legge gli eventi di mpv e li traduce per l'app.
fn event_loop(mpv: Arc<Handle>, tx: UnboundedSender<PlayerEvent>, quit: Arc<AtomicBool>) {
    let mut paused = false;
    let mut idle = true;
    let mut playing = false;
    let mut last_pos: Option<f64> = None;
    while !quit.load(Ordering::SeqCst) {
        // SAFETY: un solo thread chiama mpv_wait_event; l'evento resta valido fino alla
        // chiamata successiva.
        let ev = unsafe { &*mpv_wait_event(mpv.0, 0.5) };
        let out: Option<PlayerEvent> = match ev.event_id {
            c::EVENT_NONE => None,
            c::EVENT_SHUTDOWN => break,
            c::EVENT_PROPERTY_CHANGE if !ev.data.is_null() => {
                // SAFETY: per PROPERTY_CHANGE `data` è un mpv_event_property.
                let prop = unsafe { &*(ev.data as *const MpvEventProperty) };
                let name = unsafe { CStr::from_ptr(prop.name) }.to_bytes();
                let double = || {
                    (prop.format == c::FORMAT_DOUBLE && !prop.data.is_null())
                        // SAFETY: formato DOUBLE → data punta a un f64.
                        .then(|| unsafe { *(prop.data as *const f64) })
                };
                let flag = || {
                    (prop.format == c::FORMAT_FLAG && !prop.data.is_null())
                        // SAFETY: formato FLAG → data punta a un int.
                        .then(|| unsafe { *(prop.data as *const c_int) } != 0)
                };
                match name {
                    b"time-pos" => double().and_then(|p| {
                        logic::position_due(last_pos, p).then(|| {
                            last_pos = Some(p);
                            PlayerEvent::Position(p)
                        })
                    }),
                    b"duration" => double().map(PlayerEvent::Duration),
                    b"pause" => {
                        paused = flag().unwrap_or(paused);
                        None
                    }
                    b"idle-active" => {
                        idle = flag().unwrap_or(idle);
                        if idle {
                            last_pos = None;
                        }
                        None
                    }
                    _ => None,
                }
            }
            c::EVENT_END_FILE if !ev.data.is_null() => {
                // SAFETY: per END_FILE `data` è un mpv_event_end_file.
                let end = unsafe { &*(ev.data as *const MpvEventEndFile) };
                last_pos = None;
                match logic::end_file(end.reason, end.error) {
                    Ended::Finished => Some(PlayerEvent::TrackEnded),
                    Ended::DeviceLost => Some(PlayerEvent::AudioDeviceLost),
                    Ended::Failed(code) => Some(PlayerEvent::TrackFailed(describe(code))),
                    Ended::Stopped => None,
                }
            }
            _ => None,
        };
        if let Some(e) = out {
            if tx.send(e).is_err() {
                break;
            }
        }
        let now = !paused && !idle;
        if now != playing {
            playing = now;
            if tx.send(PlayerEvent::Playing(now)).is_err() {
                break;
            }
        }
    }
}
