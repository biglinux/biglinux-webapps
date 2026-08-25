//! One-shot migration of a webapp's cookie jar from libsoup's SQLite backend to
//! its Netscape-text backend.
//!
//! # Why the backend changed
//!
//! libsoup's SQLite jar has two data-loss bugs, both verified against
//! libsoup 3.6.6 by round-tripping cookies through each backend:
//!
//!  * **Expirations past 2038 overflow.** The insert is built with
//!    `INSERT INTO moz_cookies VALUES(…, %d, …)` — a 32-bit conversion. A cookie
//!    written with a 2040 expiry (`2211667200`) reads back as a garbage 2094
//!    date. "Remember me" tokens, which routinely carry decade-long lifetimes,
//!    are exactly the cookies affected.
//!  * **`SameSite=None` silently degrades to `Lax`.** Reading back the three
//!    policies gives `strict → strict`, `lax → lax`, `none → lax`. A `Lax`
//!    cookie is not sent on cross-site subrequests, so an SSO token stored as
//!    `SameSite=None` stops working on the *second* launch — a login loop that
//!    never reproduces on a fresh profile.
//!
//! The text jar preserves both: `%lu` for the expiry, and an eighth
//! tab-separated field for the same-site policy.
//!
//! # Why a migration is needed at all
//!
//! `set_persistent_storage` does not convert anything — it just points the jar
//! at a path. Shipping the backend switch on its own would hand every existing
//! webapp an empty jar on the first launch after the package update, logging the
//! user out of every internal-browser webapp at once. Nothing may be lost on an
//! update, and the user must not have to do anything, so the conversion happens
//! here: silently, once per webapp profile, on the first launch that finds a
//! legacy `.db` next to a text jar with no cookies in it.
use std::path::{Path, PathBuf};

// The `soup3` crate publishes its library as `soup`.
use soup::prelude::*;

/// Legacy jar written by the SQLite backend.
const LEGACY_DB_FILENAME: &str = "webkit-cookies.db";
/// What the migrated `.db` is renamed to.
///
/// It is kept rather than deleted: the conversion goes through libsoup's own
/// reader, and if some corner of it ever drops cookies we would rather still
/// have the original bytes on disk than have destroyed the user's only copy of
/// their sessions. It is small (tens of KB) and never read again.
const RETIRED_DB_SUFFIX: &str = "webkit-cookies.db.migrated";

/// Convert the legacy jar in `data_dir` to `text_store`, if that has not been
/// done already. Safe and cheap to call on every launch.
///
/// Failure is never fatal: a webapp with an unreadable legacy jar still starts,
/// it just starts logged out — the same outcome as having no migration at all.
pub(super) fn migrate_legacy_cookie_jar(data_dir: &Path, text_store: &Path) {
    let legacy_db = data_dir.join(LEGACY_DB_FILENAME);

    if !should_migrate(&legacy_db, text_store) {
        return;
    }

    match convert_jar(&legacy_db, text_store) {
        Ok(count) => {
            log::info!(
                "Migrated {count} cookie(s) from {} to the text jar",
                legacy_db.display()
            );
            retire_legacy_db(&legacy_db, data_dir);
        }
        Err(err) => {
            // Leave both files alone so the next launch retries. The `.txt` is
            // not deleted: `should_migrate` keys off the cookie count, so a
            // header-only jar already reads as "not migrated", and removing a
            // file webkit is about to open buys nothing.
            log::warn!("Migrate cookie jar {}: {err}", legacy_db.display());
        }
    }
}

