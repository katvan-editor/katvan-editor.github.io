use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tera::Tera;

#[derive(argh::FromArgs)]
#[argh(description = "Katvan homepage generator")]
struct Parameters {
    #[argh(positional, description = "output directory")]
    out_dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let tera = Tera::new("templates/**/*.html")?;

    let params: Parameters = argh::from_env();
    let out_dir = &params.out_dir;

    std::fs::create_dir_all(out_dir).context("Failed creating output directory")?;

    generate_index(&tera, out_dir).context("Failed to generate index page")?;

    copy_directory_tree(Path::new("assets"), &out_dir.join("assets"))
        .context("Failed to copy assets")?;

    copy_directory_tree(Path::new(".well-known"), &out_dir.join(".well-known"))
        .context("Failed to copy well-known files")?;

    Ok(())
}

fn generate_index(tera: &Tera, out_dir: &Path) -> anyhow::Result<()> {
    let release = get_latest_release()?;

    let mut context = tera::Context::new();
    context.insert("release", &release);

    let page = tera.render("index.html", &context)?;
    std::fs::write(out_dir.join("index.html"), page)?;

    Ok(())
}

fn copy_directory_tree(src: &Path, dest: &Path) -> anyhow::Result<()> {
    // Based on https://stackoverflow.com/a/65192210
    if !dest.is_dir() {
        std::fs::create_dir(dest)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            // Intentionally flatten the tree at the destination
            copy_directory_tree(&entry.path(), dest)?;
        } else {
            std::fs::copy(entry.path(), dest.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct ReleaseData {
    #[serde(rename(deserialize = "tag_name"))]
    version: String,
    published_at: String,
    html_url: String,
}

fn get_latest_release() -> anyhow::Result<ReleaseData> {
    let mut data = ureq::get("https://api.github.com/repos/IgKh/katvan/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .call()?
        .body_mut()
        .read_json::<ReleaseData>()?;

    data.version = String::from(data.version.trim_start_matches('v'));

    Ok(data)
}
