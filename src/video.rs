//! Lecture intégrée des bandes-annonces Steam.
//!
//! GStreamer est lancé dans un processus séparé : Quark reste lançable sans
//! GStreamer, et un plantage du décodeur ne peut pas emporter la fenêtre. Le
//! pipeline ne rend pas de vidéo à l'écran ; il écrit des images RGBA brutes
//! dans un tube, qu'egui transforme en texture. L'audio sort par la sortie
//! audio habituelle de GStreamer.
//!
//! Les bandes-annonces Steam récentes sont en HLS/DASH. `playbin` choisit le
//! décodeur et le démultiplexeur adaptés (souphttpsrc + hlsdemux + avdec).

use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, TryRecvError, sync_channel};
use std::time::{Duration, Instant};

/// Taille du flux vidéo : assez nette pour la grande vue de la fiche, sans
/// recopier inutilement une image 1080p à chaque image de l'interface.
pub const WIDTH: usize = 960;
pub const HEIGHT: usize = 540;
const FRAME_BYTES: usize = WIDTH * HEIGHT * 4;
const START_TIMEOUT: Duration = Duration::from_secs(25);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Starting,
    Playing,
    Paused,
    Ended,
    Failed(String),
}

enum Event {
    Frame(Vec<u8>),
    Ended,
    Error(String),
}

/// Un lecteur vidéo pour une seule bande-annonce.
pub struct MediaPlayer {
    child: Child,
    rx: Receiver<Event>,
    texture: Option<TextureHandle>,
    state: PlaybackState,
    started: Instant,
    url: String,
}

impl MediaPlayer {
    /// Démarre la lecture dans un processus GStreamer isolé.
    pub fn start(url: &str) -> Result<Self, String> {
        let binary = gst_launch().ok_or_else(|| {
            "GStreamer est absent : installez gstreamer1.0-tools, \
             gstreamer1.0-plugins-good et gstreamer1.0-libav."
                .to_string()
        })?;
        let audio =
            std::env::var("QUARK_GST_AUDIO_SINK").unwrap_or_else(|_| "autoaudiosink".to_string());
        let args = playback_args(url, &audio);
        let mut child = Command::new(&binary)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("GStreamer n'a pas pu démarrer : {err}"))?;
        let Some(mut stdout) = child.stdout.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err("GStreamer n'a pas fourni de flux vidéo.".to_string());
        };
        let Some(mut stderr) = child.stderr.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err("GStreamer n'a pas fourni de diagnostic.".to_string());
        };

        // Deux images d'avance suffisent : si l'interface ralentit, le tube
        // remonte la pression jusqu'au décodeur au lieu d'accumuler la vidéo.
        let (tx, rx) = sync_channel(2);
        let frame_tx = tx.clone();
        std::thread::spawn(move || {
            let mut buffer = vec![0_u8; FRAME_BYTES];
            loop {
                match stdout.read_exact(&mut buffer) {
                    Ok(()) => {
                        let frame = std::mem::take(&mut buffer);
                        if frame_tx.send(Event::Frame(frame)).is_err() {
                            break;
                        }
                        buffer = vec![0_u8; FRAME_BYTES];
                    }
                    Err(_) => {
                        let _ = frame_tx.send(Event::Ended);
                        break;
                    }
                }
            }
        });
        std::thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr.read_to_string(&mut text);
            if let Some(message) = gstreamer_error(&text) {
                let _ = tx.send(Event::Error(message));
            }
        });

        Ok(Self {
            child,
            rx,
            texture: None,
            state: PlaybackState::Starting,
            started: Instant::now(),
            url: url.to_string(),
        })
    }

    /// Traite les images et les erreurs reçues depuis GStreamer.
    pub fn poll(&mut self, ctx: &Context) {
        let mut received_frame = false;
        loop {
            match self.rx.try_recv() {
                Ok(Event::Frame(pixels)) => {
                    let image = ColorImage::from_rgba_unmultiplied([WIDTH, HEIGHT], &pixels);
                    match &mut self.texture {
                        Some(texture) => texture.set(image, TextureOptions::LINEAR),
                        None => {
                            self.texture = Some(ctx.load_texture(
                                "quark_media_video",
                                image,
                                TextureOptions::LINEAR,
                            ));
                        }
                    }
                    self.state = PlaybackState::Playing;
                    received_frame = true;
                }
                Ok(Event::Ended) => {
                    if !matches!(self.state, PlaybackState::Failed(_)) {
                        self.state = PlaybackState::Ended;
                    }
                }
                Ok(Event::Error(message)) => {
                    self.state = PlaybackState::Failed(message);
                    self.terminate();
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if matches!(self.state, PlaybackState::Starting) {
                        self.state = PlaybackState::Ended;
                    }
                    break;
                }
            }
        }
        if matches!(self.state, PlaybackState::Starting) && self.started.elapsed() >= START_TIMEOUT
        {
            self.state = PlaybackState::Failed(
                "La vidéo ne démarre pas. Vérifiez que les greffons HLS et H.264 sont installés."
                    .to_string(),
            );
            self.terminate();
        }
        if received_frame {
            ctx.request_repaint();
        } else if matches!(self.state, PlaybackState::Starting) {
            // Le premier tampon HLS peut mettre quelques secondes à arriver :
            // continuer à interroger le processus sans bloquer l'interface.
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            PlaybackState::Playing => {
                self.signal(signal_stop());
                self.state = PlaybackState::Paused;
            }
            PlaybackState::Paused => {
                self.signal(signal_continue());
                self.state = PlaybackState::Playing;
            }
            _ => {}
        }
    }

    pub fn stop(&mut self) {
        self.terminate();
        self.texture = None;
        self.state = PlaybackState::Ended;
    }

    pub fn texture(&self) -> Option<&TextureHandle> {
        self.texture.as_ref()
    }

    pub fn state(&self) -> &PlaybackState {
        &self.state
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    fn signal(&self, signal: i32) {
        #[cfg(unix)]
        unsafe {
            libc::kill(self.child.id() as i32, signal);
        }
        #[cfg(not(unix))]
        let _ = signal;
    }

    fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for MediaPlayer {
    fn drop(&mut self) {
        self.terminate();
    }
}

/// Construit les arguments du pipeline : vidéo RGBA vers stdout, audio vers la
/// sortie choisie par GStreamer.
fn playback_args(url: &str, audio_sink: &str) -> Vec<String> {
    let sink = format!(
        "videoconvert ! videoscale ! video/x-raw,format=RGBA,width={WIDTH},height={HEIGHT} ! \
         fdsink fd=1 sync=true"
    );
    vec![
        "-q".to_string(),
        "playbin".to_string(),
        format!("uri={url}"),
        format!("video-sink={sink}"),
        format!("audio-sink={audio_sink}"),
    ]
}

fn gst_launch() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("QUARK_GST_LAUNCH") {
        let path = PathBuf::from(path);
        return path.is_file().then_some(path);
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|directory| directory.join("gst-launch-1.0"))
        .find(|candidate| candidate.is_file())
}

