//! A small stand-in for Laravel's `@vite` directive.
//!
//! While `npm run dev` is running, the Vite plugin in `vite.config.ts` writes
//! the dev server's URL to `public/hot`, and the tags point at the dev server
//! (with React Fast Refresh). Otherwise they're read from the build manifest
//! in `public/build/.vite/manifest.json`, whose hash is the asset version.

use std::collections::HashMap;
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

    /// The dev server URL, while it's running.
    pub fn hot(&self) -> Option<String> {
        fs::read_to_string(&self.hot_file)
            .ok()
            .map(|url| url.trim().trim_end_matches('/').to_owned())
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

        let Some(entry) = manifest.get(ENTRY) else {
            return String::new();
        };

        let mut tags = Vec::new();
        let mut styles = entry.css.clone();
        for import in &entry.imports {
            if let Some(chunk) = manifest.get(import) {
                styles.extend(chunk.css.iter().cloned());
                tags.push(format!(
                    r#"<link rel="modulepreload" href="{}/{}">"#,
                    self.build_url, chunk.file
                ));
            }
        }
        for css in styles {
            tags.insert(
                0,
                format!(r#"<link rel="stylesheet" href="{}/{css}">"#, self.build_url),
            );
        }
        tags.push(format!(
            r#"<script type="module" src="{}/{}"></script>"#,
            self.build_url, entry.file
        ));

        tags.join("\n")
    }
}
