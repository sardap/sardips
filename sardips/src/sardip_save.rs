use std::{
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

use base64::{prelude::BASE64_STANDARD, Engine};
use bevy::prelude::*;
use bevy_http_client::{
    prelude::{HttpTypedRequestTrait, TypedRequest, TypedResponse},
    HttpClient,
};
use flate2::{write::ZlibEncoder, Compression};
use sardips_core::{persistent_id::PersistentIdGenerator, GameState};
use serde::{Deserialize, Serialize};
use shared_deps::moonshine_save::{prelude::*, GetStream};

#[cfg(target_arch = "wasm32")]
use flate2::read::ZlibDecoder;

#[cfg(target_arch = "wasm32")]
use bevy_http_client::{prelude::TypedResponseError, HttpResponse, HttpResponseError};

#[cfg(target_arch = "wasm32")]
use std::io::Read;

use crate::stock_market::{BuySellOrchestrator, OrderBook, QuarterManger};

pub struct SardipSavePlugin;

impl Plugin for SardipSavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(SardipLoadingState::default());

        app.insert_resource(SaveData::default());

        app.add_systems(
            PreUpdate,
            save_default()
                .include_resource::<OrderBook>()
                .include_resource::<QuarterManger>()
                .include_resource::<BuySellOrchestrator>()
                .include_resource::<PersistentIdGenerator>()
                .into(
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        file_from_resource::<SaveRequestFile>()
                    },
                    #[cfg(target_arch = "wasm32")]
                    {
                        shared_deps::moonshine_save::stream_from_resource::<SaveRequestWeb>()
                    },
                ),
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            app.add_systems(OnEnter(SardipLoadingState::Loading), trigger_load);
            app.add_systems(PreUpdate, load(file_from_resource::<LoadRequest>()));
            app.add_systems(
                Update,
                post_load.run_if(
                    in_state(SardipLoadingState::Loading)
                        .and_then(not(resource_exists::<LoadRequest>)),
                ),
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            app.add_systems(
                OnEnter(SardipLoadingState::Loading),
                trigger_load_save_remote,
            );
            app.add_systems(Update, handle_load_save_response);
            app.add_systems(
                PreUpdate,
                load(shared_deps::moonshine_save::stream_from_resource::<
                    LoadFromStream,
                >()),
            );
        }

        app.add_systems(Update, trigger_save.run_if(in_state(GameState::ViewScreen)))
            .add_systems(Update, handle_saved_response)
            .add_systems(
                PostUpdate,
                send_save_data.run_if(resource_removed::<SaveRequestWeb>()),
            );

        app.register_request_type::<SendSaveResponse>();
        #[cfg(target_arch = "wasm32")]
        app.register_request_type::<LoadResponse>();
    }
}

#[derive(Resource)]
struct SaveRequestFile;

impl GetFilePath for SaveRequestFile {
    fn path(&self) -> &Path {
        SAVE_PATH.as_ref()
    }
}

#[derive(Resource)]
struct SaveRequestWeb {
    pub buffer: WriteBuffer,
}

impl GetStream for SaveRequestWeb {
    type Stream = WriteBuffer;

    fn stream(&self) -> Self::Stream {
        self.buffer.clone()
    }
}

#[derive(Clone, Default)]
struct WriteBuffer {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Write for WriteBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let buffer = self.buffer.lock().unwrap();
        if buffer.is_empty() {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Buffer not empty",
            ))
        }
    }
}

#[derive(Resource, Default)]
struct SaveData {
    pub buffer: WriteBuffer,
}

#[derive(Resource)]
struct LoadRequest;

impl GetFilePath for LoadRequest {
    fn path(&self) -> &Path {
        SAVE_PATH.as_ref()
    }
}

struct SaveTimer {
    timer: Timer,
}

impl Default for SaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5., TimerMode::Repeating),
        }
    }
}