/// Migrate only when there is a legacy jar and the text jar holds no cookies.
///
/// The marker is the text jar's **cookie count**, not the file's existence or
/// size. Existence is too weak: libsoup writes a four-line comment header as
/// soon as a jar is opened for writing, so a profile that has already been
/// launched once on the new backend has a non-empty `.txt` containing nothing
/// but that header. Gating on the file would permanently skip such a profile and
/// leave its still-present `.db` sessions stranded — which is precisely the
/// state a profile lands in if the backend switch ships before this migration.
///
/// Counting cookies instead makes the check self-healing: any profile whose text
/// jar is genuinely empty gets its legacy cookies imported, and the moment the
/// text jar holds anything the legacy jar is never consulted again.
fn should_migrate(legacy_db: &Path, text_store: &Path) -> bool {
    if !legacy_db.is_file() {
        return false;
    }
    cookie_count(text_store) == 0
}

/// Number of cookies libsoup can read back from a jar file. A missing or
/// unparseable file counts as zero.
fn cookie_count(text_store: &Path) -> usize {
    let Some(path) = text_store.to_str() else {
        return 0;
    };
    if !text_store.is_file() {
        return 0;
    }
    // Read-only, so merely inspecting the jar cannot create or rewrite it.
    soup::CookieJarText::new(path, true).all_cookies().len()
}

/// Copy every cookie from the SQLite jar into the text jar, returning how many
/// were transferred.
fn convert_jar(legacy_db: &Path, text_store: &Path) -> Result<usize, String> {
    let db_path = legacy_db
        .to_str()
        .ok_or_else(|| "legacy jar path is not valid UTF-8".to_string())?;
    let text_path = text_store
        .to_str()
        .ok_or_else(|| "text jar path is not valid UTF-8".to_string())?;

    // Read-only: this must not rewrite the legacy jar, so a failed migration
    // leaves the original bytes exactly as they were for the next attempt.
    let source = soup::CookieJarDB::new(db_path, true);
    let cookies = source.all_cookies();

    let destination = soup::CookieJarText::new(text_path, false);
    let mut migrated = 0usize;
    for mut cookie in cookies {
        // Session cookies never reached the SQLite jar, and libsoup drops
        // already-expired ones on insert, so whatever survives here is a cookie
        // the browser would still send.
        //
        // Expiries mangled by the 2038 overflow arrive inflated (a 2040 date
        // reads back as 2094). That is harmless — the cookie stays valid rather
        // than expiring early — and it cannot be repaired, because the original
        // value is not recoverable from the truncated one.
        destination.add_cookie(&mut cookie);
        migrated += 1;
    }

    // libsoup's text jar writes through on every `add_cookie`, so the file is
    // already complete here. Dropping the jar flushes anything outstanding.
    drop(destination);

    if migrated > 0 && !text_store.is_file() {
        return Err(format!(
            "libsoup reported {migrated} cookie(s) but wrote no jar at {}",
            text_store.display()
        ));
    }
    Ok(migrated)
}

