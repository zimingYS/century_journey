use std::path::PathBuf;
use std::sync::atomic::Ordering;

use bevy::diagnostic::{
    DiagnosticsStore, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};
use serde::Serialize;

use crate::app::flow::PendingWorld;
use crate::client::renderer::world::MeshBuildChannel;
use crate::content::block::registry::BlockRegistry;
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::world::chunk::ChunkState;
use crate::game::world::save::level;
use crate::game::world::storage::WorldStorage;
use crate::game::world::systems::{
    PlayerChunkCache, StructureGenChannel, TerrainGenChannel, WorldStreamingConfig,
};
use crate::shared::states::AppState;
use crate::shared::time::NEW_WORLD_START_TIME;

const SCENARIO_NAME: &str = "survival_spawn_v1";
const WORLD_NAME: &str = "__perf_survival_spawn_v1";
const WORLD_SEED: u64 = 0xC317_2026;
const WARMUP_SECONDS: f32 = 5.0;
const SAMPLE_SECONDS: f32 = 15.0;
const RENDER_DISTANCE: u32 = 8;

#[derive(Resource)]
struct FixedPerformanceScenario {
    output: PathBuf,
    world_requested: bool,
    configured: bool,
    elapsed_seconds: f32,
    frame_times_ms: Vec<f64>,
    task_samples: Vec<ChunkTaskSample>,
    memory_samples_gib: Vec<f64>,
    written: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct ChunkTaskSample {
    terrain: usize,
    structure: usize,
    mesh: usize,
}

impl ChunkTaskSample {
    fn total(self) -> usize {
        self.terrain + self.structure + self.mesh
    }
}

#[derive(Serialize)]
struct PerformanceReport {
    scenario: &'static str,
    world_seed: u64,
    resolution: &'static str,
    render_distance: u32,
    warmup_seconds: f32,
    sample_seconds: f32,
    samples: usize,
    frame_time_ms: MetricSummary,
    chunk_tasks: ChunkTaskSummary,
    process_memory_gib: OptionalMetricSummary,
    chunks: ChunkSummary,
}

#[derive(Serialize)]
struct MetricSummary {
    mean: f64,
    p95: f64,
    max: f64,
}

#[derive(Serialize)]
struct OptionalMetricSummary {
    last: Option<f64>,
    max: Option<f64>,
}

#[derive(Serialize)]
struct ChunkTaskSummary {
    mean_total: f64,
    max_total: usize,
    max_terrain: usize,
    max_structure: usize,
    max_mesh: usize,
    last: ChunkTaskSample,
}

#[derive(Serialize)]
struct ChunkSummary {
    expected: usize,
    loaded: usize,
    rendered: usize,
}

pub fn configure_fixed_performance_scenario(app: &mut App) {
    let Ok(scenario) = std::env::var("CJ_PERF_SCENARIO") else {
        return;
    };
    if scenario != SCENARIO_NAME {
        warn!("Unknown performance scenario: {scenario}");
        return;
    }

    let output = std::env::var_os("CJ_PERF_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/perf/survival_spawn_v1.json"));
    app.add_plugins((
        FrameTimeDiagnosticsPlugin::new(1200),
        SystemInformationDiagnosticsPlugin,
    ))
    .insert_resource(FixedPerformanceScenario {
        output,
        world_requested: false,
        configured: false,
        elapsed_seconds: 0.0,
        frame_times_ms: Vec::with_capacity(2000),
        task_samples: Vec::with_capacity(2000),
        memory_samples_gib: Vec::with_capacity(2000),
        written: false,
    })
    .add_systems(Update, fixed_performance_scenario_system);
}

#[derive(SystemParam)]
struct PerformanceRuntimeParams<'w, 's> {
    pending_world: ResMut<'w, PendingWorld>,
    next_state: ResMut<'w, NextState<AppState>>,
    gamemode: ResMut<'w, PlayerGameMode>,
    streaming: ResMut<'w, WorldStreamingConfig>,
    terrain_channel: Res<'w, TerrainGenChannel>,
    structure_channel: Res<'w, StructureGenChannel>,
    mesh_channel: Res<'w, MeshBuildChannel>,
    diagnostics: Res<'w, DiagnosticsStore>,
    player_cache: Res<'w, PlayerChunkCache>,
    world_storage: Res<'w, WorldStorage>,
    chunk_states: Query<'w, 's, &'static ChunkState>,
    window_query: Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
    app_exit: MessageWriter<'w, AppExit>,
}

