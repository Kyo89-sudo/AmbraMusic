//! Commenti del brano in ascolto: lo stato della scheda "Commenti" del pannello di destra.
//!
//! Qui ci sono solo dati e regole, senza rete: le richieste partono da `backend.rs` con il client
//! InnerTube, e le risposte arrivano a [`Comments`] che decide cosa mostrare. I token di YouTube
//! (mi piace, risposta, invio) restano in queste strutture e non vanno mai alla GUI.

use innertube::models::comments::available_actions;
use innertube::{
    Comment, CommentAction, CommentReplies, CommentSortKey, CommentThread, CommentsPage,
    CommentsState, VoteState, WriteCommand,
};
use std::path::PathBuf;

/// Cosa mostra la scheda.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Loading,
    List,
    Empty,
    Disabled,
    Failed,
}

impl View {
    /// Il numero che usa la GUI (`comments-state`).
    pub fn code(self) -> i32 {
        match self {
            View::Loading => 0,
            View::List => 1,
            View::Empty => 2,
            View::Disabled => 3,
            View::Failed => 4,
        }
    }
}

/// Una riga della scheda: un commento oppure una risposta (rientrata sotto il suo commento).
#[derive(Clone, Debug)]
pub struct Item {
    pub comment: Comment,
    pub reply: bool,
    /// Solo sui commenti: token per caricare (altre) risposte. `None` = niente da caricare.
    pub replies_token: Option<String>,
    /// Quante risposte ha il commento, come la scrive YouTube ("24").
    pub reply_count: Option<String>,
    /// Risposte già caricate sotto questo commento.
    pub shown_replies: usize,
}

/// Quello che arriva alla GUI per ogni riga: testo pronto e foto già scaricata.
#[derive(Clone, Debug, Default)]
pub struct Row {
    pub id: String,
    pub author: String,
    pub initial: String,
    pub avatar: Option<PathBuf>,
    pub text: String,
    pub published: String,
    pub likes: String,
    pub liked: bool,
    pub can_like: bool,
    pub can_reply: bool,
    /// Etichetta del pulsante delle risposte, vuota se non ce ne sono da caricare.
    pub replies: String,
    pub reply: bool,
    pub pinned: bool,
    pub hearted: bool,
    pub creator: bool,
    pub own: bool,
}

#[derive(Default)]
pub struct Comments {
    /// Brano a cui appartengono i commenti mostrati.
    pub video_id: Option<String>,
    /// Cresce a ogni nuovo caricamento: le risposte di richieste vecchie si scartano.
    pub gen: u64,
    pub items: Vec<Item>,
    pub continuation: Option<String>,
    top: Option<String>,
    newest_token: Option<String>,
    pub newest: bool,
    pub count: String,
    /// Comando per pubblicare un commento nuovo (solo con l'account).
    pub create: Option<WriteCommand>,
    /// La pagina è stata letta con l'account: i suoi token si rimandano allo stesso modo.
    pub as_account: bool,
    /// Account con cui sono stati letti i token (impronta di `comments_identity`).
    pub identity: Option<String>,
    pub view: Option<View>,
    /// Un invio in corso (commento o risposta).
    pub sending: bool,
    /// Commenti con un mi piace in corso, per non mandarne due insieme.
    pub voting: Vec<String>,
}

impl Comments {
    /// Svuota tutto e prepara il caricamento dei commenti di `video_id`.
    pub fn start(&mut self, video_id: &str) -> u64 {
        let gen = self.gen + 1;
        *self = Comments {
            video_id: Some(video_id.to_owned()),
            gen,
            view: Some(View::Loading),
            ..Default::default()
        };
        gen
    }

    /// Dimentica i commenti (account cambiato, brano fermo): si ricaricano alla prossima apertura.
    pub fn clear(&mut self) {
        let gen = self.gen + 1;
        *self = Comments {
            gen,
            ..Default::default()
        };
    }

    pub fn is_for(&self, video_id: &str) -> bool {
        self.video_id.as_deref() == Some(video_id)
    }

    /// Prima pagina o cambio di ordine: sostituisce l'elenco. Altrimenti accoda.
    pub fn apply_page(&mut self, page: CommentsPage, replace: bool) {
        if let Some(h) = &page.header {
            if let Some(c) = &h.count_text {
                self.count = c.clone();
            }
            for s in &h.sorts {
                match s.key {
                    CommentSortKey::Top => self.top = Some(s.token.clone()),
                    CommentSortKey::Newest => self.newest_token = Some(s.token.clone()),
                }
                if s.selected {
                    self.newest = s.key == CommentSortKey::Newest;
                }
            }
            if h.create.is_some() {
                self.create = h.create.clone();
            }
        }
        if replace {
            self.items.clear();
        }
        for t in page.threads {
            self.push_thread(t);
        }
        self.continuation = page.continuation;
        self.view = Some(match page.state {
            CommentsState::Disabled => View::Disabled,
            _ if self.items.is_empty() => View::Empty,
            _ => View::List,
        });
    }

