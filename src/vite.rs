//! A small stand-in for Laravel's `@vite` directive.
//!
//! While `npm run dev` is running, the Vite plugin in `vite.config.ts` writes
//! the dev server's URL to `public/hot`, and the tags point at the dev server
//! (with React Fast Refresh). Otherwise they're read from the build manifest
//! in `public/build/.vite/manifest.json`, whose hash is the asset version.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

use serde::Deserialize;

pub const ENTRY: &str = "resources/js/app.tsx";

#[derive(Clone, Debug)]
pub struct Vite {
    hot_file: PathBuf,
    manifest: PathBuf,
    build_url: String,
}

#[derive(Deserialize)]
struct Chunk {
    file: String,
    #[serde(default)]
    css: Vec<String>,
    #[serde(default)]
    imports: Vec<String>,
}

impl Default for Vite {
    fn default() -> Self {
        Self::new()
    }
}

impl Vite {
    pub fn new() -> Self {
        Self {
            hot_file: "public/hot".into(),
            manifest: "public/build/.vite/manifest.json".into(),
            build_url: "/build".into(),
        }
    }

    /// The dev server URL, while it's running. Only a local `http(s)` URL is
    /// accepted: whatever is in the hot file decides where every script on
    /// the page comes from.
    pub fn hot(&self) -> Option<String> {
        let url = fs::read_to_string(&self.hot_file).ok()?;
        let url = url.trim().trim_end_matches('/');
        if is_local_url(url) {
            Some(url.to_owned())
        } else {
            tracing::warn!(
                url,
                "ignoring a hot file that doesn't point at a local dev server"
            );
            None
        }
    }

    /// A hash of the build manifest: it changes whenever the assets do.
    /// In development the dev server serves the latest code, so it's "dev".
    pub fn version(&self) -> String {
        if self.hot().is_some() {
            return "dev".to_owned();
        }

        match fs::read(&self.manifest) {
            Ok(bytes) => {
                let mut hasher = DefaultHasher::new();
                bytes.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            }
            Err(_) => String::new(),
        }
    }

    /// The `<script>` and `<link>` tags for the app's entry point.
    pub fn tags(&self) -> String {
        if let Some(url) = self.hot() {
            return format!(
                r#"<script type="module">
import RefreshRuntime from "{url}/@react-refresh";
RefreshRuntime.injectIntoGlobalHook(window);
window.$RefreshReg$ = () => {{}};
window.$RefreshSig$ = () => (type) => type;
window.__vite_plugin_react_preamble_installed__ = true;
</script>
<script type="module" src="{url}/@vite/client"></script>
<script type="module" src="{url}/{ENTRY}"></script>"#
            );
        }

        let Some(manifest) = fs::read_to_string(&self.manifest)
            .ok()
            .and_then(|json| serde_json::from_str::<HashMap<String, Chunk>>(&json).ok())
        else {
            tracing::warn!(
                "no Vite manifest: run `npm run build`, or `npm run dev` for the dev server"
            );
            return String::new();
        };

        tags_for(&manifest, ENTRY, &self.build_url)
    }
}

