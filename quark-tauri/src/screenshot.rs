//! Capture d'écran globale (F12), sans dépendre d'un outil graphique externe.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn directory() -> PathBuf {
    quark::config::data_dir().join("screenshots")
}

pub fn path_for(game: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    directory()
        .join(safe_name(game))
        .join(format!("{stamp}.png"))
}

fn safe_name(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in name.chars() {
        let keep = ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.');
        if keep {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "jeu".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Bruit d'obturateur, comme le fait Steam. Silencieux si le système n'a pas
/// de lecteur ou de sonnerie installée : la capture reste le résultat attendu.
pub fn play_shutter() {
    let attempts: [(&str, &[&str]); 2] = [
        ("canberra-gtk-play", &["-i", "camera-shutter"]),
        (
            "paplay",
            &["/usr/share/sounds/freedesktop/stereo/camera-shutter.oga"],
        ),
    ];
    for (program, args) in attempts {
        if std::process::Command::new(program)
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
        {
            return;
        }
    }
}

pub fn capture(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| "dossier de captures inaccessible".to_string())?;
    }
    if std::env::var_os("DISPLAY").is_some() && capture_x11(path).is_ok() {
        return Ok(());
    }
    capture_external(path)
}

fn capture_x11(path: &Path) -> Result<(), String> {
    use x11_dl::xlib;
    let xlib = xlib::Xlib::open().map_err(|_| "X11 indisponible".to_string())?;
    let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };
    if display.is_null() {
        return Err("connexion X11 impossible".to_string());
    }
    let root = unsafe { (xlib.XDefaultRootWindow)(display) };
    let width = unsafe { (xlib.XDisplayWidth)(display, 0) };
    let height = unsafe { (xlib.XDisplayHeight)(display, 0) };
    if width <= 0 || height <= 0 {
        unsafe { (xlib.XCloseDisplay)(display) };
        return Err("taille d'écran inconnue".to_string());
    }
    let image = unsafe {
        (xlib.XGetImage)(
            display,
            root,
            0,
            0,
            width as u32,
            height as u32,
            !0,
            xlib::ZPixmap,
        )
    };
    if image.is_null() {
        unsafe { (xlib.XCloseDisplay)(display) };
        return Err("capture X11 refusée".to_string());
    }
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    let (red_mask, green_mask, blue_mask) =
        unsafe { ((*image).red_mask, (*image).green_mask, (*image).blue_mask) };
    for y in 0..height {
        for x in 0..width {
            let pixel = unsafe { (xlib.XGetPixel)(image, x, y) };
            pixels.push(color_channel(pixel, red_mask));
            pixels.push(color_channel(pixel, green_mask));
            pixels.push(color_channel(pixel, blue_mask));
            pixels.push(255);
        }
    }
    unsafe {
        (xlib.XDestroyImage)(image);
        (xlib.XCloseDisplay)(display);
    }
    image::save_buffer(
        path,
        &pixels,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .map_err(|_| "écriture de la capture impossible".to_string())
}

fn color_channel(pixel: u64, mask: u64) -> u8 {
    if mask == 0 {
        return 0;
    }
    let shift = mask.trailing_zeros();
    let max = mask >> shift;
    if max == 0 {
        return 0;
    }
    (((pixel & mask) >> shift) * 255 / max) as u8
}

fn capture_external(path: &Path) -> Result<(), String> {
    let mut attempts: Vec<(&str, Vec<String>)> = Vec::new();
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        attempts.push(("grim", vec![path.display().to_string()]));
    }
    attempts.push((
        "gnome-screenshot",
        vec!["-f".into(), path.display().to_string()],
    ));
    attempts.push((
        "spectacle",
        vec![
            "-b".into(),
            "-n".into(),
            "-o".into(),
            path.display().to_string(),
        ],
    ));
    attempts.push(("scrot", vec![path.display().to_string()]));
    attempts.push((
        "import",
        vec!["-window".into(), "root".into(), path.display().to_string()],
    ));
    for (program, args) in attempts {
        if std::process::Command::new(program)
            .args(&args)
            .status()
            .is_ok_and(|status| status.success())
        {
            return Ok(());
        }
    }
    Err("aucun outil de capture disponible".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_produit_un_png() {
        if std::env::var_os("DISPLAY").is_none() {
            return;
        }
        let path = std::env::temp_dir().join(format!("quark-capture-{}.png", std::process::id()));
        capture(&path).expect("capture X11");
        assert!(std::fs::metadata(&path).expect("capture créée").len() > 0);
        let _ = std::fs::remove_file(path);
    }
}