fn fixed_performance_scenario_system(
    real_time: Res<Time<Real>>,
    app_state: Res<State<AppState>>,
    config: Option<ResMut<FixedPerformanceScenario>>,
    block_registry: Option<Res<BlockRegistry>>,
    mut params: PerformanceRuntimeParams,
) {
    let Some(mut config) = config else {
        return;
    };
    if config.written {
        return;
    }

    if *app_state.get() == AppState::MainMenu && !config.world_requested {
        let Some(block_registry) = block_registry else {
            return;
        };
        if !level::world_exists(WORLD_NAME)
            && let Err(error) = level::save_level(
                WORLD_NAME,
                WORLD_SEED,
                crate::game::world::generation::pipeline::CURRENT_GENERATION_VERSION,
                Vec3::new(0.0, 70.0, 0.0),
                NEW_WORLD_START_TIME,
                &block_registry,
            )
        {
            error!("Failed to create performance world: {error}");
            config.written = true;
            params.app_exit.write(AppExit::error());
            return;
        }
        params.pending_world.0 = Some(WORLD_NAME.into());
        config.world_requested = true;
        params.next_state.set(AppState::WorldLoading);
        return;
    }

    if *app_state.get() != AppState::InGame {
        return;
    }
    if !config.configured {
        params.gamemode.mode = GameMode::Survival;
        *params.streaming = WorldStreamingConfig::new(RENDER_DISTANCE, RENDER_DISTANCE, 2, 3);
        if let Ok(mut window) = params.window_query.single_mut() {
            window.present_mode = PresentMode::AutoNoVsync;
        }
        config.configured = true;
        info!("[perf:{SCENARIO_NAME}] warmup={WARMUP_SECONDS}s sample={SAMPLE_SECONDS}s");
    }

    config.elapsed_seconds += real_time.delta_secs();
    if config.elapsed_seconds < WARMUP_SECONDS {
        return;
    }

    if let Some(frame_ms) = params
        .diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diagnostic| diagnostic.value())
        .filter(|value| value.is_finite())
    {
        config.frame_times_ms.push(frame_ms);
    }
    let task_sample = ChunkTaskSample {
        terrain: params.terrain_channel.in_flight.load(Ordering::Relaxed),
        structure: params.structure_channel.in_flight.load(Ordering::Relaxed),
        mesh: params.mesh_channel.in_flight.load(Ordering::Relaxed),
    };
    config.task_samples.push(task_sample);
    if let Some(memory_gib) = params
        .diagnostics
        .get(&SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE)
        .and_then(|diagnostic| diagnostic.value())
        .filter(|value| value.is_finite())
    {
        config.memory_samples_gib.push(memory_gib);
    }

    if config.elapsed_seconds < WARMUP_SECONDS + SAMPLE_SECONDS {
        return;
    }

    let report = PerformanceReport {
        scenario: SCENARIO_NAME,
        world_seed: WORLD_SEED,
        resolution: "1280x720",
        render_distance: RENDER_DISTANCE,
        warmup_seconds: WARMUP_SECONDS,
        sample_seconds: SAMPLE_SECONDS,
        samples: config.frame_times_ms.len(),
        frame_time_ms: summarize(&config.frame_times_ms),
        chunk_tasks: summarize_tasks(&config.task_samples),
        process_memory_gib: OptionalMetricSummary {
            last: config.memory_samples_gib.last().copied(),
            max: config.memory_samples_gib.iter().copied().reduce(f64::max),
        },
        chunks: ChunkSummary {
            expected: params.player_cache.expected_chunks.len(),
            loaded: params.world_storage.loaded_chunks.len(),
            rendered: params
                .chunk_states
                .iter()
                .filter(|state| **state == ChunkState::Rendered)
                .count(),
        },
    };
    match write_report(&config.output, &report) {
        Ok(()) => info!("[perf:{SCENARIO_NAME}] report={}", config.output.display()),
        Err(error) => error!("Failed to write performance report: {error}"),
    }
    config.written = true;
    params.app_exit.write(AppExit::Success);
}

fn summarize(values: &[f64]) -> MetricSummary {
    if values.is_empty() {
        return MetricSummary {
            mean: 0.0,
            p95: 0.0,
            max: 0.0,
        };
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let p95_index = ((sorted.len() as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    MetricSummary {
        mean: sorted.iter().sum::<f64>() / sorted.len() as f64,
        p95: sorted[p95_index],
        max: *sorted.last().unwrap_or(&0.0),
    }
}

fn summarize_tasks(samples: &[ChunkTaskSample]) -> ChunkTaskSummary {
    let last = samples.last().copied().unwrap_or(ChunkTaskSample {
        terrain: 0,
        structure: 0,
        mesh: 0,
    });
    let mean_total = if samples.is_empty() {
        0.0
    } else {
        samples.iter().map(|sample| sample.total()).sum::<usize>() as f64 / samples.len() as f64
    };
    ChunkTaskSummary {
        mean_total,
        max_total: samples
            .iter()
            .map(|sample| sample.total())
            .max()
            .unwrap_or(0),
        max_terrain: samples
            .iter()
            .map(|sample| sample.terrain)
            .max()
            .unwrap_or(0),
        max_structure: samples
            .iter()
            .map(|sample| sample.structure)
            .max()
            .unwrap_or(0),
        max_mesh: samples.iter().map(|sample| sample.mesh).max().unwrap_or(0),
        last,
    }
}

fn write_report(path: &PathBuf, report: &PerformanceReport) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(report).map_err(|error| error.to_string())?;
    std::fs::write(path, bytes).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_summary_uses_a_stable_nearest_rank_p95() {
        let values: Vec<_> = (1..=100).map(f64::from).collect();
        let summary = summarize(&values);

        assert_eq!(summary.mean, 50.5);
        assert_eq!(summary.p95, 95.0);
        assert_eq!(summary.max, 100.0);
    }
}
