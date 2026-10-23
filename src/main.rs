#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod art;
mod audioproxy;
mod backend;
mod comments;
mod desktop;
mod discord;
mod http;
mod login;
mod orchestrator;
mod pages;
mod session;
#[cfg(windows)]
mod single_instance;
mod ui_models;
use backend::{Command, Settings, Shelf, Update};
use desktop::{data_dir, open_browser, open_url, trim_memory};
use slint::{CloseRequestResponse, ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::{collections::HashSet, rc::Rc};
use ui_models::{colour, find_track, for_each_list, load, row, rows, shelf, time};
slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    // Processo figlio della finestra di accesso Google: nessuna GUI Slint.
    if let Some(mode) = login::Mode::from_args(&args) {
        let data = login::data_arg(&args).unwrap_or_else(data_dir);
        login::run_window(mode, data);
    }
    #[cfg(windows)]
    let instance = match single_instance::Instance::acquire()? {
        Some(instance) => std::rc::Rc::new(instance),
        None => return Ok(()),
    };
    let demo = args.iter().any(|a| a == "--demo");
    let data = data_dir();
    std::fs::create_dir_all(&data)?;
    let settings: Settings = std::fs::read(data.join("settings.json"))
        .ok()
        .and_then(|s| serde_json::from_slice(&s).ok())
        .unwrap_or_default();
    let ui = AppWindow::new()?;
    ui.set_demo(demo);
    let version = env!("CARGO_PKG_VERSION");
    ui.set_version(version.into());
    ui.set_volume(settings.volume.clamp(0, 100) as f32);
    ui.set_discord_enabled(settings.discord);
    ui.set_close_to_tray(settings.close_to_tray);
    ui.set_playlists_right(settings.playlists_right);
    ui.set_open_player(settings.open_player);
    ui.set_autoplay(settings.autoplay);
    ui.set_comments_enabled(settings.comments);
    ui.set_google_login_available(login::SUPPORTED && !demo);
    ui.set_session_saved(session::Store::new(&data).exists());
    ui.invoke_apply_theme(settings.dark());

    let (updates, receiver) = std::sync::mpsc::channel();
    let (cmd, worker) = backend::start(settings.clone(), data, demo, updates);
    macro_rules! bind0 {
        ($on:ident, $cmd:expr) => {{
            let tx = cmd.clone();
            ui.$on(move || {
                let _ = tx.send($cmd);
            });
        }};
    }
    macro_rules! bind1 {
        ($on:ident, |$a:ident| $cmd:expr) => {{
            let tx = cmd.clone();
            ui.$on(move |$a| {
                let _ = tx.send($cmd);
            });
        }};
    }
    macro_rules! bind2 {
        ($on:ident, |$a:ident, $b:ident| $cmd:expr) => {{
            let tx = cmd.clone();
            ui.$on(move |$a, $b| {
                let _ = tx.send($cmd);
            });
        }};
    }
    bind0!(on_toggle, Command::Toggle);
    bind0!(on_logout, Command::Logout);
    bind0!(on_like_current, Command::LikeCurrent);
    bind0!(on_load_more, Command::More);
    bind0!(on_cancel_login, Command::CancelLogin);
    bind0!(on_back, Command::Back);
    bind0!(on_clear_cache, Command::ClearCache);
    bind0!(on_comments_open, Command::CommentsOpen);
    bind0!(on_comments_more, Command::CommentsMore);
    bind1!(on_panel_tab, |t| Command::PanelTab(t));
    bind1!(on_comments_sort, |newest| Command::CommentsSort(newest));
    bind1!(on_comment_like, |i| Command::CommentLike(i.max(0) as usize));
    bind1!(on_comment_replies, |i| Command::CommentReplies(
        i.max(0) as usize
    ));
    bind2!(on_comment_send, |text, to| Command::CommentSend(
        text.into(),
        to
    ));
    bind0!(on_previous, Command::Skip(-1));
    bind0!(on_next, Command::Skip(1));
    bind1!(on_search, |q| Command::Search(q.into()));
    bind1!(on_suggest, |t| Command::Suggest(t.into()));
    bind1!(on_suggest_pick, |i| Command::SuggestPick(i.max(0) as usize));
    bind1!(on_search_filter, |f| Command::SearchFilter(f.into()));
    bind1!(on_navigate, |p| Command::Go(p));
    bind1!(on_activate, |i| Command::Open(i.max(0) as usize));
    bind1!(on_open_library, |i| Command::OpenLibrary(i.max(0) as usize));
    bind1!(on_play_all, |shuffle| Command::PlayAll(shuffle));
    bind1!(on_enqueue, |i| Command::Add(i.max(0) as usize));
    bind1!(on_like_row, |i| Command::Like(i.max(0) as usize));
    bind1!(on_play_queue, |i| Command::PlayQueue(i.max(0) as usize));
    bind1!(on_remove_queue, |i| Command::Remove(i.max(0) as usize));
    bind1!(on_seek, |s| Command::Seek(s as f64));
    bind1!(on_change_volume, |v| Command::Volume(v as i64));
    bind1!(on_set_shuffle, |on| Command::Shuffle(on));
    bind1!(on_set_repeat, |m| Command::Repeat(m));
    bind1!(on_discord, |on| Command::Discord(on));
    bind1!(on_login, |s| Command::Login(s.into()));
    bind1!(on_google_login, |add| Command::GoogleLogin(add));
    bind1!(on_theme_changed, |dark| Command::Theme(dark));
    bind2!(on_open_shelf, |s, i| Command::OpenShelf(
        s.max(0) as usize,
        i.max(0) as usize
    ));
    bind2!(on_like_shelf, |s, i| Command::LikeShelf(
        s.max(0) as usize,
        i.max(0) as usize
    ));
    bind2!(on_add_shelf, |s, i| Command::AddShelf(
        s.max(0) as usize,
        i.max(0) as usize
    ));
    bind2!(on_setting, |k, on| Command::Setting(k.into(), on));
    ui.on_browser(open_browser);
    ui.on_open_url(|u| open_url(u.as_str()));

    // Icona nell'area di notifica (Mostra / Riproduci / Successivo / Precedente / Riavvia / Esci).
    let restart_requested = Rc::new(std::cell::Cell::new(false));
    let tray = AppTray::new().ok();
    if let Some(tray) = &tray {
        let w = ui.as_weak();
        tray.on_show_app(move || {
            if let Some(u) = w.upgrade() {
                let _ = u.show();
                u.window().set_minimized(false);
            }
        });
        let tx = cmd.clone();
        tray.on_toggle(move || {
            let _ = tx.send(Command::Toggle);
        });
        let tx = cmd.clone();
        tray.on_next(move || {
            let _ = tx.send(Command::Skip(1));
        });
        let tx = cmd.clone();
        tray.on_previous(move || {
            let _ = tx.send(Command::Skip(-1));
        });
        let restart = restart_requested.clone();
        tray.on_restart(move || {
            restart.set(true);
            let _ = slint::quit_event_loop();
        });
        tray.on_quit(|| {
            let _ = slint::quit_event_loop();
        });
    }
    let has_tray = tray.is_some();
    {
        // ✕: nell'area di notifica (impostazione, attiva come in Limusic), altrimenti esce.
        let w = ui.as_weak();
        ui.window().on_close_requested(move || {
            let to_tray = has_tray && w.upgrade().is_some_and(|u| u.get_close_to_tray());
            if !to_tray {
                let _ = slint::quit_event_loop();
            } else {
                trim_memory();
            }
            CloseRequestResponse::HideWindow
        });
    }

    let weak = ui.as_weak();
    let tray_weak = tray.as_ref().map(|t| t.as_weak());
    let bridge = std::thread::spawn(move || {
        for update in receiver {
            let tray_weak = tray_weak.clone();
            let result = weak.upgrade_in_event_loop(move |ui| {
                let tray = tray_weak.as_ref().and_then(|t| t.upgrade());
                match update {
                    Update::Page(p) => {
                        if !p.append {
                            if p.view >= 0 {
                                ui.set_page(p.view);
                            }
                            ui.set_can_go_back(p.can_back);
                            ui.set_heading(p.title.into());
                            ui.set_subheading(p.subtitle.into());
                            ui.set_filter(if p.filter.is_empty() {
                                "all".into()
                            } else {
                                p.filter.clone().into()
                            });
                            if p.view == pages::VIEW_SEARCH
                                && !p.query.is_empty()
                                && ui.get_query() != p.query.as_str()
                            {
                                ui.set_query(p.query.clone().into());
                            }
                            match p.hero {
                                Some(h) => {
                                    ui.set_hero_visible(true);
                                    // Prima i valori derivati, poi si cedono le stringhe a Slint.
                                    let tint = colour(art::placeholder_tint(&h.title));
                                    let kind = if h.round {
                                        "artist"
                                    } else if h.label.starts_with("PLAYLIST") {
                                        "playlist"
                                    } else {
                                        "album"
                                    };
                                    ui.set_hero_label(h.label.into());
                                    ui.set_hero_title(h.title.into());
                                    ui.set_hero_subtitle(h.subtitle.into());
                                    ui.set_hero_detail(h.detail.into());
                                    ui.set_hero_round(h.round);
                                    ui.set_hero_tint(tint);
                                    ui.set_hero_kind(kind.into());
                                    let cover = load(&h.cover);
                                    ui.set_hero_has_cover(cover.is_some());
                                    ui.set_hero_cover(cover.unwrap_or_default());
                                }
                                None => ui.set_hero_visible(false),
                            }
                            // Ricerca "Tutti": risultato principale accanto all'elenco dei brani.
                            let mut list: Vec<(usize, Shelf)> =
                                p.shelves.into_iter().enumerate().collect();
                            let lead = p.view == pages::VIEW_SEARCH
                                && list.len() >= 2
                                && list[0].1.title == "In evidenza"
                                && list[1].1.songs
                                && !list[0].1.items.is_empty();
                            ui.set_has_lead(lead);
                            if lead {
                                let mut rest = list.split_off(2);
                                let (si, songs) = list.pop().unwrap();
                                let (ti, top) = list.pop().unwrap();
                                if let Some(first) = top.items.into_iter().next() {
                                    ui.set_lead_item(row(first, true));
                                }
                                ui.set_lead_shelf(ti as i32);
                                ui.set_lead_songs(shelf(si, songs));
                                list = std::mem::take(&mut rest);
                            }
                            ui.set_shelves(
                                Rc::new(VecModel::from(
                                    list.into_iter()
                                        .map(|(i, s)| shelf(i, s))
                                        .collect::<Vec<_>>(),
                                ))
                                .into(),
                            );
                        }
                        ui.set_rows(rows(p.rows, false));
                        ui.set_has_more(p.continuation.is_some());
                    }
                    Update::Library(list) => ui.set_library(rows(list, false)),
                    Update::Suggestions(queries, items) => {
                        let q: Vec<SharedString> = queries.into_iter().map(Into::into).collect();
                        ui.set_suggest_queries(Rc::new(VecModel::from(q)).into());
                        ui.set_suggest_items(rows(items, false));
                    }
                    Update::Queue(q, i) => {
                        ui.set_queue(rows(q, false));
                        ui.set_queue_index(i);
                    }
                    Update::QueueSource(s) => ui.set_queue_source(s.into()),
                    Update::OpenPlayer => ui.set_np_visible(true),
                    Update::Shuffle(on) => ui.set_shuffle(on),
                    Update::Comments(c) => {
                        let n = c.rows.len() as i32;
                        let items: Vec<CommentItem> = c
                            .rows
                            .into_iter()
                            .map(|r| {
                                let avatar = load(&r.avatar);
                                CommentItem {
                                    id: r.id.into(),
                                    author: r.author.into(),
                                    initial: r.initial.into(),
                                    has_avatar: avatar.is_some(),
                                    avatar: avatar.unwrap_or_default(),
                                    text: r.text.into(),
                                    published: r.published.into(),
                                    likes: r.likes.into(),
                                    liked: r.liked,
                                    can_like: r.can_like,
                                    can_reply: r.can_reply,
                                    replies: r.replies.into(),
                                    reply: r.reply,
                                    pinned: r.pinned,
                                    hearted: r.hearted,
                                    creator: r.creator,
                                    own: r.own,
                                }
                            })
                            .collect();
                        ui.set_comments(ModelRc::new(VecModel::from(items)));
                        ui.set_comments_state(c.view);
                        ui.set_comments_count(c.count.into());
                        ui.set_comments_newest(c.newest);
                        ui.set_comments_has_more(c.has_more);
                        ui.set_comments_can_write(c.can_write);
                        ui.set_comments_sending(c.sending);
                        // La risposta in corso punta a una riga: con un elenco nuovo non vale più.
                        if c.reset || ui.get_comments_reply_to() >= n {
                            ui.set_comments_reply_to(-1);
                        }
                    }
                    Update::Status(s, b) => {
                        ui.set_status(s.into());
                        ui.set_busy(b);
                    }
                    Update::Account(name, on) => {
                        let initial: String = name
                            .chars()
                            .next()
                            .map(|c| c.to_uppercase().collect())
                            .unwrap_or_default();
                        ui.set_account_initial(initial.into());
                        ui.set_account(name.into());
                        ui.set_signed_in(on);
                    }
                    Update::Avatar(path) => match load(&path) {
                        Some(img) => {
                            ui.set_account_avatar(img);
                            ui.set_has_avatar(true);
                        }
                        None => ui.set_has_avatar(false),
                    },
                    Update::LoginPending(on) => ui.set_login_pending(on),
                    Update::SessionSaved(on) => ui.set_session_saved(on),
                    Update::Track(e) => {
                        ui.set_track_liked(e.liked);
                        let title: SharedString = if e.title.is_empty() {
                            "Nessun brano".into()
                        } else {
                            e.title.clone().into()
                        };
                        ui.set_track_title(title.clone());
                        ui.set_track_artist(e.subtitle.clone().into());
                        ui.set_track_album(
                            e.song
                                .as_ref()
                                .and_then(|s| s.album.clone())
                                .unwrap_or_default()
                                .into(),
                        );
                        ui.set_track_id(e.id.clone().into());
                        if let Some(t) = &tray {
                            t.set_track(if e.title.is_empty() {
                                "".into()
                            } else {
                                format!("{} – {}", e.title, e.subtitle).into()
                            });
                        }
                        // Copertina già in memoria (righe, scaffali, coda) finché non arriva quella grande.
                        let found = find_track(&ui, &e.id, e.thumb.is_some());
                        // Senza copertina: copertina generata col colore del titolo.
                        ui.set_np_tint(colour(art::placeholder_tint(&e.title)));
                        match found.filter(|r| r.has_cover) {
                            Some(r) => {
                                ui.set_track_cover(r.thumb);
                                ui.set_np_cover(r.cover);
                                ui.set_track_has_cover(true);
                                ui.set_np_has_cover(true);
                            }
                            None => {
                                ui.set_track_has_cover(false);
                                ui.set_np_has_cover(false);
                            }
                        }
                    }
                    Update::Art(id, path, tint) => {
                        if ui.get_track_id() == id.as_str() {
                            if let Some(img) = load(&path) {
                                ui.set_track_cover(img.clone());
                                ui.set_np_cover(img);
                                ui.set_track_has_cover(true);
                                ui.set_np_has_cover(true);
                                ui.set_np_tint(colour(tint));
                            }
                        }
                    }
                    Update::Position(s) => {
                        ui.set_position(s as f32);
                        ui.set_elapsed(time(s));
                    }
                    Update::Duration(s) => {
                        ui.set_duration(s.max(1.0) as f32);
                        ui.set_length(time(s));
                    }
                    Update::Playing(on) => {
                        ui.set_playing(on);
                        if let Some(t) = &tray {
                            t.set_playing(on);
                        }
                    }
                    Update::Favorites(ids) => {
                        let favorites: HashSet<&str> = ids.iter().map(String::as_str).collect();
                        for_each_list(&ui, &|r: &MusicRow| Some(favorites.contains(r.id.as_str())));
                        let i = ui.get_queue_index();
                        if i >= 0 {
                            if let Some(r) = ui.get_queue().row_data(i as usize) {
                                ui.set_track_liked(r.liked);
                            }
                        }
                    }
                    Update::Rating(id, on) => {
                        for_each_list(&ui, &|r: &MusicRow| (r.id == id.as_str()).then_some(on));
                        if ui.get_track_id() == id.as_str() {
                            ui.set_track_liked(on);
                        }
                    }
                }
            });
            if result.is_err() {
                break;
            }
        }
    });
    if args.iter().any(|a| a == "--account-preview") {
        ui.set_account_visible(true);
    }
    if let Some(path) = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .cloned()
    {
        let weak = ui.as_weak();
        slint::Timer::single_shot(std::time::Duration::from_secs(2), move || {
            if let Some(ui) = weak.upgrade() {
                match ui.window().take_snapshot() {
                    Ok(pixels) => {
                        if let Err(e) = image::save_buffer(
                            &path,
                            pixels.as_bytes(),
                            pixels.width(),
                            pixels.height(),
                            image::ColorType::Rgba8,
                        ) {
                            eprintln!("Screenshot: {e}");
                        }
                    }
                    Err(e) => eprintln!("Screenshot: {e}"),
                }
            }
            let _ = slint::quit_event_loop();
        });
    }
    // La finestra nascosta nell'area di notifica non chiude il ciclo: lo chiude Esci (o la ✕
    // con l'impostazione disattivata).
    ui.show()?;
    #[cfg(windows)]
    desktop::fit_work_area(ui.window());
    #[cfg(windows)]
    let _instance_timer = {
        let timer = slint::Timer::default();
        let instance = instance.clone();
        let weak = ui.as_weak();
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(200),
            move || {
                if instance.wake_requested() {
                    if let Some(ui) = weak.upgrade() {
                        let _ = ui.show();
                        ui.window().set_minimized(false);
                        single_instance::Instance::activate_window();
                    }
                }
            },
        );
        timer
    };
    let result = slint::run_event_loop();
    let _ = ui.hide();
    let _ = cmd.send(Command::Shutdown);
    let _ = worker.join();
    drop(tray);
    let _ = bridge.join();
    #[cfg(windows)]
    {
        drop(_instance_timer);
        drop(instance);
    }
    result?;
    if restart_requested.get() {
        std::process::Command::new(std::env::current_exe()?).spawn()?;
    }
    Ok(())
}