/// The tags for `entry`: stylesheets first, in the order Vite would load them
/// (an import's CSS before its importer's), then a `modulepreload` for every
/// chunk the entry imports, however deep, then the entry's script.
fn tags_for(manifest: &HashMap<String, Chunk>, entry: &str, build_url: &str) -> String {
    let Some(chunk) = manifest.get(entry) else {
        return String::new();
    };

    let mut seen = HashSet::from([entry]);
    let mut styles = Vec::new();
    let mut preloads = Vec::new();
    collect(manifest, chunk, &mut seen, &mut styles, &mut preloads);

    let asset = |file: &str| escape(&format!("{build_url}/{file}"));
    let mut tags: Vec<String> = styles
        .iter()
        .map(|css| format!(r#"<link rel="stylesheet" href="{}">"#, asset(css)))
        .collect();
    tags.extend(
        preloads
            .iter()
            .map(|file| format!(r#"<link rel="modulepreload" href="{}">"#, asset(file))),
    );
    tags.push(format!(
        r#"<script type="module" src="{}"></script>"#,
        asset(&chunk.file)
    ));

    tags.join("\n")
}

/// Walk `chunk`'s imports depth first: each import's CSS and preloads come
/// before the chunk's own CSS, and each chunk is visited once.
fn collect<'a>(
    manifest: &'a HashMap<String, Chunk>,
    chunk: &'a Chunk,
    seen: &mut HashSet<&'a str>,
    styles: &mut Vec<&'a str>,
    preloads: &mut Vec<&'a str>,
) {
    for import in &chunk.imports {
        if !seen.insert(import) {
            continue;
        }
        if let Some(imported) = manifest.get(import) {
            collect(manifest, imported, seen, styles, preloads);
            preloads.push(&imported.file);
        }
    }
    for css in &chunk.css {
        if !styles.contains(&css.as_str()) {
            styles.push(css);
        }
    }
}

/// A dev server URL on this machine, made only of characters that need no
/// escaping, since it's also written into an inline `<script>`.
fn is_local_url(url: &str) -> bool {
    if !url
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ":/.[]-_".contains(c))
    {
        return false;
    }
    let Some(rest) = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
    else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = match host.rsplit_once(':') {
        Some((host, port)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => host,
        _ => host,
    };
    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
}

/// Escape a value for a double-quoted HTML attribute.
fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(file: &str, css: &[&str], imports: &[&str]) -> Chunk {
        Chunk {
            file: file.into(),
            css: css.iter().map(|s| s.to_string()).collect(),
            imports: imports.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn styles_keep_their_order_and_nested_imports_are_preloaded() {
        let manifest = HashMap::from([
            (
                ENTRY.to_owned(),
                chunk("app.js", &["app.css"], &["_a.js", "_b.js"]),
            ),
            ("_a.js".to_owned(), chunk("a.js", &["a.css"], &["_c.js"])),
            ("_b.js".to_owned(), chunk("b.js", &["b.css"], &["_c.js"])),
            ("_c.js".to_owned(), chunk("c.js", &["c.css"], &[])),
        ]);

        let tags = tags_for(&manifest, ENTRY, "/build");

        assert_eq!(
            tags.lines().collect::<Vec<_>>(),
            [
                r#"<link rel="stylesheet" href="/build/c.css">"#,
                r#"<link rel="stylesheet" href="/build/a.css">"#,
                r#"<link rel="stylesheet" href="/build/b.css">"#,
                r#"<link rel="stylesheet" href="/build/app.css">"#,
                r#"<link rel="modulepreload" href="/build/c.js">"#,
                r#"<link rel="modulepreload" href="/build/a.js">"#,
                r#"<link rel="modulepreload" href="/build/b.js">"#,
                r#"<script type="module" src="/build/app.js"></script>"#,
            ]
        );
    }

    #[test]
    fn asset_paths_are_escaped() {
        let manifest = HashMap::from([(ENTRY.to_owned(), chunk(r#"x"><script>.js"#, &[], &[]))]);
        let tags = tags_for(&manifest, ENTRY, "/build");
        assert!(!tags.contains("\"><script>"));
        assert!(tags.contains("&quot;&gt;&lt;script&gt;"));
    }

    #[test]
    fn only_local_dev_server_urls_are_trusted() {
        for url in [
            "http://localhost:5173",
            "http://127.0.0.1:5173",
            "https://[::1]:5173",
            "http://localhost",
        ] {
            assert!(is_local_url(url), "{url}");
        }
        for url in [
            "https://evil.example",
            "http://localhost.evil.example:5173",
            "http://localhost@evil.example",
            "javascript:alert(1)",
            "//localhost:5173",
            "http://localhost:5173/\"><script>",
        ] {
            assert!(!is_local_url(url), "{url}");
        }
    }
}
