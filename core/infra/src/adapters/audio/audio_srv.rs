use super::asset_provider::DefaultAudioAssetProvider;
use domain::{
    AudioAsset, AudioError, AudioLibrary, AudioService, PlaybackHandle,
    PlaybackRequest, Result,
};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

/// Commands sent to the dedicated audio thread
enum AudioCommand {
    Play(PlaybackRequest),
    Stop(String),
    StopAll,
    Pause(String),
    Resume(String),
    SetVolume(String, f32),
    GetActivePlaybacks,
    CleanupFinished,
    GetLibrary,
    PlayNotification(String, f32),
    PlayBackgroundAudio(String, f32),
    StopBackgroundAudio,
    AddAsset(AudioAsset),
    RemoveAsset(String),
    Shutdown,
}

/// Responses from the audio thread back to callers
enum AudioResponse {
    PlaybackHandle(std::result::Result<PlaybackHandle, AudioError>),
    Ok(std::result::Result<(), AudioError>),
    ActivePlaybacks(Vec<PlaybackHandle>),
    Library(AudioLibrary),
    RemovedAsset(Option<AudioAsset>),
}

/// Thread-safe wrapper that confines audio operations to a dedicated OS thread.
///
/// `OutputStream` from rodio is `!Send`, so all audio operations must happen
/// on the thread that created the stream. This struct sends commands to that
/// thread via channels and receives responses via `tokio::sync::oneshot`.
pub struct AudioThread {
    sender: std::sync::mpsc::Sender<(
        AudioCommand,
        tokio::sync::oneshot::Sender<AudioResponse>,
    )>,
    /// Handle to the dedicated audio thread, joined on drop
    _thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for AudioThread {
    fn drop(&mut self) {
        // Send shutdown command, ignoring errors if thread already exited
        let (tx, _rx) = tokio::sync::oneshot::channel();
        let _ = self.sender.send((AudioCommand::Shutdown, tx));
    }
}

impl AudioThread {
    /// Create a new `AudioThread` that spawns a dedicated OS thread for audio.
    pub fn new() -> std::result::Result<Self, AudioError> {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<(
            AudioCommand,
            tokio::sync::oneshot::Sender<AudioResponse>,
        )>();

        // We need to verify audio initialization works before returning.
        // Use a oneshot to get the result from the spawned thread.
        let (init_tx, init_rx) =
            std::sync::mpsc::channel::<std::result::Result<(), AudioError>>();

        let thread_handle = std::thread::Builder::new()
            .name("pomotoro-audio".to_string())
            .spawn(move || {
                let service = match RodioAudioService::new() {
                    Ok(s) => {
                        let _ = init_tx.send(Ok(()));
                        s
                    }
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };

                Self::run_audio_loop(service, cmd_rx);
            })
            .map_err(|e| {
                AudioError::PlaybackFailed(format!(
                    "Failed to spawn audio thread: {e}"
                ))
            })?;

        // Wait for initialization result
        let init_result = init_rx.recv().map_err(|_| {
            AudioError::PlaybackFailed(
                "Audio thread terminated during initialization".to_string(),
            )
        })?;

        init_result?;

        Ok(Self {
            sender: cmd_tx,
            _thread_handle: Some(thread_handle),
        })
    }

    /// Main loop running on the dedicated audio thread
    fn run_audio_loop(
        mut service: RodioAudioService,
        rx: std::sync::mpsc::Receiver<(
            AudioCommand,
            tokio::sync::oneshot::Sender<AudioResponse>,
        )>,
    ) {
        while let Ok((cmd, reply_tx)) = rx.recv() {
            let response = match cmd {
                AudioCommand::Play(request) => {
                    AudioResponse::PlaybackHandle(service.play(request))
                }
                AudioCommand::Stop(id) => {
                    AudioResponse::Ok(service.stop_playback(&id))
                }
                AudioCommand::StopAll => {
                    service.stop_all_playbacks();
                    AudioResponse::Ok(Ok(()))
                }
                AudioCommand::Pause(id) => {
                    AudioResponse::Ok(service.pause_playback(&id))
                }
                AudioCommand::Resume(id) => {
                    AudioResponse::Ok(service.resume_playback(&id))
                }
                AudioCommand::SetVolume(id, vol) => {
                    AudioResponse::Ok(service.set_audio_volume(&id, vol))
                }
                AudioCommand::GetActivePlaybacks => {
                    AudioResponse::ActivePlaybacks(
                        service.get_active_playbacks(),
                    )
                }
                AudioCommand::CleanupFinished => {
                    service.cleanup_finished_playbacks();
                    AudioResponse::Ok(Ok(()))
                }
                AudioCommand::GetLibrary => {
                    AudioResponse::Library(service.get_library().clone())
                }
                AudioCommand::PlayNotification(asset_id, volume) => {
                    AudioResponse::PlaybackHandle(
                        service.play_notification(&asset_id, volume),
                    )
                }
                AudioCommand::PlayBackgroundAudio(asset_id, volume) => {
                    AudioResponse::PlaybackHandle(
                        service.play_background_audio(&asset_id, volume),
                    )
                }
                AudioCommand::StopBackgroundAudio => {
                    service.stop_background_audio();
                    AudioResponse::Ok(Ok(()))
                }
                AudioCommand::AddAsset(asset) => {
                    service.add_asset(asset);
                    AudioResponse::Ok(Ok(()))
                }
                AudioCommand::RemoveAsset(id) => {
                    AudioResponse::RemovedAsset(service.remove_asset(&id))
                }
                AudioCommand::Shutdown => break,
            };
            // Ignore send errors: caller may have dropped the receiver
            let _ = reply_tx.send(response);
        }
    }