fn trigger_save(
    mut commands: Commands,
    time: Res<Time>,
    mut save_timer: Local<SaveTimer>,
    mut _save_data: ResMut<SaveData>,
) {
    if save_timer.timer.tick(time.delta()).just_finished() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            commands.insert_resource(SaveRequestFile);
        }

        #[cfg(target_arch = "wasm32")]
        {
            _save_data.buffer = WriteBuffer::default();
            commands.insert_resource(SaveRequestWeb {
                buffer: _save_data.buffer.clone(),
            });
        }
    }
}

fn trigger_load(mut commands: Commands) {
    commands.insert_resource(LoadRequest);
}

fn post_load(mut state: ResMut<NextState<SardipLoadingState>>) {
    state.set(SardipLoadingState::Loaded);
}

#[derive(Debug, States, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SardipLoadingState {
    #[default]
    None,
    Loading,
    Loaded,
    Failed,
}

const SAVE_PATH: &str = "sardip_save.ron";

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendSaveRequest {
    name: String,
    save_blob: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SendSaveResponse {}

fn send_save_data(
    mut save_request: EventWriter<TypedRequest<SendSaveResponse>>,
    save_data: Res<SaveData>,
) {
    let buffer = save_data.buffer.buffer.lock().unwrap();

    if !buffer.is_empty() {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::new(9));
        e.write_all(&buffer).unwrap();
        let compressed_data = e.finish().unwrap();
        let body: SendSaveRequest = SendSaveRequest {
            name: "sardips_save".to_string(),
            save_blob: BASE64_STANDARD.encode(compressed_data),
        };

        save_request.send(
            HttpClient::new()
                .post("/api/user/sardips/save")
                .json(&body)
                .with_type::<SendSaveResponse>(),
        );
    }
}

fn handle_saved_response(mut events: ResMut<Events<TypedResponse<SendSaveResponse>>>) {
    for _ in events.drain() {
        println!("Saved on remote");
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct LoadResponse {
    save_blob: String,
}

#[cfg(target_arch = "wasm32")]
fn trigger_load_save_remote(mut load_request: EventWriter<TypedRequest<LoadResponse>>) {
    info!("Loading save data from remote");
    load_request.send(
        HttpClient::new()
            .get("/api/user/sardips/save")
            .with_type::<LoadResponse>(),
    );
}

#[cfg(target_arch = "wasm32")]
#[derive(Resource)]
struct LoadFromStream {
    pub buffer: ReadBuffer,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct ReadBuffer {
    buffer: Arc<Mutex<Vec<u8>>>,
}

#[cfg(target_arch = "wasm32")]
impl std::io::Read for ReadBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.is_empty() {
            Ok(0)
        } else {
            let len = std::cmp::min(buf.len(), buffer.len());
            buf[..len].copy_from_slice(&buffer[..len]);
            buffer.drain(..len);
            Ok(len)
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl GetStream for LoadFromStream {
    type Stream = ReadBuffer;

    fn stream(&self) -> Self::Stream {
        self.buffer.clone()
    }
}

#[cfg(target_arch = "wasm32")]
fn handle_load_save_response(
    mut commands: Commands,
    mut events: ResMut<Events<TypedResponse<LoadResponse>>>,
    mut state: ResMut<NextState<SardipLoadingState>>,
) {
    for event in events.drain() {
        let response: LoadResponse = event.into_inner();
        if response.save_blob.len() > 0 {
            let decoded_data = BASE64_STANDARD.decode(&response.save_blob).unwrap();
            let mut e = ZlibDecoder::new(&decoded_data[..]);
            let mut decompressed_data = Vec::new();
            e.read_to_end(&mut decompressed_data).unwrap();

            commands.insert_resource(LoadFromStream {
                buffer: ReadBuffer {
                    buffer: Arc::new(Mutex::new(decompressed_data)),
                },
            });
            info!("Loaded save data from remote");
        } else {
            info!("No save data found on remote");
        }

        state.set(SardipLoadingState::Loaded);
    }
}