    fn thread_item(t: &CommentThread) -> Item {
        Item {
            comment: t.comment.clone(),
            reply: false,
            replies_token: t.replies_token.clone(),
            reply_count: t.comment.reply_count.clone(),
            shown_replies: t.replies.len(),
        }
    }

    fn reply_item(c: Comment) -> Item {
        Item {
            comment: c,
            reply: true,
            replies_token: None,
            reply_count: None,
            shown_replies: 0,
        }
    }

    fn push_thread(&mut self, t: CommentThread) {
        self.items.push(Self::thread_item(&t));
        for r in t.replies {
            self.items.push(Self::reply_item(r));
        }
    }

    /// Token per l'altro ordine (popolari ↔ recenti), se la pagina l'ha dato.
    pub fn sort_token(&self, newest: bool) -> Option<String> {
        if newest {
            self.newest_token.clone()
        } else {
            self.top.clone()
        }
    }

    /// Indice del commento a cui appartiene la riga `i` (se stessa, o il commento sopra la risposta).
    pub fn thread_of(&self, i: usize) -> Option<usize> {
        (0..=i.min(self.items.len().checked_sub(1)?))
            .rev()
            .find(|&j| !self.items[j].reply)
    }

    /// Fine delle risposte del commento `t` (dove inserirne altre).
    fn replies_end(&self, t: usize) -> usize {
        let mut end = t + 1;
        while end < self.items.len() && self.items[end].reply {
            end += 1;
        }
        end
    }

    /// Risposte arrivate per il commento con questo id: si inseriscono sotto le sue, senza doppioni.
    pub fn apply_replies(&mut self, comment_id: &str, page: CommentReplies) -> bool {
        let Some(t) = self
            .items
            .iter()
            .position(|it| !it.reply && it.comment.id == comment_id)
        else {
            return false;
        };
        let end = self.replies_end(t);
        let known: Vec<String> = self.items[t + 1..end]
            .iter()
            .map(|it| it.comment.id.clone())
            .collect();
        let fresh: Vec<Item> = page
            .replies
            .into_iter()
            .filter(|c| !known.contains(&c.id))
            .map(Self::reply_item)
            .collect();
        let added = fresh.len();
        self.items.splice(end..end, fresh);
        let thread = &mut self.items[t];
        thread.shown_replies += added;
        thread.replies_token = page.continuation;
        true
    }

    /// Il proprio commento appena pubblicato, in cima (sotto quello fissato).
    pub fn add_thread(&mut self, t: CommentThread) {
        let at = self
            .items
            .iter()
            .position(|it| !it.reply && !it.comment.pinned)
            .unwrap_or(self.items.len());
        let mut rows = vec![Self::thread_item(&t)];
        rows.extend(t.replies.into_iter().map(Self::reply_item));
        self.items.splice(at..at, rows);
        self.view = Some(View::List);
    }

    /// La propria risposta, in fondo alle risposte del commento.
    pub fn add_reply(&mut self, comment_id: &str, c: Comment) {
        if let Some(t) = self
            .items
            .iter()
            .position(|it| !it.reply && it.comment.id == comment_id)
        {
            let end = self.replies_end(t);
            self.items.insert(end, Self::reply_item(c));
            self.items[t].shown_replies += 1;
        }
    }

    /// Il mi piace da mandare per la riga `i`: metterlo o toglierlo, se YouTube lo permette.
    pub fn vote_for(&self, i: usize) -> Option<(String, CommentAction, String)> {
        let c = &self.items.get(i)?.comment;
        let action = if c.vote == Some(VoteState::Liked) {
            CommentAction::Unlike
        } else {
            CommentAction::Like
        };
        if !c.actions.contains(&action) || self.voting.contains(&c.id) {
            return None;
        }
        let token = c.tokens.get(action)?.to_owned();
        Some((c.id.clone(), action, token))
    }