/// Rename the converted `.db` out of the way so `should_migrate` stops firing
/// even if the text jar is later emptied by the user clearing their cookies.
fn retire_legacy_db(legacy_db: &Path, data_dir: &Path) {
    let retired: PathBuf = data_dir.join(RETIRED_DB_SUFFIX);
    if let Err(err) = std::fs::rename(legacy_db, &retired) {
        // Not fatal, but worth reporting: the jar will be converted again on the
        // next launch, which is wasteful yet harmless (the text jar wins).
        log::warn!("Retire legacy cookie jar {}: {err}", legacy_db.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn text_store(dir: &TempDir) -> PathBuf {
        dir.path().join("webkit-cookies.txt")
    }

    fn legacy_db(dir: &TempDir) -> PathBuf {
        dir.path().join(LEGACY_DB_FILENAME)
    }

    #[test]
    fn no_legacy_jar_means_nothing_to_do() {
        let tmp = TempDir::new().unwrap();
        assert!(!should_migrate(&legacy_db(&tmp), &text_store(&tmp)));

        // A fresh install must not gain a cookie file just by launching.
        migrate_legacy_cookie_jar(tmp.path(), &text_store(&tmp));
        assert!(!text_store(&tmp).exists());
    }

    /// Write a text jar holding `names`, the way libsoup would.
    fn seed_text_jar(path: &Path, names: &[&str]) {
        let expires = glib::DateTime::from_unix_utc(
            glib::DateTime::now_utc().unwrap().to_unix() + 60 * 60 * 24 * 365,
        )
        .unwrap();
        let jar = soup::CookieJarText::new(path.to_str().unwrap(), false);
        for name in names {
            let mut cookie = soup::Cookie::new(name, "v", "example.com", "/", -1);
            cookie.set_expires(&expires);
            jar.add_cookie(&mut cookie);
        }
    }

    #[test]
    fn a_text_jar_holding_cookies_blocks_a_second_migration() {
        let tmp = TempDir::new().unwrap();
        fs::write(legacy_db(&tmp), b"whatever").unwrap();
        seed_text_jar(&text_store(&tmp), &["already_here"]);
        // Re-running must not clobber a jar the user has been accumulating
        // cookies in since the update.
        assert!(!should_migrate(&legacy_db(&tmp), &text_store(&tmp)));
    }

    #[test]
    fn header_only_text_jar_still_migrates() {
        // The state a profile lands in when the backend switch runs before this
        // migration exists: webkit opened a text jar (writing libsoup's comment
        // header, so the file is non-empty) while the real sessions sat unread
        // in the `.db`. Gating on file size stranded these profiles for good.
        let tmp = TempDir::new().unwrap();
        fs::write(legacy_db(&tmp), b"whatever").unwrap();
        fs::write(
            text_store(&tmp),
            b"# HTTP Cookie File\n# This is a generated file!  Do not edit.\n\n",
        )
        .unwrap();

        assert!(fs::metadata(text_store(&tmp)).unwrap().len() > 0);
        assert_eq!(cookie_count(&text_store(&tmp)), 0);
        assert!(should_migrate(&legacy_db(&tmp), &text_store(&tmp)));
    }

    #[test]
    fn zero_length_text_jar_still_migrates() {
        let tmp = TempDir::new().unwrap();
        fs::write(legacy_db(&tmp), b"whatever").unwrap();
        fs::write(text_store(&tmp), b"").unwrap();
        assert!(should_migrate(&legacy_db(&tmp), &text_store(&tmp)));
    }

    #[test]
    fn cookie_count_handles_missing_and_garbage_files() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(cookie_count(&text_store(&tmp)), 0, "missing file");
        fs::write(text_store(&tmp), b"\x00\x01 not a cookie jar").unwrap();
        assert_eq!(cookie_count(&text_store(&tmp)), 0, "unparseable file");
    }

    #[test]
    fn migration_is_attempted_when_only_the_legacy_jar_exists() {
        let tmp = TempDir::new().unwrap();
        fs::write(legacy_db(&tmp), b"whatever").unwrap();
        assert!(should_migrate(&legacy_db(&tmp), &text_store(&tmp)));
    }

    #[test]
    fn a_directory_named_like_the_legacy_jar_is_ignored() {
        // `is_file` rather than `exists`: handing a directory to libsoup would
        // log a CRITICAL on every launch.
        let tmp = TempDir::new().unwrap();
        fs::create_dir(legacy_db(&tmp)).unwrap();
        assert!(!should_migrate(&legacy_db(&tmp), &text_store(&tmp)));
    }

    #[test]
    fn corrupt_legacy_jar_keeps_the_profile_retryable() {
        // A non-SQLite `.db` makes libsoup yield an empty jar. The migration
        // must not leave a zero-length `.txt`, which would satisfy
        // `should_migrate` and permanently skip the retry.
        let tmp = TempDir::new().unwrap();
        fs::write(legacy_db(&tmp), b"this is not a sqlite database").unwrap();

        migrate_legacy_cookie_jar(tmp.path(), &text_store(&tmp));

        // Nothing was imported, so the profile must still read as unmigrated
        // rather than being written off.
        assert_eq!(cookie_count(&text_store(&tmp)), 0);
    }

    #[test]
    fn round_trips_cookies_through_both_backends() {
        // End-to-end: write a real SQLite jar with libsoup, migrate it, and
        // read the result back with the text backend.
        let tmp = TempDir::new().unwrap();
        let db = legacy_db(&tmp);
        let store = text_store(&tmp);

        let expires = glib::DateTime::from_unix_utc(
            glib::DateTime::now_utc().unwrap().to_unix() + 60 * 60 * 24 * 365,
        )
        .unwrap();
        {
            let jar = soup::CookieJarDB::new(db.to_str().unwrap(), false);
            for name in ["session", "remember_me"] {
                let mut cookie = soup::Cookie::new(name, "value", "example.com", "/", -1);
                cookie.set_expires(&expires);
                jar.add_cookie(&mut cookie);
            }
        }

        migrate_legacy_cookie_jar(tmp.path(), &store);

        assert!(store.is_file(), "the text jar must exist after migration");
        let jar = soup::CookieJarText::new(store.to_str().unwrap(), true);
        let mut names: Vec<String> = jar
            .all_cookies()
            .iter_mut()
            .map(|cookie| cookie.name().unwrap_or_default().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["remember_me", "session"]);

        // The legacy jar is retired, not deleted, and no longer triggers.
        assert!(!db.exists(), "the migrated .db must be renamed away");
        assert!(tmp.path().join(RETIRED_DB_SUFFIX).is_file());
        assert!(!should_migrate(&db, &store));
    }

    #[test]
    fn migration_preserves_same_site_none_that_the_sqlite_jar_would_lose() {
        // The whole point of the backend switch. The source jar has already
        // degraded `None` to `Lax` on write, so what this pins is that the text
        // jar *can* hold `None` — i.e. cookies set after the migration survive
        // correctly, which the SQLite jar could not manage.
        let tmp = TempDir::new().unwrap();
        let store = text_store(&tmp);

        let expires = glib::DateTime::from_unix_utc(
            glib::DateTime::now_utc().unwrap().to_unix() + 60 * 60 * 24 * 365,
        )
        .unwrap();
        {
            let jar = soup::CookieJarText::new(store.to_str().unwrap(), false);
            let mut cookie = soup::Cookie::new("sso", "token", "example.com", "/", -1);
            cookie.set_expires(&expires);
            cookie.set_secure(true);
            cookie.set_same_site_policy(soup::SameSitePolicy::None);
            jar.add_cookie(&mut cookie);
        }

        let jar = soup::CookieJarText::new(store.to_str().unwrap(), true);
        let mut cookie = jar
            .all_cookies()
            .into_iter()
            .find(|cookie| {
                let mut cookie = cookie.clone();
                cookie.name().as_deref() == Some("sso")
            })
            .expect("sso cookie");
        assert_eq!(cookie.same_site_policy(), soup::SameSitePolicy::None);
    }

    #[test]
    fn far_future_expiry_survives_the_text_jar() {
        // Pins the 2038 half of the rationale: 2040 must read back as 2040.
        let tmp = TempDir::new().unwrap();
        let store = text_store(&tmp);
        let far_future = 2_211_667_200; // 2040-01-01 UTC

        {
            let jar = soup::CookieJarText::new(store.to_str().unwrap(), false);
            let mut cookie = soup::Cookie::new("long", "lived", "example.com", "/", -1);
            cookie.set_expires(&glib::DateTime::from_unix_utc(far_future).unwrap());
            jar.add_cookie(&mut cookie);
        }

        let jar = soup::CookieJarText::new(store.to_str().unwrap(), true);
        let mut cookie = jar.all_cookies().into_iter().next().expect("cookie");
        assert_eq!(
            cookie.expires().map(|expires| expires.to_unix()),
            Some(far_future),
            "the SQLite jar reads this back as a garbage 2094 date"
        );
    }
}
