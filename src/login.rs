//! Accesso Google integrato.
//!
//! La pagina di accesso gira in un **processo separato** (lo stesso eseguibile avviato con
//! `--login-window`, `--add-account-window` o `--refresh-session`): una finestra tao con WebView2
//! (wry). Così il suo ciclo di eventi non si scontra con quello di Slint/winit della finestra
//! principale, e chiudere o terminare il login non tocca il player.
//!
//! Il processo figlio scrive su stdout una sola riga, `AMBRA-COOKIE\t<Cookie header>` oppure
//! `AMBRA-ERROR\t<messaggio>`, ed esce. La password non passa mai dall'app: viene digitata nella
//! pagina ufficiale di Google; all'app arrivano solo i cookie di youtube.com.
//!
//! Il profilo WebView2 è persistente (`<dati>/webview`): Google ricorda l'accesso, quindi un nuovo
//! login è un clic e la sessione si può rinnovare in background quando i cookie ruotano (stessa
//! strategia di Limusic, `session.rs` dell'originale).

use std::path::{Path, PathBuf};
use std::time::Duration;

pub const MARK_COOKIE: &str = "AMBRA-COOKIE\t";
pub const MARK_ERROR: &str = "AMBRA-ERROR\t";

/// Il login integrato richiede WebView2, quindi Windows.
pub const SUPPORTED: bool = cfg!(windows);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Accesso normale: se il profilo è già collegato a Google basta un clic.
    SignIn,
    /// "Cambia account": svuota prima la sessione Google del profilo e apre AddSession.
    AddAccount,
    /// Rinnovo invisibile: apre music.youtube.com con il profilo esistente e rilegge i cookie.
    Refresh,
}

impl Mode {
    pub fn arg(self) -> &'static str {
        match self {
            Mode::SignIn => "--login-window",
            Mode::AddAccount => "--add-account-window",
            Mode::Refresh => "--refresh-session",
        }
    }
    pub fn from_args(args: &[String]) -> Option<Self> {
        [Mode::SignIn, Mode::AddAccount, Mode::Refresh]
            .into_iter()
            .find(|m| args.iter().any(|a| a == m.arg()))
    }
    fn timeout(self) -> Duration {
        match self {
            // Il tempo di digitare password e codice 2FA.
            Mode::SignIn | Mode::AddAccount => Duration::from_secs(15 * 60),
            Mode::Refresh => Duration::from_secs(60),
        }
    }
}

/// Esito del processo di login visto dall'app.
#[derive(Debug)]
pub enum Outcome {
    Cookie(String),
    /// L'utente ha chiuso la finestra.
    Cancelled,
    Failed(String),
}

/// Cartella dati passata al processo figlio: `--data <dir>`.
pub fn data_arg(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .position(|a| a == "--data")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
}

/// Interpreta lo stdout del processo figlio.
pub fn parse_output(stdout: &str, exit_ok: bool) -> Outcome {
    for line in stdout.lines() {
        if let Some(c) = line.strip_prefix(MARK_COOKIE) {
            let c = crate::session::normalize(c);
            if innertube::cookie_sapisid(&c).is_some() {
                return Outcome::Cookie(c);
            }
            return Outcome::Failed("La sessione ottenuta non contiene SAPISID.".into());
        }
        if let Some(e) = line.strip_prefix(MARK_ERROR) {
            return Outcome::Failed(e.trim().to_owned());
        }
    }
    if exit_ok {
        Outcome::Failed("La finestra di accesso non ha restituito una sessione.".into())
    } else {
        Outcome::Cancelled
    }
}

/// Avvia il processo di login e ne attende l'esito. Annullare il future chiude la finestra.
pub async fn run_helper(mode: Mode, data: &Path) -> Outcome {
    if !SUPPORTED {
        return Outcome::Failed(
            "Il login integrato è disponibile solo su Windows: usa il Cookie header.".into(),
        );
    }
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return Outcome::Failed(format!("Eseguibile non trovato: {e}")),
    };
    let mut command = tokio::process::Command::new(exe);
    command
        .arg(mode.arg())
        .arg("--data")
        .arg(data)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    // Nelle build di debug l'eseguibile è un'app console: niente finestra nera in più.
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    let child = command.spawn();
    let child = match child {
        Ok(c) => c,
        Err(e) => {
            return Outcome::Failed(format!("Impossibile aprire la finestra di accesso: {e}"))
        }
    };
    match tokio::time::timeout(mode.timeout(), child.wait_with_output()).await {
        Ok(Ok(out)) => parse_output(&String::from_utf8_lossy(&out.stdout), out.status.success()),
        Ok(Err(e)) => Outcome::Failed(format!("Finestra di accesso interrotta: {e}")),
        Err(_) => Outcome::Failed("Tempo scaduto per l'accesso.".into()),
    }
}

