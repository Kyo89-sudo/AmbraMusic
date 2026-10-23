//! Copertina grande della vista "In riproduzione": URL in alta risoluzione e colore dominante.
//! Funzioni pure, senza rete né decodifica (quelle stanno in `backend::fetch_art`).

/// Le miniature da provare, dalla più grande alla meno grande, sempre chiudendo con l'originale.
///
/// Come in Limusic (`ui/src/lib/thumb.ts`): InnerTube restituisce miniature video da 426x240 al
/// massimo, quindi per i.ytimg si chiede `maxresdefault` (non sempre esiste, da qui il ripiego su
/// `hqdefault`); per le copertine di lh3/yt3 si riscrive la dimensione `=wN-hN` o `=sN`.
pub fn candidates(url: &str, px: u32) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(id) = ytimg_video_id(url) {
        out.push(format!("https://i.ytimg.com/vi/{id}/maxresdefault.jpg"));
        out.push(format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg"));
    } else if let Some(u) = resize_param(url, px) {
        out.push(u);
    }
    if !out.iter().any(|u| u == url) {
        out.push(url.to_owned());
    }
    out
}

fn ytimg_video_id(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://i.ytimg.com/vi/")?;
    let id = rest.split('/').next()?;
    (id.len() == 11
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
    .then_some(id)
}

/// Sostituisce il primo `=wN-hN` (o, in mancanza, `=sN`) con la dimensione richiesta.
fn resize_param(url: &str, px: u32) -> Option<String> {
    let digits = |s: &str| s.bytes().take_while(u8::is_ascii_digit).count();
    let mut from = 0;
    while let Some(i) = url[from..].find("=w").map(|i| i + from) {
        let a = i + 2;
        let wn = digits(&url[a..]);
        if wn > 0 && url[a + wn..].starts_with("-h") {
            let b = a + wn + 2;
            let hn = digits(&url[b..]);
            if hn > 0 {
                return Some(format!("{}=w{px}-h{px}{}", &url[..i], &url[b + hn..]));
            }
        }
        from = i + 2;
    }
    let mut from = 0;
    while let Some(i) = url[from..].find("=s").map(|i| i + from) {
        let a = i + 2;
        let n = digits(&url[a..]);
        if n > 0 {
            return Some(format!("{}=s{px}{}", &url[..i], &url[a + n..]));
        }
        from = i + 2;
    }
    None
}

/// Come `candidates`, ma per schede e righe: niente `maxresdefault` (1280 px) per una miniatura,
/// basta `hqdefault` (480x360); le copertine lh3/yt3 si chiedono a `px`. YouTube Music manda nei
/// risultati miniature da 60 px: usate così sarebbero sfocate.
pub fn list_candidates(url: &str, px: u32) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(id) = ytimg_video_id(url) {
        out.push(format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg"));
    } else if let Some(u) = resize_param(url, px) {
        out.push(u);
    }
    if !out.iter().any(|u| u == url) {
        out.push(url.to_owned());
    }
    out
}

/// i.ytimg risponde a una dimensione mancante con un'immagine grigia 120x90: va trattata come assente.
pub fn is_missing_placeholder(width: u32, height: u32) -> bool {
    width == 120 && height == 90
}

/// Colore dominante per lo sfondo della vista: media pesata per saturazione, poi portata a una
/// luminosità media così funziona sia sul tema scuro sia su quello chiaro.
pub fn tint(pixels: &[[u8; 3]]) -> [u8; 3] {
    let (mut r, mut g, mut b, mut w) = (0f64, 0f64, 0f64, 0f64);
    for p in pixels {
        let max = *p.iter().max().unwrap() as f64;
        let min = *p.iter().min().unwrap() as f64;
        // Saturazione e un po' di luminosità: il grigio e il nero contano poco.
        let weight = (max - min) / 255.0 * (0.25 + max / 255.0) + 0.02;
        r += p[0] as f64 * weight;
        g += p[1] as f64 * weight;
        b += p[2] as f64 * weight;
        w += weight;
    }
    if w <= 0.0 {
        return [255, 138, 18]; // ambra
    }
    let (r, g, b) = (r / w, g / w, b / w);
    let max = r.max(g).max(b);
    if max < 1.0 {
        return [255, 138, 18];
    }
    let k = 210.0 / max;
    [
        (r * k).round().clamp(0.0, 255.0) as u8,
        (g * k).round().clamp(0.0, 255.0) as u8,
        (b * k).round().clamp(0.0, 255.0) as u8,
    ]
}

/// Colore della copertina generata: sempre lo stesso per lo stesso titolo, toni smorzati che
/// stanno bene su entrambi i temi (tinta dal titolo, saturazione e luminosità fisse).
pub fn placeholder_tint(key: &str) -> [u8; 3] {
    // FNV-1a: stabile tra un avvio e l'altro (DefaultHasher non lo garantisce).
    let mut h: u32 = 0x811c_9dc5;
    for b in key.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    let hue = (h % 360) as f64;
    let (s, v) = (0.48, 0.66);
    let c = v * s;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (hue / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let to = |f: f64| ((f + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    [to(r), to(g), to(b)]
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder_colour_is_stable_and_muted() {
        let a = super::placeholder_tint("Cozy");
        assert_eq!(a, super::placeholder_tint("Cozy"));
        assert_ne!(a, super::placeholder_tint("Gym"));
        for key in ["", "a", "Brani che ti piacciono", "日本語"] {
            let c = super::placeholder_tint(key);
            let max = *c.iter().max().unwrap();
            let min = *c.iter().min().unwrap();
            assert!(max <= 170 && max >= 150 && min >= 80, "{key}: {c:?}");
        }
    }

    use super::*;

    #[test]
    fn video_thumbs_ask_for_maxres_then_hq_then_the_original() {
        let u = "https://i.ytimg.com/vi/dQw4w9WgXcQ/hq720.jpg?sqp=abc";
        assert_eq!(
            candidates(u, 720),
            vec![
                "https://i.ytimg.com/vi/dQw4w9WgXcQ/maxresdefault.jpg".to_string(),
                "https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg".to_string(),
                u.to_string(),
            ]
        );
    }

    #[test]
    fn album_art_size_is_rewritten() {
        let u = "https://lh3.googleusercontent.com/abc=w120-h120-l90-rj";
        assert_eq!(
            candidates(u, 720)[0],
            "https://lh3.googleusercontent.com/abc=w720-h720-l90-rj"
        );
        let s = "https://yt3.ggpht.com/xyz=s88-c-k";
        assert_eq!(candidates(s, 720)[0], "https://yt3.ggpht.com/xyz=s720-c-k");
        assert_eq!(candidates(s, 720).len(), 2);
    }

    #[test]
    fn list_thumbnails_ask_for_a_usable_size() {
        assert_eq!(
            list_candidates("https://lh3.googleusercontent.com/x=w60-h60-l90-rj", 320)[0],
            "https://lh3.googleusercontent.com/x=w320-h320-l90-rj"
        );
        assert_eq!(
            list_candidates("https://i.ytimg.com/vi/dQw4w9WgXcQ/sddefault.jpg", 320),
            vec![
                "https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg".to_string(),
                "https://i.ytimg.com/vi/dQw4w9WgXcQ/sddefault.jpg".to_string(),
            ]
        );
    }

    #[test]
    fn unknown_urls_are_kept_as_they_are() {
        let u = "https://example.com/cover.png";
        assert_eq!(candidates(u, 720), vec![u.to_string()]);
        assert_eq!(resize_param("https://x/=wide", 720), None);
    }

    #[test]
    fn placeholder_and_tint() {
        assert!(is_missing_placeholder(120, 90));
        assert!(!is_missing_placeholder(1280, 720));
        // Un'immagine metà rossa satura e metà grigia tende al rosso.
        let mut px = vec![[220, 20, 20]; 50];
        px.extend(vec![[128, 128, 128]; 50]);
        let t = tint(&px);
        assert!(t[0] > 180 && t[1] < 120 && t[2] < 120, "{t:?}");
        assert_eq!(tint(&[]), [255, 138, 18]);
        assert_eq!(tint(&[[0, 0, 0]]), [255, 138, 18]);
    }
}
