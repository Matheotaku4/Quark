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
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError, sync_channel};
use std::time::{Duration, Instant};

/// Taille du flux vidéo : assez nette pour la grande vue de la fiche, sans
/// recopier inutilement une image 1080p à chaque image de l'interface.
pub const WIDTH: usize = 960;
pub const HEIGHT: usize = 540;
const FRAME_BYTES: usize = WIDTH * HEIGHT * 4;
const START_TIMEOUT: Duration = Duration::from_secs(25);
/// Le premier tampon HLS peut mettre quelques secondes à arriver.
const STARTUP_REPAINT: Duration = Duration::from_millis(100);
/// Sans réveil régulier, egui ne repeint qu'à la prochaine entrée (souris,
/// clavier) : la lecture s'arrêtait dès que la souris ne bougeait plus.
const PLAYING_REPAINT: Duration = Duration::from_millis(16);
/// Après la fin du processus, le diagnostic stderr peut arriver juste après
/// `Ended` : on continue à interroger le canal pendant cette courte fenêtre
/// pour laisser sa chance au secours audio.
const AUDIO_FALLBACK_WATCH: Duration = Duration::from_secs(5);

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
    volume: f32,
    volume_tx: Option<Sender<f32>>,
    volume_supported: bool,
    volume_synced: bool,
    audio_fallback: bool,
}

impl MediaPlayer {
    /// Démarre la lecture dans un processus GStreamer isolé.
    pub fn start(url: &str) -> Result<Self, String> {
        let (audio, volume_supported) = audio_sink();
        // `QUARK_GST_AUDIO_SINK` est un choix explicite : on ne le remplace
        // jamais par le secours automatique.
        let audio_fallback =
            std::env::var_os("QUARK_GST_AUDIO_SINK").is_none() && audio == "pulsesink";
        Self::start_with_sink(url, &audio, volume_supported, audio_fallback, None)
    }