/// Ne remonte que les vraies erreurs de GStreamer, pas ses avertissements de
/// greffons optionnels.
#[cfg(unix)]
fn signal_stop() -> i32 {
    libc::SIGSTOP
}

#[cfg(not(unix))]
fn signal_stop() -> i32 {
    0
}

#[cfg(unix)]
fn signal_continue() -> i32 {
    libc::SIGCONT
}

#[cfg(not(unix))]
fn signal_continue() -> i32 {
    0
}

fn gstreamer_error(stderr: &str) -> Option<String> {
    let line = stderr
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with("ERROR"))?;
    let line = line.trim();
    let line = line.strip_prefix("ERROR: ").unwrap_or(line);
    Some(line.chars().take(240).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_avertissements_de_greffons_ne_sont_pas_des_erreurs() {
        assert_eq!(gstreamer_error("WARNING: plugin not found"), None);
        assert_eq!(
            gstreamer_error("ERROR: from element /GstPlayBin:playbin0: bad stream"),
            Some("from element /GstPlayBin:playbin0: bad stream".to_string())
        );
    }

    #[test]
    #[ignore = "nécessite GStreamer et le réseau Steam"]
    fn lit_une_vraie_bande_annonce_hls() {
        let media = crate::media::fetch(&crate::artwork::http(), 292030, "french", "fr")
            .expect("médias Steam");
        let url = media.trailers[0].video.clone();
        let ctx = Context::default();
        let mut player = MediaPlayer::start(&url).expect("lecteur GStreamer");
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            player.poll(&ctx);
            if player.texture().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(
            player.texture().is_some(),
            "aucune image reçue : {:?}",
            player.state()
        );
        player.stop();
    }

    #[test]
    fn le_pipeline_sort_des_images_rgba_et_garde_l_audio() {
        let args = playback_args("https://video.test/master.m3u8", "autoaudiosink");
        assert!(args.contains(&"playbin".to_string()));
        assert!(args.contains(&"uri=https://video.test/master.m3u8".to_string()));
        let sink = args
            .iter()
            .find(|arg| arg.starts_with("video-sink="))
            .unwrap();
        assert!(sink.contains("format=RGBA"));
        assert!(sink.contains("width=960"));
        assert!(sink.contains("height=540"));
        assert!(sink.contains("fdsink fd=1 sync=true"));
        assert!(args.contains(&"audio-sink=autoaudiosink".to_string()));
    }
}