    /// Mi piace accettato: stato, conteggio (YouTube manda già entrambe le cifre) e azioni.
    pub fn voted(&mut self, comment_id: &str, action: CommentAction, ok: bool) {
        self.voting.retain(|id| id != comment_id);
        if !ok {
            return;
        }
        for it in self
            .items
            .iter_mut()
            .filter(|it| it.comment.id == comment_id)
        {
            let c = &mut it.comment;
            let vote = action.resulting_vote();
            c.vote = Some(vote);
            c.like_count = if vote == VoteState::Liked {
                c.like_count_liked.clone()
            } else {
                c.like_count_notliked.clone()
            };
            c.actions = available_actions(c.vote, &c.tokens);
        }
    }

    /// Righe per la GUI. `avatar` trova la foto già scaricata di un URL.
    pub fn rows(&self, avatar: impl Fn(&str) -> Option<PathBuf>) -> Vec<Row> {
        self.items
            .iter()
            .map(|it| {
                let c = &it.comment;
                Row {
                    id: c.id.clone(),
                    author: c.author.name.clone(),
                    initial: initial(&c.author.name),
                    avatar: c.author.avatar.as_deref().and_then(&avatar),
                    text: c.text.clone(),
                    published: c.published.clone().unwrap_or_default(),
                    likes: c.like_count.clone().unwrap_or_default(),
                    liked: c.vote == Some(VoteState::Liked),
                    can_like: c
                        .actions
                        .iter()
                        .any(|a| matches!(a, CommentAction::Like | CommentAction::Unlike)),
                    can_reply: !it.reply && c.commands.reply.is_some(),
                    replies: if it.reply {
                        String::new()
                    } else {
                        replies_label(it)
                    },
                    reply: it.reply,
                    pinned: c.pinned,
                    hearted: c.hearted,
                    creator: c.author.is_creator || c.author.is_artist,
                    own: c.own,
                }
            })
            .collect()
    }

    /// Foto degli autori da scaricare.
    pub fn avatar_urls(&self) -> Vec<String> {
        self.items
            .iter()
            .filter_map(|it| it.comment.author.avatar.clone())
            .collect()
    }
}

/// "Mostra 24 risposte", "Mostra 1 risposta", "Altre risposte"; vuoto se non c'è altro.
fn replies_label(it: &Item) -> String {
    if it.replies_token.is_none() {
        return String::new();
    }
    if it.shown_replies > 0 {
        return "Altre risposte".into();
    }
    match it.reply_count.as_deref() {
        Some("1") => "Mostra 1 risposta".into(),
        Some(n) if !n.is_empty() => format!("Mostra {n} risposte"),
        _ => "Mostra le risposte".into(),
    }
}

/// Iniziale del nome per la foto mancante, senza la "@" dei nomi dei canali.
pub fn initial(name: &str) -> String {
    name.trim_start_matches('@')
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_default()
}