    fn start_with_sink(
        url: &str,
        audio: &str,
        volume_supported: bool,
        audio_fallback: bool,
        initial_volume: Option<f32>,
    ) -> Result<Self, String> {
        let binary = gst_launch().ok_or_else(|| {
            "GStreamer est absent : installez gstreamer1.0-tools, \
             gstreamer1.0-plugins-good et gstreamer1.0-libav."
                .to_string()
        })?;
        let args = playback_args(url, audio, initial_volume);
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

        let (volume_tx, volume_rx) = mpsc::channel();
        if volume_supported {
            let pid = child.id();
            std::thread::spawn(move || volume_worker(pid, volume_rx));
        }

        Ok(Self {
            child,
            rx,
            texture: None,
            state: PlaybackState::Starting,
            started: Instant::now(),
            url: url.to_string(),
            volume: 1.0,
            volume_tx: volume_supported.then_some(volume_tx),
            volume_supported,
            volume_synced: false,
            audio_fallback,
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
        let fallback = match &self.state {
            PlaybackState::Failed(message) => should_try_audio_fallback(
                self.audio_fallback,
                self.texture.is_some(),
                self.started.elapsed(),
                message,
            ),
            PlaybackState::Ended => should_try_audio_fallback(
                self.audio_fallback,
                self.texture.is_some(),
                self.started.elapsed(),
                "",
            ),
            _ => false,
        };
        if fallback {
            let error = match &self.state {
                PlaybackState::Failed(message) => message.clone(),
                _ => "flux vidéo interrompu".to_string(),
            };
            self.try_audio_fallback(&error);
        }
        if received_frame {
            if !self.volume_synced {
                self.send_volume();
                self.volume_synced = true;
            }
            ctx.request_repaint();
        }
        if let Some(delay) = poll_interval(&self.state) {
            ctx.request_repaint_after(delay);
        } else if let Some(delay) = fallback_watch_interval(
            &self.state,
            self.audio_fallback,
            self.texture.is_some(),
            self.started.elapsed(),
        ) {
            ctx.request_repaint_after(delay);
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
        self.audio_fallback = false;
        self.terminate();
        self.texture = None;
        self.state = PlaybackState::Ended;
    }

    /// Relance une fois le pipeline avec `autoaudiosink` quand `pulsesink` n'a
    /// pas ouvert le flux. GStreamer choisit alors PipeWire, ALSA ou une autre
    /// sortie disponible ; le volume n'est plus réglable via `pactl`, donc le
    /// curseur est masqué.
    fn try_audio_fallback(&mut self, error: &str) {
        self.audio_fallback = false;
        match Self::start_with_sink(&self.url, "autoaudiosink", false, false, Some(self.volume)) {
            Ok(mut player) => {
                player.volume = self.volume;
                let previous = std::mem::replace(self, player);
                drop(previous);
            }
            Err(message) => {
                self.state = PlaybackState::Failed(format!(
                    "{error} · secours audio impossible : {message}"
                ));
            }
        }
    }

    pub fn texture(&self) -> Option<&TextureHandle> {
        self.texture.as_ref()
    }

    pub fn state(&self) -> &PlaybackState {
        &self.state
    }

    pub fn volume_supported(&self) -> bool {
        self.volume_supported
    }

    /// Applique le volume au flux PulseAudio/PipeWire du décodeur, sans
    /// redémarrer la vidéo.
    pub fn set_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        if (volume - self.volume).abs() < f32::EPSILON {
            return;
        }
        self.volume = volume;
        self.send_volume();
    }

    fn send_volume(&self) {
        if let Some(tx) = &self.volume_tx {
            let _ = tx.send(self.volume);
        }
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
fn playback_args(url: &str, audio_sink: &str, volume: Option<f32>) -> Vec<String> {
    let sink = format!(
        "videoconvert ! videoscale ! video/x-raw,format=RGBA,width={WIDTH},height={HEIGHT} ! \
         fdsink fd=1 sync=true"
    );
    let mut args = vec![
        "-q".to_string(),
        "playbin".to_string(),
        format!("uri={url}"),
        format!("video-sink={sink}"),
        format!("audio-sink={audio_sink}"),
    ];
    // La sortie de secours n'a pas de contrôle de volume par `pactl` : on
    // applique le volume courant au démarrage du pipeline.
    if let Some(volume) = volume {
        args.push(format!("volume={volume:.2}"));
    }
    args
}

fn gst_launch() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("QUARK_GST_LAUNCH") {
        let path = PathBuf::from(path);
        return path.is_file().then_some(path);
    }
    command_in_path("gst-launch-1.0")
}

fn command_in_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
}

/// Choisit la sortie audio et indique si son volume peut être réglé en cours
/// de lecture. `pulsesink` fonctionne aussi sous PipeWire (pipewire-pulse), où
/// `pactl` retrouve le flux par le PID du décodeur. En cas d'échec de
/// `pulsesink` avant la première image, le lecteur retente avec
/// `autoaudiosink` (voir `try_audio_fallback`).
fn audio_sink() -> (String, bool) {
    let pulse = pulse_server_available() && command_in_path("pactl").is_some();
    if let Ok(sink) = std::env::var("QUARK_GST_AUDIO_SINK") {
        let supported = pulse && (sink == "autoaudiosink" || sink.contains("pulse"));
        return (sink, supported);
    }
    if pulse {
        ("pulsesink".to_string(), true)
    } else {
        ("autoaudiosink".to_string(), false)
    }
}

fn pulse_server_available() -> bool {
    std::env::var_os("PULSE_SERVER").is_some()
        || std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .is_some_and(|dir| dir.join("pulse/native").exists())
}

/// Applique les changements de volume dans un fil dédié : `pactl` ne doit
/// jamais bloquer le rendu. Les valeurs d'un glissement sont fusionnées.
fn volume_worker(pid: u32, rx: Receiver<f32>) {
    let mut sink_input = None;
    while let Ok(mut volume) = rx.recv() {
        std::thread::sleep(Duration::from_millis(50));
        while let Ok(next) = rx.try_recv() {
            volume = next;
        }
        let mut attempts = 0;
        loop {
            if let Some(index) = sink_input {
                if set_sink_input_volume(index, volume) {
                    break;
                }
                sink_input = None;
            }
            if let Some(index) = find_sink_input(pid) {
                sink_input = Some(index);
                continue;
            }
            attempts += 1;
            if attempts >= 6 {
                break;
            }
            std::thread::sleep(Duration::from_millis(150));
        }
    }
}

fn find_sink_input(pid: u32) -> Option<u32> {
    let output = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_sink_input(&String::from_utf8_lossy(&output.stdout), pid)
}

fn parse_sink_input(output: &str, pid: u32) -> Option<u32> {
    let needle = format!("application.process.id = \"{pid}\"");
    let mut current = None;
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Sink Input #") {
            current = rest.parse::<u32>().ok();
        } else if line == needle
            && let Some(index) = current
        {
            return Some(index);
        }
    }
    None
}

