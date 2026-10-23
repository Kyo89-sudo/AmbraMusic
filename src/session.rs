//! Sessione YouTube Music salvata su disco e funzioni pure sui cookie.
//!
//! Su Windows il Cookie header è cifrato con DPAPI (`CryptProtectData`, ambito utente corrente):
//! il file `session.dat` è leggibile solo dallo stesso utente Windows sullo stesso PC. Sugli altri
//! sistemi (solo sviluppo) viene scritto in chiaro con permessi 0600.
//! Il file delle impostazioni non contiene mai cookie.

use std::path::{Path, PathBuf};

const MAGIC_DPAPI: &[u8; 4] = b"AMBW";
#[cfg(not(windows))]
const MAGIC_PLAIN: &[u8; 4] = b"AMBP";

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn new(data: &Path) -> Self {
        Self {
            path: data.join("session.dat"),
        }
    }

    /// Il cookie salvato, se esiste, si decifra e contiene ancora SAPISID.
    pub fn load(&self) -> Option<String> {
        let bytes = std::fs::read(&self.path).ok()?;
        let plain = decode(&bytes)?;
        let cookie = String::from_utf8(plain).ok()?;
        innertube::cookie_sapisid(&cookie)
            .is_some()
            .then_some(cookie)
    }

    pub fn save(&self, cookie: &str) -> std::io::Result<()> {
        let bytes = encode(cookie.as_bytes())?;
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, &bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))?;
        }
        std::fs::rename(&tmp, &self.path)
    }

    pub fn clear(&self) {
        let _ = std::fs::remove_file(&self.path);
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(windows)]
fn encode(plain: &[u8]) -> std::io::Result<Vec<u8>> {
    let blob = dpapi::protect(plain).ok_or_else(std::io::Error::last_os_error)?;
    let mut out = MAGIC_DPAPI.to_vec();
    out.extend_from_slice(&blob);
    Ok(out)
}
#[cfg(windows)]
fn decode(bytes: &[u8]) -> Option<Vec<u8>> {
    dpapi::unprotect(bytes.strip_prefix(MAGIC_DPAPI)?)
}

#[cfg(not(windows))]
fn encode(plain: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = MAGIC_PLAIN.to_vec();
    out.extend_from_slice(plain);
    Ok(out)
}
#[cfg(not(windows))]
fn decode(bytes: &[u8]) -> Option<Vec<u8>> {
    // Un file cifrato su Windows e copiato altrove non è leggibile: lo si ignora.
    if bytes.starts_with(MAGIC_DPAPI) {
        return None;
    }
    bytes.strip_prefix(MAGIC_PLAIN).map(<[u8]>::to_vec)
}

/// DPAPI dichiarata a mano: due funzioni di crypt32 e LocalFree, senza dipendenze aggiuntive.
#[cfg(windows)]
mod dpapi {
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    #[repr(C)]
    struct Blob {
        len: u32,
        data: *mut u8,
    }

