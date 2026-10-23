//! Parti del player che non toccano libmpv: si compilano e si provano da sole.

/// Costanti dell'API C di libmpv (include/mpv/client.h, API 2.x).
pub mod mpv {
    pub const EVENT_NONE: i32 = 0;
    pub const EVENT_SHUTDOWN: i32 = 1;
    pub const EVENT_END_FILE: i32 = 7;
    pub const EVENT_PROPERTY_CHANGE: i32 = 22;

    pub const FORMAT_FLAG: i32 = 3;
    pub const FORMAT_DOUBLE: i32 = 5;

    pub const END_FILE_EOF: i32 = 0;
    pub const END_FILE_ERROR: i32 = 4;

    pub const ERROR_AO_INIT_FAILED: i32 = -14;
}

/// Cosa è successo quando mpv chiude un file.
#[derive(Debug, PartialEq, Eq)]
pub enum Ended {
    /// Finito normalmente: si passa al brano successivo.
    Finished,
    /// L'uscita audio non si apre più (cuffie staccate, dispositivo sparito).
    DeviceLost,
    /// Errore di rete o di formato: il codice di errore di mpv.
    Failed(i32),
    /// Fermato da noi (stop, nuovo brano): nessun evento per l'app.
    Stopped,
}

/// Interpreta `mpv_event_end_file` (motivo + codice di errore).
pub fn end_file(reason: i32, error: i32) -> Ended {
    match reason {
        mpv::END_FILE_EOF => Ended::Finished,
        mpv::END_FILE_ERROR if error == mpv::ERROR_AO_INIT_FAILED => Ended::DeviceLost,
        mpv::END_FILE_ERROR => Ended::Failed(error),
        _ => Ended::Stopped,
    }
}

/// Volume del cursore (0–100) → proprietà `volume` di mpv.
///
/// mpv applica un guadagno di (v/100)³: con una mappa lineare la parte bassa del cursore salta
/// di molti dB a ogni passo e quella alta quasi non cambia. Qui il cursore copre 50 dB in modo
/// uniforme (ogni passo ≈ 0,5 dB) e lo 0 è silenzio vero.
pub fn mpv_volume(percent: i64) -> f64 {
    let p = percent.clamp(0, 100) as f64;
    if p <= 0.0 {
        return 0.0;
    }
    let db = -50.0 * (1.0 - p / 100.0);
    let gain = 10f64.powf(db / 20.0);
    100.0 * gain.cbrt()
}

/// Divide le intestazioni HTTP tra `user-agent` e `http-header-fields` (lista separata da
/// virgole, per cui un valore con una virgola non si può passare e viene scartato).
pub fn split_headers<'a, I>(headers: I) -> (Option<String>, String)
where
    I: IntoIterator<Item = (&'a String, &'a String)>,
{
    let mut agent = None;
    let mut fields: Vec<String> = Vec::new();
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("user-agent") {
            agent = Some(v.clone());
        } else if !v.contains(',') && !k.contains(',') && !k.contains(':') {
            fields.push(format!("{k}: {v}"));
        }
    }
    fields.sort(); // ordine stabile, comodo da confrontare nei log
    (agent, fields.join(","))
}

/// Si inoltra la posizione solo se è cambiata abbastanza (o è tornata indietro, cioè un salto):
/// mpv la aggiorna molte volte al secondo, la barra ne ha bisogno di poche.
pub fn position_due(last: Option<f64>, now: f64) -> bool {
    match last {
        None => true,
        Some(l) => now < l || now - l >= 0.25,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn end_file_reasons() {
        assert_eq!(end_file(0, 0), Ended::Finished);
        assert_eq!(end_file(4, -14), Ended::DeviceLost);
        assert_eq!(end_file(4, -13), Ended::Failed(-13));
        assert_eq!(end_file(2, 0), Ended::Stopped);
        assert_eq!(end_file(3, 0), Ended::Stopped);
        assert_eq!(end_file(5, 0), Ended::Stopped);
    }

    #[test]
    fn volume_curve() {
        assert_eq!(mpv_volume(0), 0.0);
        assert!((mpv_volume(100) - 100.0).abs() < 1e-9);
        assert_eq!(mpv_volume(150), mpv_volume(100));
        assert_eq!(mpv_volume(-5), 0.0);
        // Monotona e con passi in dB uguali: a metà cursore -25 dB.
        let half = mpv_volume(50) / 100.0;
        let db = 20.0 * (half.powi(3)).log10();
        assert!((db + 25.0).abs() < 1e-6, "{db}");
        for p in 1..100 {
            assert!(mpv_volume(p) < mpv_volume(p + 1));
        }
    }

    #[test]
    fn headers_are_split() {
        let mut h = HashMap::new();
        h.insert("User-Agent".to_string(), "Mozilla/5.0 (X)".to_string());
        h.insert(
            "Origin".to_string(),
            "https://music.youtube.com".to_string(),
        );
        h.insert("X-Bad".to_string(), "a,b".to_string());
        h.insert("Accept".to_string(), "*/*".to_string());
        let (ua, fields) = split_headers(&h);
        assert_eq!(ua.as_deref(), Some("Mozilla/5.0 (X)"));
        assert_eq!(fields, "Accept: */*,Origin: https://music.youtube.com");
        let (ua, fields) = split_headers(&HashMap::new());
        assert_eq!(ua, None);
        assert_eq!(fields, "");
    }

    #[test]
    fn position_throttle() {
        assert!(position_due(None, 0.0));
        assert!(!position_due(Some(1.0), 1.1));
        assert!(position_due(Some(1.0), 1.25));
        assert!(
            position_due(Some(30.0), 2.0),
            "un salto indietro passa subito"
        );
    }
}