/// Testo da inviare: senza spazi ai lati, non vuoto e nei 10.000 caratteri di YouTube.
pub fn clean_text(text: &str) -> Option<String> {
    let t = text.trim();
    (!t.is_empty() && t.chars().count() <= 10_000).then(|| t.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials() {
        assert_eq!(initial("@marta.vinili"), "M");
        assert_eq!(initial("élodie"), "É");
        assert_eq!(initial("  "), "");
        assert_eq!(initial("_x"), "X");
    }

    #[test]
    fn text_is_cleaned() {
        assert_eq!(clean_text("  ciao \n"), Some("ciao".into()));
        assert_eq!(clean_text("   "), None);
        assert_eq!(clean_text(&"a".repeat(10_001)), None);
    }

    #[test]
    fn view_codes_match_the_gui() {
        let codes: Vec<i32> = [
            View::Loading,
            View::List,
            View::Empty,
            View::Disabled,
            View::Failed,
        ]
        .iter()
        .map(|v| v.code())
        .collect();
        assert_eq!(codes, vec![0, 1, 2, 3, 4]);
    }

    fn comment(id: &str, pinned: bool) -> Comment {
        Comment {
            id: id.into(),
            text: format!("testo {id}"),
            author: innertube::models::comments::CommentAuthor {
                name: format!("@autore{id}"),
                channel_id: None,
                avatar: Some(format!("https://yt3.ggpht.com/{id}=s88")),
                verified: false,
                is_creator: false,
                is_artist: false,
            },
            published: Some("1 giorno fa".into()),
            like_count: Some("5".into()),
            like_count_notliked: Some("5".into()),
            like_count_liked: Some("6".into()),
            vote: None,
            actions: vec![],
            tokens: Default::default(),
            own: false,
            edit_text: None,
            writes: vec![],
            reply_placeholder: None,
            commands: Default::default(),
            reply_count: Some("2".into()),
            hearted: false,
            pinned,
        }
    }
    fn thread(id: &str, pinned: bool) -> CommentThread {
        CommentThread {
            comment: comment(id, pinned),
            replies_token: Some(format!("tok-{id}")),
            replies: vec![],
        }
    }
    fn page(ids: &[&str], more: bool) -> CommentsPage {
        CommentsPage {
            header: None,
            threads: ids.iter().map(|i| thread(i, false)).collect(),
            continuation: more.then(|| "next".into()),
            state: CommentsState::Ok,
            read_as_account: false,
        }
    }

    #[test]
    fn pages_replace_or_append() {
        let mut c = Comments::default();
        c.start("v");
        c.apply_page(page(&["a", "b"], true), true);
        assert_eq!(c.view, Some(View::List));
        assert_eq!(c.continuation.as_deref(), Some("next"));
        c.apply_page(page(&["c"], false), false);
        assert_eq!(c.items.len(), 3);
        assert!(c.continuation.is_none());
        c.apply_page(page(&[], false), true);
        assert_eq!(c.view, Some(View::Empty));
        let mut off = page(&[], false);
        off.state = CommentsState::Disabled;
        c.apply_page(off, true);
        assert_eq!(c.view, Some(View::Disabled));
    }

    #[test]
    fn replies_go_under_their_comment() {
        let mut c = Comments::default();
        c.apply_page(page(&["a", "b"], false), true);
        assert_eq!(c.rows(|_| None)[0].replies, "Mostra 2 risposte");
        let r = CommentReplies {
            replies: vec![comment("a1", false), comment("a2", false)],
            continuation: Some("more".into()),
        };
        assert!(c.apply_replies("a", r.clone()));
        let ids: Vec<&str> = c.items.iter().map(|i| i.comment.id.as_str()).collect();
        assert_eq!(ids, ["a", "a1", "a2", "b"]);
        assert!(c.items[1].reply && !c.items[3].reply);
        // Le stesse risposte di nuovo non si duplicano.
        c.apply_replies("a", r);
        assert_eq!(c.items.len(), 4);
        assert_eq!(c.rows(|_| None)[0].replies, "Altre risposte");
        assert_eq!(c.thread_of(2), Some(0));
        assert_eq!(c.thread_of(3), Some(3));
        assert_eq!(c.thread_of(99), Some(3));
        c.add_reply("a", comment("mia", false));
        assert_eq!(c.items[3].comment.id, "mia");
        assert!(!c.apply_replies("zzz", CommentReplies::default()));
    }

    #[test]
    fn own_comment_goes_under_the_pinned_one() {
        let mut c = Comments::default();
        let mut p = page(&["b"], false);
        p.threads.insert(0, thread("fissato", true));
        c.apply_page(p, true);
        c.add_thread(thread("nuovo", false));
        let ids: Vec<&str> = c.items.iter().map(|i| i.comment.id.as_str()).collect();
        assert_eq!(ids, ["fissato", "nuovo", "b"]);
    }

    #[test]
    fn a_like_swaps_the_count() {
        let mut c = Comments::default();
        c.apply_page(page(&["a"], false), true);
        // Letto senza account: nessun token, nessun mi piace possibile.
        assert!(c.vote_for(0).is_none());
        assert!(!c.rows(|_| None)[0].can_like);
        c.voting.push("a".into());
        c.voted("a", CommentAction::Like, true);
        assert!(c.voting.is_empty());
        let row = &c.rows(|_| None)[0];
        assert!(row.liked);
        assert_eq!(row.likes, "6");
        c.voted("a", CommentAction::Unlike, true);
        assert_eq!(c.rows(|_| None)[0].likes, "5");
        c.voted("a", CommentAction::Like, false);
        assert!(!c.rows(|_| None)[0].liked);
    }

    #[test]
    fn rows_carry_avatars_and_initials() {
        let mut c = Comments::default();
        c.apply_page(page(&["a"], false), true);
        let rows = c.rows(|u| u.contains("/a=").then(|| PathBuf::from("a.png")));
        assert_eq!(rows[0].avatar, Some(PathBuf::from("a.png")));
        assert_eq!(rows[0].initial, "A");
        assert_eq!(c.avatar_urls().len(), 1);
    }

    #[test]
    fn start_and_clear_bump_the_generation() {
        let mut c = Comments::default();
        let g = c.start("abc");
        assert!(c.is_for("abc"));
        assert_eq!(c.view, Some(View::Loading));
        c.clear();
        assert!(c.gen > g);
        assert!(!c.is_for("abc"));
        assert_eq!(c.view, None);
    }
}