fn set_sink_input_volume(index: u32, volume: f32) -> bool {
    Command::new("pactl")
        .arg("set-sink-input-volume")
        .arg(index.to_string())
        .arg(format!("{}%", volume_percent(volume)))
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn volume_percent(volume: f32) -> u32 {
    (volume.clamp(0.0, 1.0) * 100.0).round() as u32
}

/// Intervalle de réveil de l'interface pendant la lecture. `None` quand rien
/// ne doit être repeint : pause, fin ou erreur.
fn poll_interval(state: &PlaybackState) -> Option<Duration> {
    match state {
        PlaybackState::Starting => Some(STARTUP_REPAINT),
        PlaybackState::Playing => Some(PLAYING_REPAINT),
        _ => None,
    }
}

/// Tant que le secours audio n'a pas été tenté, un `Ended` sans image peut
/// précéder de quelques millisecondes l'erreur `pulsesink` sur stderr. On
/// maintient donc un réveil court après la fin, borné dans le temps.
fn fallback_watch_interval(
    state: &PlaybackState,
    audio_fallback: bool,
    has_texture: bool,
    elapsed: Duration,
) -> Option<Duration> {
    (audio_fallback
        && !has_texture
        && matches!(state, PlaybackState::Ended)
        && elapsed < AUDIO_FALLBACK_WATCH)
        .then_some(STARTUP_REPAINT)
}

/// Le secours n'est tenté qu'avant la première image : redémarrer une vidéo
/// déjà commencée ferait perdre la position. Un échec précoce ou un message
/// qui désigne la sortie audio est considéré comme récupérable.
fn should_try_audio_fallback(
    audio_fallback: bool,
    has_texture: bool,
    elapsed: Duration,
    error: &str,
) -> bool {
    audio_fallback
        && !has_texture
        && (elapsed < Duration::from_secs(3) || looks_like_audio_failure(error))
}

fn looks_like_audio_failure(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    [
        "pulse",
        "alsa",
        "pipewire",
        "audio sink",
        "audiosink",
        "audio device",
    ]
    .iter()
    .any(|marker| message.contains(marker))
}

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

/// Ne remonte que les vraies erreurs de GStreamer, pas ses avertissements de
/// greffons optionnels.
fn gstreamer_error(stderr: &str) -> Option<String> {
    let errors: Vec<&str> = stderr
        .lines()
        .filter(|line| line.trim_start().starts_with("ERROR"))
        .collect();
    // GStreamer peut ajouter « pipeline doesn't want to preroll » après
    // l'erreur de l'élément audio : on préfère cette dernière pour que le
    // secours reconnaisse `pulsesink`.
    let line = errors
        .iter()
        .rev()
        .copied()
        .find(|line| looks_like_audio_failure(line))
        .or_else(|| errors.last().copied())?;
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
        let args = playback_args("https://video.test/master.m3u8", "autoaudiosink", None);
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
        assert!(!args.iter().any(|arg| arg.starts_with("volume=")));
        let fallback = playback_args("https://video.test/master.m3u8", "autoaudiosink", Some(0.4));
        assert!(fallback.contains(&"volume=0.40".to_string()));
    }

    #[test]
    fn le_secours_audio_est_limite_aux_echecs_precoces_de_pulsesink() {
        assert!(looks_like_audio_failure(
            "from element /GstPlayBin:playbin0/GstPulseSink:pulsesink0: Failed to connect: Connection refused"
        ));
        assert!(looks_like_audio_failure("no element \"pulsesink\""));
        assert!(!looks_like_audio_failure(
            "from element /GstPlayBin:playbin0/GstFdSink:fdsink0: broken pipe"
        ));
        assert!(should_try_audio_fallback(
            true,
            false,
            Duration::from_secs(1),
            "flux interrompu"
        ));
        assert!(!should_try_audio_fallback(
            false,
            false,
            Duration::from_secs(1),
            "pulse"
        ));
        assert!(!should_try_audio_fallback(
            true,
            true,
            Duration::from_secs(1),
            "pulse"
        ));
        assert!(!should_try_audio_fallback(
            true,
            false,
            Duration::from_secs(10),
            "flux interrompu"
        ));
        assert!(should_try_audio_fallback(
            true,
            false,
            Duration::from_secs(10),
            "pulsesink: Failed to connect"
        ));
        // gst-launch ajoute souvent une erreur générique après celle du sink.
        let stderr = "ERROR: from element /GstPlayBin:playbin0/GstPulseSink:pulsesink0: Failed to connect: Connection refused\n\
                      ERROR: pipeline doesn't want to preroll.";
        let error = gstreamer_error(stderr).expect("erreur audio");
        assert!(error.contains("pulsesink"), "{error}");
        assert!(should_try_audio_fallback(
            true,
            false,
            Duration::from_secs(4),
            &error
        ));
    }

    #[test]
    fn le_secours_audio_reste_a_l_ecoute_apres_la_fin() {
        assert_eq!(
            fallback_watch_interval(&PlaybackState::Ended, true, false, Duration::from_secs(1)),
            Some(STARTUP_REPAINT)
        );
        assert_eq!(
            fallback_watch_interval(&PlaybackState::Ended, true, false, Duration::from_secs(6)),
            None
        );
        assert_eq!(
            fallback_watch_interval(&PlaybackState::Ended, false, false, Duration::from_secs(1)),
            None
        );
        assert_eq!(
            fallback_watch_interval(&PlaybackState::Ended, true, true, Duration::from_secs(1)),
            None
        );
        assert_eq!(
            fallback_watch_interval(&PlaybackState::Playing, true, false, Duration::from_secs(1)),
            None
        );
    }

    #[test]
    fn la_lecture_se_repeint_sans_attendre_un_mouvement_de_souris() {
        assert_eq!(
            poll_interval(&PlaybackState::Playing),
            Some(PLAYING_REPAINT)
        );
        assert_eq!(
            poll_interval(&PlaybackState::Starting),
            Some(STARTUP_REPAINT)
        );
        assert_eq!(poll_interval(&PlaybackState::Paused), None);
        assert_eq!(poll_interval(&PlaybackState::Ended), None);
    }

    #[test]
    fn le_flux_audio_est_retrouve_par_le_pid_du_decodeur() {
        let output = r#"
Sink Input #41
	Driver: protocol-native.c
	Properties:
		application.name = "GStreamer"
		application.process.id = "999"
Sink Input #42
	Driver: protocol-native.c
	Properties:
		application.name = "GStreamer"
		application.process.id = "1234"
		media.role = "video"
"#;
        assert_eq!(parse_sink_input(output, 1234), Some(42));
        assert_eq!(parse_sink_input(output, 77), None);
        assert_eq!(volume_percent(0.5), 50);
        assert_eq!(volume_percent(1.0), 100);
        assert_eq!(volume_percent(-1.0), 0);
    }
}
