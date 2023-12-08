//! `antigen` R&D sandbox.

use antigen_async::{parallel, serial, spawn};
use antigen_components::Label;
use antigen_egui::EguiUserInterface;
use antigen_rendering::{OnCpu, OnGpu};
use antigen_tracing::EnvLogTracer;
use database::MyTable;
use deebs::{Table, WriteSingleton};
use futures::StreamExt;
use integrator::*;
use log::LevelFilter;

use antigen_log::Logger;
use antigen_wgpu::{
    wgpu::{BackendBit, PowerPreference, RequestAdapterOptions},
    WgpuDevice, WgpuInstance, WgpuQueue, WgpuRenderer,
};
use antigen_winit::WinitEventLoopSystem;
use async_std::sync::Arc;
use std::{
    ops::Deref,
    time::{Duration, Instant},
};
use tracing::Instrument;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

pub async fn spin_sleep(duration: Duration) {
    let until = Instant::now() + duration;
    let mut wait = async_std::stream::repeat(());
    while wait.next().await.is_some() {
        if Instant::now() >= until {
            break;
        }
    }
}

const SEC_NANOS: u64 = 1000000000;
const TARGET_FRAME_RATE: u64 = 60;
const TIME_STEP: Duration = Duration::from_nanos(SEC_NANOS / TARGET_FRAME_RATE);

fn main() {
    let table = Arc::new(MyTable::default());

    tracing::subscriber::set_global_default(
        Registry::default()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(antigen_tracing::TraceExecution::new(table.clone()))
            .with(antigen_tracing::TraceSelfTime::new(table.clone()))
            .with(antigen_tracing::TraceTotalTime::new(table.clone())),
    )
    .expect("Failed to set tracing subscriber.");

    log::set_boxed_logger(Box::new(EnvLogTracer::new())).unwrap();
    log::set_max_level(LevelFilter::Trace);

    /*
    log::set_boxed_logger(Box::new(Logger::new(table.clone())))
        .map(|()| log::set_max_level(LOG_LEVEL))
        .expect("Failed to set logger.");
    */

    tracing::info_span!("main").in_scope(|| {
        // Perform initial assemblage
        spawn!(assemble(table.clone()));

        // Spawn game loop into its own task
        spawn!(game_loop(table.clone()));

        // Hand control of this thread off to winit
        WinitEventLoopSystem::run(table, main_events_cleared, redraw_events_cleared);
    });
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
async fn assemble(table: Arc<MyTable<'static>>) {
    // Setup wgpu
    let mut wgpu_instance = WriteSingleton::<WgpuInstance>::new(table.deref()).await;
    wgpu_instance.init(BackendBit::PRIMARY);
    let wgpu_instance = if let WgpuInstance::Ready(instance) = wgpu_instance.deref() {
        instance
    } else {
        panic!("wgpu instance is not ready.")
    };

    // Fetch physical device
    let adapter = wgpu_instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: None,
        })
        .instrument(tracing::info_span!("request_adapter"))
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(&Default::default(), None)
        .instrument(tracing::info_span!("request_device"))
        .await
        .expect("Failed to create device");

    let device_queue_key = table.next_key();
    serial!(
        table.insert(device_queue_key, Label::from("WgpuDevice / WgpuQueue")),
        table.insert(device_queue_key, WgpuDevice::from(device)),
        table.insert(device_queue_key, WgpuQueue::from(queue))
    );

    // Assemble crates
    parallel!(
        hello_triangle::assemble(table.clone()),
        hello_quads::assemble(table.clone()),
        hello_egui::assemble(table.clone()),
        debugger_egui::assemble::<database::DebugRow, _>(table.clone()),
        logger_egui::assemble(table.clone()),
        tracer_egui::assemble(table.clone()),
        integrator::assemble(table.clone()),
        database::stdout_debugger::assemble(table.clone())
    );
}

/// Game loop, runs in its own thread at a fixed tick rate
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
async fn game_loop(table: Arc<MyTable<'static>>) {
    loop {
        // Take a timestamp to mark the start of the game tick
        let start = Instant::now();

        // Run the game tick
        game_tick(table.clone()).await;

        // Taking elapsed time into account, spin sleep until the start of the next game tick
        spin_sleep(
            TIME_STEP
                .checked_sub(Instant::now() - start)
                .unwrap_or_default(),
        )
        .await;
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
async fn game_tick(table: Arc<MyTable<'static>>) {
    tracing::info!("game_tick");

    parallel!(
        {
            let table = table.clone();
            async move {
                serial!(
                    antigen_crossterm::run_events_system(table.clone()),
                    antigen_crossterm::run_key_events_system(table.clone())
                );
            }
        },
        IntegratorSystem::run(table.clone()),
        antigen_rendering::run_always_redraw_system::<OnCpu, _>(table.clone()),
    );
}

/// Main event callback, invoked when winit finishes processing main events
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
async fn main_events_cleared(table: Arc<MyTable<'static>>) {
    serial!(
        antigen_winit::run_close_window_system(table.clone()),
        antigen_winit_wgpu::drop_swap_chains(table.clone()),
        antigen_winit_wgpu::create_swap_chains(table.clone()),
        antigen_winit::run_window_event_system(table.clone()),
        antigen_egui::run_window_event_system(table.clone()),
        antigen_winit::run_redraw_system(table.clone())
    );
}

/// Main event callback, invoked when winit finishes processing redraw events
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
async fn redraw_events_cleared(table: Arc<MyTable<'static>>) {
    serial!(
        antigen_rendering::run_always_redraw_system::<OnGpu, _>(table.clone()),
        antigen_winit::run_redraw_flag_system(table.clone()),
        antigen_winit_wgpu::run_swap_chain_frame_system(table.clone()),
        antigen_wgpu::run_clear_command_buffers_system(table.clone())
    );

    parallel!(
        antigen_winit_wgpu::run_swap_chain_render_system::<EguiUserInterface<_, _>, _>(
            table.clone()
        ),
        antigen_winit_wgpu::run_swap_chain_render_system::<WgpuRenderer, _>(table.clone())
    );

    serial!(
        antigen_wgpu::run_flush_command_buffers_system(table.clone()),
        antigen_winit_wgpu::run_swap_chain_present_system(table.clone()),
    );
}