    #[link(name = "crypt32")]
    extern "system" {
        fn CryptProtectData(
            data_in: *const Blob,
            description: *const u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            data_out: *mut Blob,
        ) -> i32;
        fn CryptUnprotectData(
            data_in: *const Blob,
            description: *mut *mut u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            data_out: *mut Blob,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(mem: *mut c_void) -> *mut c_void;
    }

    const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x1;
    /// Entropia fissa: impedisce ad altre app dello stesso utente di decifrare il file
    /// semplicemente chiamando DPAPI senza sapere a cosa serve.
    const ENTROPY: &[u8] = b"AmbraMusic/session/v1";

    fn run(input: &[u8], protect: bool) -> Option<Vec<u8>> {
        let data_in = Blob {
            len: u32::try_from(input.len()).ok()?,
            data: input.as_ptr() as *mut u8,
        };
        let entropy = Blob {
            len: ENTROPY.len() as u32,
            data: ENTROPY.as_ptr() as *mut u8,
        };
        let mut out = Blob {
            len: 0,
            data: null_mut(),
        };
        // SAFETY: tutti i puntatori restano validi per la durata della chiamata; il buffer di
        // uscita è allocato da DPAPI e liberato con LocalFree subito dopo la copia.
        let ok = unsafe {
            if protect {
                CryptProtectData(
                    &data_in,
                    null(),
                    &entropy,
                    null_mut(),
                    null_mut(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut out,
                )
            } else {
                CryptUnprotectData(
                    &data_in,
                    null_mut(),
                    &entropy,
                    null_mut(),
                    null_mut(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut out,
                )
            }
        };
        if ok == 0 || out.data.is_null() {
            return None;
        }
        let bytes = unsafe { std::slice::from_raw_parts(out.data, out.len as usize).to_vec() };
        unsafe {
            LocalFree(out.data as *mut c_void);
        }
        Some(bytes)
    }

    pub fn protect(plain: &[u8]) -> Option<Vec<u8>> {
        run(plain, true)
    }
    pub fn unprotect(blob: &[u8]) -> Option<Vec<u8>> {
        run(blob, false)
    }
}

/// Toglie un eventuale prefisso `Cookie:` e gli spazi da un valore incollato.
pub fn normalize(cookie: &str) -> String {
    let c = cookie.trim();
    let c = c
        .strip_prefix("Cookie:")
        .or_else(|| c.strip_prefix("cookie:"))
        .unwrap_or(c);
    c.trim().to_owned()
}

/// Stessi cookie, in qualunque ordine. L'esportazione dalla finestra di login ordina, la
/// rotazione del trasporto accoda: confrontare le stringhe segnalerebbe un cambio a ogni avvio.
pub fn same_jar(a: &str, b: &str) -> bool {
    let pairs = |s: &str| {
        let mut v: Vec<String> = s
            .split(';')
            .map(|kv| kv.trim().to_owned())
            .filter(|kv| !kv.is_empty())
            .collect();
        v.sort();
        v
    };
    pairs(a) == pairs(b)
}

/// Lo stesso account Google: SAPISID resta stabile mentre i cookie `*SIDTS` ruotano.
pub fn same_account(a: &str, b: &str) -> bool {
    matches!(
        (innertube::cookie_sapisid(a), innertube::cookie_sapisid(b)),
        (Some(x), Some(y)) if x == y
    )
}

/// Costruisce il Cookie header per music.youtube.com dai cookie della finestra di login
/// (dominio senza punto iniziale, nome, valore). Tiene solo youtube.com e sottodomini; a parità
/// di nome vince il dominio più specifico, come in un browser.
pub fn youtube_cookie_header<I>(cookies: I) -> String
where
    I: IntoIterator<Item = (String, String, String)>,
{
    let mut list: Vec<(String, String, String)> = cookies
        .into_iter()
        .map(|(d, n, v)| (d.trim_start_matches('.').to_owned(), n, v))
        .filter(|(d, _, _)| d == "youtube.com" || d.ends_with(".youtube.com"))
        .collect();
    list.sort_by_key(|(d, _, _)| d.len());
    let mut jar = std::collections::BTreeMap::new();
    for (_, name, value) in list {
        jar.insert(name, value);
    }
    jar.into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(d: &str, n: &str, v: &str) -> (String, String, String) {
        (d.into(), n.into(), v.into())
    }

    #[test]
    fn keeps_only_the_youtube_jar() {
        let h = youtube_cookie_header(vec![
            c(".youtube.com", "SAPISID", "abc"),
            c("youtube.com", "SID", "def"),
            c("music.youtube.com", "VISITOR_INFO1_LIVE", "xyz"),
            c(".google.com", "SAPISID", "nope"),
            c("", "nodomain", "1"),
            c("evilyoutube.com", "X", "1"),
        ]);
        assert_eq!(h, "SAPISID=abc; SID=def; VISITOR_INFO1_LIVE=xyz");
        assert_eq!(innertube::cookie_sapisid(&h), Some("abc"));
    }

    #[test]
    fn the_most_specific_domain_wins() {
        let h = youtube_cookie_header(vec![
            c("music.youtube.com", "PREF", "specific"),
            c(".youtube.com", "PREF", "broad"),
        ]);
        assert_eq!(h, "PREF=specific");
    }

    #[test]
    fn jar_comparison_ignores_order() {
        assert!(same_jar("SAPISID=a; SID=b", "SID=b;SAPISID=a"));
        assert!(!same_jar(
            "SAPISID=a; __Secure-3PSIDTS=new",
            "SAPISID=a; __Secure-3PSIDTS=old"
        ));
    }

    #[test]
    fn account_identity_is_sapisid() {
        assert!(same_account("SAPISID=a; X=1", "X=2; SAPISID=a"));
        assert!(!same_account("SAPISID=a", "SAPISID=b"));
        assert!(!same_account("X=1", "X=1"));
    }

    #[test]
    fn normalize_strips_the_header_name() {
        assert_eq!(normalize("  Cookie: SAPISID=a; B=c \n"), "SAPISID=a; B=c");
        assert_eq!(normalize("SAPISID=a"), "SAPISID=a");
    }

    #[test]
    fn store_round_trip_and_clear() {
        let dir = std::env::temp_dir().join(format!("ambra-store-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = Store::new(&dir);
        assert_eq!(store.load(), None);
        store.save("SAPISID=secret; SID=x").unwrap();
        assert!(store.exists());
        assert_eq!(store.load().as_deref(), Some("SAPISID=secret; SID=x"));
        store.clear();
        assert_eq!(store.load(), None);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_jar_without_sapisid_is_not_restored() {
        let dir = std::env::temp_dir().join(format!("ambra-store-nosid-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = Store::new(&dir);
        store.save("PREF=x").unwrap();
        assert_eq!(store.load(), None);
        let _ = std::fs::remove_dir_all(dir);
    }
}