/// Punto d'ingresso del processo figlio (`main` lo chiama prima di creare la GUI).
#[cfg(windows)]
pub fn run_window(mode: Mode, data: PathBuf) -> ! {
    windows_impl::run(mode, data)
}
#[cfg(not(windows))]
pub fn run_window(mode: Mode, data: PathBuf) -> ! {
    let _ = (mode, data);
    emit(&format!(
        "{MARK_ERROR}Login integrato non disponibile su questo sistema."
    ));
    std::process::exit(2)
}

fn emit(line: &str) {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

/// `true` se l'URL punta a music.youtube.com (atterraggio dopo l'accesso).
pub fn is_music_url(url: &str) -> bool {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or("");
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host);
    host.eq_ignore_ascii_case("music.youtube.com")
}

#[cfg(windows)]
mod windows_impl {
    use super::{emit, is_music_url, Mode, MARK_COOKIE, MARK_ERROR};
    use std::path::PathBuf;
    use std::time::{Duration, Instant};
    use tao::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoopBuilder},
        window::{Icon, WindowBuilder},
    };
    use wry::{PageLoadEvent, WebContext, WebViewBuilder};

    /// Pagina di accesso Google con ritorno a YouTube Music (il ritorno è il segnale di fine).
    const LOGIN_URL: &str =
        "https://accounts.google.com/ServiceLogin?service=youtube&continue=https://music.youtube.com/";
    /// "Aggiungi account" di Google: mostra la schermata di accesso anche con una sessione attiva.
    const ADD_ACCOUNT_URL: &str =
        "https://accounts.google.com/AddSession?service=youtube&continue=https://music.youtube.com/";
    const REFRESH_URL: &str = "https://music.youtube.com/";

    enum UserEvent {
        Landed,
    }

    fn fail(msg: &str) -> ! {
        emit(&format!("{MARK_ERROR}{msg}"));
        std::process::exit(2)
    }

    fn window_icon() -> Option<Icon> {
        let img = image::load_from_memory(include_bytes!("../assets/icon.png")).ok()?;
        let img = img
            .resize_exact(64, 64, image::imageops::FilterType::Lanczos3)
            .to_rgba8();
        Icon::from_rgba(img.into_raw(), 64, 64).ok()
    }

    /// Cookie della finestra trasformati nel Cookie header di music.youtube.com.
    fn read_jar(webview: &wry::WebView) -> String {
        let Ok(cookies) = webview.cookies() else {
            return String::new();
        };
        crate::session::youtube_cookie_header(cookies.iter().map(|c| {
            (
                c.domain().unwrap_or_default().to_owned(),
                c.name().to_owned(),
                c.value().to_owned(),
            )
        }))
    }

    /// Svuota la sessione Google/YouTube del profilo (per "Cambia account").
    fn clear_google(webview: &wry::WebView) -> Result<(), String> {
        let cookies = webview.cookies().map_err(|e| e.to_string())?;
        for cookie in cookies.iter() {
            let d = cookie
                .domain()
                .unwrap_or_default()
                .trim_start_matches('.')
                .to_owned();
            let ours = ["google.com", "youtube.com"]
                .iter()
                .any(|s| d == *s || d.ends_with(&format!(".{s}")));
            if ours {
                webview.delete_cookie(cookie).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn run(mode: Mode, data: PathBuf) -> ! {
        let refresh = mode == Mode::Refresh;
        let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
        let proxy = event_loop.create_proxy();
        let window = WindowBuilder::new()
            .with_title(if mode == Mode::AddAccount {
                "Aggiungi un account · AmbraMusic"
            } else {
                "Accedi a YouTube Music · AmbraMusic"
            })
            .with_inner_size(LogicalSize::new(500.0, 720.0))
            .with_min_inner_size(LogicalSize::new(380.0, 520.0))
            .with_visible(!refresh)
            .with_focused(!refresh)
            .with_window_icon(window_icon())
            .build(&event_loop)
            .unwrap_or_else(|e| fail(&format!("Impossibile creare la finestra: {e}")));

        // Profilo persistente di WebView2, separato da Edge e dagli altri programmi.
        let _ = std::fs::create_dir_all(data.join("webview"));
        let mut context = WebContext::new(Some(data.join("webview")));
        let start = match mode {
            Mode::SignIn => LOGIN_URL,
            Mode::AddAccount => "about:blank",
            Mode::Refresh => REFRESH_URL,
        };
        // Nessun cambio di user agent: su WebView2 quello predefinito (Edge) è coerente con i
        // client hints, e Google rifiuta i browser che si contraddicono ("browser non sicuro").
        let webview = WebViewBuilder::new_with_web_context(&mut context)
            .with_url(start)
            .with_devtools(false)
            .with_on_page_load_handler(move |event, url| {
                if matches!(event, PageLoadEvent::Finished) && is_music_url(&url) {
                    let _ = proxy.send_event(UserEvent::Landed);
                }
            })
            .build(&window)
            .unwrap_or_else(|e| {
                fail(&format!(
                    "WebView2 non disponibile ({e}). Installa \"Microsoft Edge WebView2 Runtime\"."
                ))
            });

        if mode == Mode::AddAccount {
            if let Err(e) = clear_google(&webview) {
                fail(&format!(
                    "Impossibile svuotare la sessione Google precedente: {e}"
                ));
            }
            if let Err(e) = webview.load_url(ADD_ACCOUNT_URL) {
                fail(&format!("Impossibile aprire la pagina di Google: {e}"));
            }
        }

        let deadline = Instant::now() + mode_deadline(mode);
        // Dopo l'atterraggio su music.youtube.com i cookie possono arrivare un attimo dopo:
        // si rileggono a intervalli. Nel rinnovo si aspetta un po' di più, perché è il caricamento
        // della pagina di YouTube a far ruotare i cookie.
        let first_delay = if refresh {
            Duration::from_millis(2500)
        } else {
            Duration::from_millis(400)
        };
        let mut next_poll: Option<Instant> = None;
        let mut polls = 0u32;

        event_loop.run(move |event, _, control_flow| {
            // Tengono in vita finestra e profilo per tutta la durata del ciclo.
            let _ = (&window, &context);
            let now = Instant::now();
            match event {
                Event::UserEvent(UserEvent::Landed) => {
                    if next_poll.is_none() {
                        next_poll = Some(now + first_delay);
                        polls = 0;
                    }
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::ExitWithCode(1);
                    return;
                }
                _ => {}
            }
            if now >= deadline {
                emit(&format!("{MARK_ERROR}Tempo scaduto per l'accesso."));
                *control_flow = ControlFlow::ExitWithCode(2);
                return;
            }
            if let Some(at) = next_poll {
                if now >= at {
                    let jar = read_jar(&webview);
                    if innertube::cookie_sapisid(&jar).is_some() {
                        emit(&format!("{MARK_COOKIE}{jar}"));
                        *control_flow = ControlFlow::ExitWithCode(0);
                        return;
                    }
                    polls += 1;
                    if polls >= 8 {
                        if refresh {
                            emit(&format!(
                                "{MARK_ERROR}La sessione Google della finestra integrata non è più attiva."
                            ));
                            *control_flow = ControlFlow::ExitWithCode(3);
                            return;
                        }
                        // Arrivati su YouTube Music ma non ancora collegati: si continua ad
                        // aspettare il prossimo atterraggio (l'utente sta ancora accedendo).
                        next_poll = None;
                    } else {
                        next_poll = Some(now + Duration::from_millis(500));
                    }
                }
            }
            let wake = next_poll.map_or(deadline, |p| p.min(deadline));
            *control_flow = ControlFlow::WaitUntil(wake);
        })
    }

    fn mode_deadline(mode: Mode) -> Duration {
        match mode {
            Mode::Refresh => Duration::from_secs(45),
            _ => Duration::from_secs(14 * 60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_only_the_music_host() {
        assert!(is_music_url("https://music.youtube.com/"));
        assert!(is_music_url("https://music.youtube.com/library?x=1"));
        assert!(is_music_url("https://MUSIC.youtube.com"));
        assert!(!is_music_url(
            "https://accounts.google.com/?continue=https://music.youtube.com/"
        ));
        assert!(!is_music_url("https://music.youtube.com.evil.example/"));
        assert!(!is_music_url("https://www.youtube.com/"));
        assert!(!is_music_url("about:blank"));
    }

    #[test]
    fn parses_the_child_output() {
        match parse_output("noise\nAMBRA-COOKIE\tSAPISID=a; SID=b\n", true) {
            Outcome::Cookie(c) => assert_eq!(c, "SAPISID=a; SID=b"),
            o => panic!("{o:?}"),
        }
        assert!(matches!(
            parse_output("AMBRA-COOKIE\tSID=b\n", true),
            Outcome::Failed(_)
        ));
        match parse_output("AMBRA-ERROR\tWebView2 mancante\n", false) {
            Outcome::Failed(e) => assert_eq!(e, "WebView2 mancante"),
            o => panic!("{o:?}"),
        }
        assert!(matches!(parse_output("", false), Outcome::Cancelled));
        assert!(matches!(parse_output("", true), Outcome::Failed(_)));
    }

    #[test]
    fn mode_and_data_arguments() {
        let args: Vec<String> = ["x", "--refresh-session", "--data", "C:\\d"]
            .map(String::from)
            .into();
        assert_eq!(Mode::from_args(&args), Some(Mode::Refresh));
        assert_eq!(data_arg(&args), Some(PathBuf::from("C:\\d")));
        assert_eq!(Mode::from_args(&["x".to_string()]), None);
    }
}
