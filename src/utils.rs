use std::path::Path;
use std::time::Duration;
use std::path::PathBuf;


const EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "avif"];

pub fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn parse_interval(interval: &str) -> Result<Duration, String> {
    let last = interval.chars().last().unwrap_or('\0');

    if !"smh".contains(last) {
        return Err(format!(
            "Formato de intervalo inválido: '{}'. \
             Use um número seguido de s, m ou h (ex: 30s, 10m, 2h).",
            interval
        ));
    }

    let num_part = &interval[..interval.len() - 1];
    let n = num_part.parse::<u64>().map_err(|_| {
        format!(
            "Número inválido em '{}'. Use um inteiro positivo (ex: 30s, 10m, 2h).",
            interval
        )
    })?;

    if n == 0 {
        return Err("O intervalo deve ser maior que zero.".into());
    }

    match last {
        's' => Ok(Duration::from_secs(n)),
        'm' => Ok(Duration::from_secs(n * 60)),
        'h' => Ok(Duration::from_secs(n * 3600)),
        _ => unreachable!(),
    }
}

pub fn to_sorted_entries(images: Vec<PathBuf>) -> Vec<(String, PathBuf)> {
    let mut entries: Vec<(String, PathBuf)> = images
        .into_iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?.to_owned();
            Some((name, p))
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}