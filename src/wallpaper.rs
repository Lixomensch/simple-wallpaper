use std::fs;
use std::path::{Path, PathBuf};

use rand::prelude::IndexedRandom;
use walkdir::WalkDir;

use crate::backends;
use crate::utils::{is_image, to_sorted_entries};

pub fn wallpaper_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "Variável de ambiente HOME não definida".to_string())?;
    let dir = PathBuf::from(home).join(".local/share/simple-wallpaper/wallpapers");
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Erro ao criar diretório de wallpapers: {e}"))?;
    Ok(dir)
}

pub fn list_images(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .filter(|p| is_image(p))
        .collect()
}

fn matches_all_words(filename: &str, words: &[&str]) -> bool {
    let lower = filename.to_lowercase();
    words.iter().all(|w| lower.contains(*w))
}

pub fn find_by_words(dir: &Path, query: &str) -> Vec<PathBuf> {
    let words: Vec<&str> = query.split_whitespace().collect();

    if words.is_empty() {
        return Vec::new();
    }

    list_images(dir)
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| matches_all_words(n, &words))
        })
        .collect()
}

pub fn random_image(dir: &Path) -> Option<PathBuf> {
    let images = list_images(dir);
    let mut rng = rand::rng();
    images.choose(&mut rng).cloned()
}

pub fn set(path: &Path) -> Result<PathBuf, String> {

    let img = path.to_path_buf();

    backends::apply(&img)?;
    Ok(img)
}

pub fn random() -> Result<PathBuf, String> {
    let dir = wallpaper_dir()?;
    let img = random_image(&dir).ok_or(
        "Nenhuma imagem encontrada. Adicione imagens em `swp path` primeiro.",
    )?;
    backends::apply(&img)?;
    Ok(img)
}

pub fn pick_from_entries(entries: Vec<(String, PathBuf)>, prompt: &str) -> Result<PathBuf, String> {
    let count = entries.len();
    let names: Vec<String> = entries.iter().map(|(n, _)| n.clone()).collect();

    let selected = inquire::Select::new(prompt, names)
        .with_page_size(14.min(count))
        .with_help_message(
            "↑↓ para navegar  •  Digite para filtrar  •  Enter confirma  •  Esc cancela",
        )
        .prompt()
        .map_err(|e| match e {
            inquire::InquireError::OperationCanceled
            | inquire::InquireError::OperationInterrupted => "Cancelado.".to_string(),
            other => other.to_string(),
        })?;

    entries
        .into_iter()
        .find(|(name, _)| name == &selected)
        .map(|(_, path)| path)
        .ok_or_else(|| "Seleção inválida.".to_string())
}

pub fn interactive_pick() -> Result<PathBuf, String> {
    let dir = wallpaper_dir()?;
    let images = list_images(&dir);

    if images.is_empty() {
        return Err(format!(
            "Nenhum wallpaper encontrado em {}.\n\
             Adicione imagens e tente novamente (`swp list` para conferir).",
            dir.display()
        ));
    }

    pick_from_entries(to_sorted_entries(images), "Escolha um wallpaper:")
}

pub fn pick_from_matches(matches: Vec<PathBuf>, query: &str) -> Result<PathBuf, String> {
    let entries = to_sorted_entries(matches);
    let count = entries.len();
    let prompt = format!(
        "{} resultado(s) para \"{}\" — escolha:",
        count, query
    );
    pick_from_entries(entries, &prompt)
}

pub fn resolve_set_input(input: &str) -> Result<PathBuf, String> {
    let literal = PathBuf::from(input);
    if literal.exists() {
        return Ok(literal);
    }

    let dir = wallpaper_dir()?;
    let mut matches = find_by_words(&dir, input);

    match matches.len() {
        0 => Err(format!(
            "Nenhum wallpaper encontrado para '{}'.\n\
             Use `swp list` para ver os disponíveis ou `swp path` para abrir a pasta.",
            input
        )),
        1 => Ok(matches.remove(0)),
        _ => {
            matches.sort();
            pick_from_matches(matches, input)
        }
    }
}