    /// Send a command and wait for the response
    fn send_command(
        &self,
        cmd: AudioCommand,
    ) -> std::result::Result<AudioResponse, AudioError> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.sender.send((cmd, reply_tx)).map_err(|_| {
            AudioError::PlaybackFailed(
                "Audio thread has terminated".to_string(),
            )
        })?;
        reply_rx.blocking_recv().map_err(|_| {
            AudioError::PlaybackFailed(
                "Audio thread did not respond".to_string(),
            )
        })
    }
}

impl AudioService for AudioThread {
    fn play_audio(
        &mut self,
        request: PlaybackRequest,
    ) -> Result<PlaybackHandle> {
        match self.send_command(AudioCommand::Play(request))? {
            AudioResponse::PlaybackHandle(result) => {
                result.map_err(|e| e.into())
            }
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn stop_audio(&mut self, playback_id: &str) -> Result<()> {
        match self.send_command(AudioCommand::Stop(playback_id.to_string()))? {
            AudioResponse::Ok(result) => result.map_err(|e| e.into()),
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn stop_all_audio(&mut self) -> Result<()> {
        match self.send_command(AudioCommand::StopAll)? {
            AudioResponse::Ok(result) => result.map_err(|e| e.into()),
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn pause_audio(&mut self, playback_id: &str) -> Result<()> {
        match self.send_command(AudioCommand::Pause(playback_id.to_string()))? {
            AudioResponse::Ok(result) => result.map_err(|e| e.into()),
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn resume_audio(&mut self, playback_id: &str) -> Result<()> {
        match self
            .send_command(AudioCommand::Resume(playback_id.to_string()))?
        {
            AudioResponse::Ok(result) => result.map_err(|e| e.into()),
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn set_volume(&mut self, playback_id: &str, volume: f32) -> Result<()> {
        match self.send_command(AudioCommand::SetVolume(
            playback_id.to_string(),
            volume,
        ))? {
            AudioResponse::Ok(result) => result.map_err(|e| e.into()),
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn get_active_playbacks(&self) -> Result<Vec<PlaybackHandle>> {
        // get_active_playbacks takes &self, but send_command also takes &self,
        // so we need a separate path. Since AudioThread is Send+Sync by design,
        // we use the same channel approach.
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.sender
            .send((AudioCommand::GetActivePlaybacks, reply_tx))
            .map_err(|_| domain::Error::AudioError {
                message: "Audio thread has terminated".to_string(),
            })?;
        match reply_rx.blocking_recv().map_err(|_| {
            domain::Error::AudioError {
                message: "Audio thread did not respond".to_string(),
            }
        })? {
            AudioResponse::ActivePlaybacks(playbacks) => Ok(playbacks),
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn cleanup_finished(&mut self) -> Result<()> {
        match self.send_command(AudioCommand::CleanupFinished)? {
            AudioResponse::Ok(result) => result.map_err(|e| e.into()),
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn get_library(&self) -> &AudioLibrary {
        // This is tricky: the trait requires returning a reference, but we
        // communicate via channels. We cannot return a reference to data
        // owned by the audio thread. For now, we use a workaround with a
        // cached library that is only valid as a snapshot.
        //
        // Since the AudioServiceWrapper already clones the library (it calls
        // `get_library().clone()`), and the AudioThread is used through
        // AudioServiceWrapper, this method should not be called directly.
        // We provide a minimal implementation that returns a static empty library.
        //
        // NOTE: The actual library access goes through get_library_snapshot()
        // which the AudioServiceWrapper uses.
        static EMPTY_LIBRARY: std::sync::LazyLock<AudioLibrary> =
            std::sync::LazyLock::new(AudioLibrary::new);
        &EMPTY_LIBRARY
    }

    fn play_notification(
        &mut self,
        asset_id: &str,
        volume: f32,
    ) -> Result<PlaybackHandle> {
        match self.send_command(AudioCommand::PlayNotification(
            asset_id.to_string(),
            volume,
        ))? {
            AudioResponse::PlaybackHandle(result) => {
                result.map_err(|e| e.into())
            }
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn play_background_audio(
        &mut self,
        asset_id: &str,
        volume: f32,
    ) -> Result<PlaybackHandle> {
        match self.send_command(AudioCommand::PlayBackgroundAudio(
            asset_id.to_string(),
            volume,
        ))? {
            AudioResponse::PlaybackHandle(result) => {
                result.map_err(|e| e.into())
            }
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn stop_background_audio(&mut self) -> Result<()> {
        match self.send_command(AudioCommand::StopBackgroundAudio)? {
            AudioResponse::Ok(result) => result.map_err(|e| e.into()),
            _ => unreachable!("unexpected response variant"),
        }
    }

    fn add_asset(&mut self, asset: AudioAsset) {
        let _ = self.send_command(AudioCommand::AddAsset(asset));
    }

    fn remove_asset(&mut self, asset_id: &str) -> Option<AudioAsset> {
        match self.send_command(AudioCommand::RemoveAsset(asset_id.to_string()))
        {
            Ok(AudioResponse::RemovedAsset(result)) => result,
            _ => None,
        }
    }
}

impl AudioThread {
    /// Get a snapshot of the audio library via the channel.
    /// This is used by `AudioServiceWrapper` which needs the full library.
    pub fn get_library_snapshot(&self) -> AudioLibrary {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        if self
            .sender
            .send((AudioCommand::GetLibrary, reply_tx))
            .is_err()
        {
            return AudioLibrary::new();
        }
        match reply_rx.blocking_recv() {
            Ok(AudioResponse::Library(lib)) => lib,
            _ => AudioLibrary::new(),
        }
    }
}

// ---- Private implementation detail: RodioAudioService ----
// This struct is NOT Send or Sync because OutputStream is !Send.
// It lives entirely on the dedicated audio thread.

struct RodioAudioService {
    stream_handle: OutputStream,
    library: AudioLibrary,
    active_playbacks: HashMap<String, AudioPlayback>,
}

struct AudioPlayback {
    sink: Sink,
    handle: PlaybackHandle,
}

impl RodioAudioService {
    fn new() -> std::result::Result<Self, AudioError> {
        let stream_handle = OutputStreamBuilder::open_default_stream()
            .map_err(|e| {
                AudioError::PlaybackFailed(format!(
                    "Failed to create audio stream: {e}"
                ))
            })?;

        Ok(Self {
            stream_handle,
            library:
                DefaultAudioAssetProvider::create_library_with_default_assets(),
            active_playbacks: HashMap::new(),
        })
    }

    /// Resolve relative paths to absolute paths based on the current directory
    /// or the executable's directory
    fn resolve_audio_path(path: &Path) -> PathBuf {
        if path.is_absolute() {
            return path.to_path_buf();
        }

        let from_cwd = std::env::current_dir().ok().map(|cwd| cwd.join(path));

        if let Some(ref p) = from_cwd {
            if p.exists() {
                return p.clone();
            }
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let mut current = exe_dir;

                for _ in 0..5 {
                    let candidate = current.join(path);
                    if candidate.exists() {
                        return candidate;
                    }

                    if let Some(parent) = current.parent() {
                        let parent_candidate = parent.join(path);
                        if parent_candidate.exists() {
                            return parent_candidate;
                        }
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }

        path.to_path_buf()
    }

    fn get_library(&self) -> &AudioLibrary {
        &self.library
    }

    fn add_asset(&mut self, asset: AudioAsset) {
        self.library.add_asset(asset);
    }

    fn remove_asset(&mut self, asset_id: &str) -> Option<AudioAsset> {
        self.stop_playback(asset_id).ok();
        self.library.remove_asset(asset_id)
    }

    fn play(
        &mut self,
        request: PlaybackRequest,
    ) -> std::result::Result<PlaybackHandle, AudioError> {
        let asset = self
            .library
            .get_asset(&request.asset_id)
            .cloned()
            .ok_or_else(|| {
                AudioError::AssetNotFound(request.asset_id.clone())
            })?;

        if asset.file_path.to_string_lossy().starts_with("embedded:") {
            return self.play_embedded_sound(request, &asset);
        }

        let resolved_path = Self::resolve_audio_path(&asset.file_path);

        let file = File::open(&resolved_path).map_err(|e| {
            eprintln!(
                "Warning: Audio file not found: {} (resolved to: {}). Error: {}",
                asset.file_path.to_string_lossy(),
                resolved_path.to_string_lossy(),
                e
            );
            AudioError::InvalidFile(
                asset.file_path.to_string_lossy().to_string(),
            )
        })?;

        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|e| {
            AudioError::PlaybackFailed(format!("Failed to decode audio: {e}"))
        })?;

        let sink = Sink::connect_new(self.stream_handle.mixer());

        if let Some(fade_in_ms) = request.fade_in_ms {
            let source: Box<dyn Source<Item = f32> + Send> = if request.looped {
                Box::new(
                    decoder
                        .repeat_infinite()
                        .amplify(request.volume)
                        .fade_in(Duration::from_millis(fade_in_ms as u64)),
                )
            } else {
                Box::new(
                    decoder
                        .amplify(request.volume)
                        .fade_in(Duration::from_millis(fade_in_ms as u64)),
                )
            };
            sink.append(source);
        } else {
            let source: Box<dyn Source<Item = f32> + Send> = if request.looped {
                Box::new(decoder.repeat_infinite().amplify(request.volume))
            } else {
                Box::new(decoder.amplify(request.volume))
            };
            sink.append(source);
        }

        let handle_id = Uuid::new_v4().to_string();
        let handle = PlaybackHandle {
            id: handle_id.clone(),
            asset_id: request.asset_id.clone(),
            is_playing: true,
            is_looped: request.looped,
            volume: request.volume,
        };

        let playback = AudioPlayback {
            sink,
            handle: handle.clone(),
        };

        self.active_playbacks.insert(handle_id, playback);

        Ok(handle)
    }

    fn stop_playback(
        &mut self,
        handle_id: &str,
    ) -> std::result::Result<(), AudioError> {
        if let Some(playback) = self.active_playbacks.remove(handle_id) {
            playback.sink.stop();
            return Ok(());
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    fn pause_playback(
        &self,
        handle_id: &str,
    ) -> std::result::Result<(), AudioError> {
        if let Some(playback) = self.active_playbacks.get(handle_id) {
            playback.sink.pause();
            return Ok(());
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    fn resume_playback(
        &self,
        handle_id: &str,
    ) -> std::result::Result<(), AudioError> {
        if let Some(playback) = self.active_playbacks.get(handle_id) {
            playback.sink.play();
            return Ok(());
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    fn set_audio_volume(
        &self,
        handle_id: &str,
        volume: f32,
    ) -> std::result::Result<(), AudioError> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(AudioError::VolumeOutOfRange(volume));
        }

        if let Some(playback) = self.active_playbacks.get(handle_id) {
            playback.sink.set_volume(volume);
            return Ok(());
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    fn get_active_playbacks(&self) -> Vec<PlaybackHandle> {
        self.active_playbacks
            .values()
            .map(|playback| playback.handle.clone())
            .collect()
    }

    fn stop_all_playbacks(&mut self) {
        for (_, playback) in self.active_playbacks.drain() {
            playback.sink.stop();
        }
    }

    fn play_notification(
        &mut self,
        asset_id: &str,
        volume: f32,
    ) -> std::result::Result<PlaybackHandle, AudioError> {
        let request = PlaybackRequest::new(asset_id.to_string(), volume)?;
        self.play(request)
    }

    fn play_background_audio(
        &mut self,
        asset_id: &str,
        volume: f32,
    ) -> std::result::Result<PlaybackHandle, AudioError> {
        let request = PlaybackRequest::new(asset_id.to_string(), volume)?
            .with_loop()
            .with_fade_in(2000);
        self.play(request)
    }

    fn stop_background_audio(&mut self) {
        let handles: Vec<String> = self
            .active_playbacks
            .values()
            .filter(|playback| playback.handle.is_looped)
            .map(|playback| playback.handle.id.clone())
            .collect();

        for handle_id in handles {
            self.stop_playback(&handle_id).ok();
        }
    }

    fn cleanup_finished_playbacks(&mut self) {
        let finished_handles: Vec<String> = self
            .active_playbacks
            .iter()
            .filter(|(_, playback)| playback.sink.empty())
            .map(|(handle_id, _)| handle_id.clone())
            .collect();

        for handle_id in finished_handles {
            self.active_playbacks.remove(&handle_id);
        }
    }

    fn play_embedded_sound(
        &mut self,
        request: PlaybackRequest,
        _asset: &AudioAsset,
    ) -> std::result::Result<PlaybackHandle, AudioError> {
        let sink = Sink::connect_new(self.stream_handle.mixer());

        let silent_samples: Vec<f32> = vec![0.0; 44100 / 10];
        let source =
            rodio::buffer::SamplesBuffer::new(1, 44100, silent_samples);

        sink.append(source);
        sink.set_volume(0.0);

        let handle_id = Uuid::new_v4().to_string();
        let handle = PlaybackHandle {
            id: handle_id.clone(),
            asset_id: request.asset_id.clone(),
            is_playing: true,
            is_looped: false,
            volume: 0.0,
        };

        let playback = AudioPlayback {
            sink,
            handle: handle.clone(),
        };

        self.active_playbacks.insert(handle_id, playback);

        Ok(handle)
    }
}
