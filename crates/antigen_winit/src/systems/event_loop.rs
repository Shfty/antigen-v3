use std::ops::Deref;

pub use winit;

use async_std::sync::Arc;
use deebs::{BorrowColumn, BorrowSingleton, Table, WriteSingleton};
use futures::Future;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    CreateWindowSystem, WinitMainEvent, WinitMainEvents, WinitRedrawEvent, WinitRedrawEvents,
    WinitWindow, WinitWindowEvent, WinitWindows,
};

#[derive(Debug, Default, Clone)]
pub struct WinitEventLoopSystem {
    state: State,
    main_events: Vec<WinitMainEvent>,
    redraw_events: Vec<WinitRedrawEvent>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum State {
    Waiting,
    MainEvents,
    RedrawEvents,
}

impl Default for State {
    fn default() -> Self {
        State::Waiting
    }
}

impl WinitEventLoopSystem {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip(table, main_event_callback, redraw_event_callback))
    )]
    pub fn run<T, MF, RF, MFut, RFut>(
        table: Arc<T>,
        main_event_callback: MF,
        redraw_event_callback: RF,
    ) -> !
    where
        T: Table
            + BorrowSingleton<WinitWindows>
            + BorrowSingleton<WinitMainEvents>
            + BorrowSingleton<WinitRedrawEvents>
            + BorrowColumn<WinitWindow>
            + Sync
            + 'static,
        MF: Fn(Arc<T>) -> MFut + 'static,
        RF: Fn(Arc<T>) -> RFut + 'static,
        MFut: Future<Output = ()> + Send + 'static,
        RFut: Future<Output = ()> + Send + 'static,
    {
        let mut winit = Self::default();
        EventLoop::new().run(move |event, window_target, control_flow| {
            *control_flow = ControlFlow::Poll;

            let winit_event = Into::<WinitMainEvent>::into(&event);

            match winit.state {
                State::Waiting => {
                    if let Event::NewEvents(_) = event {
                        winit.state = State::MainEvents;
                    } else {
                        panic!("Unexpected event in Waiting state: {:?}", event);
                    }
                }
                State::MainEvents => {
                    if let Event::MainEventsCleared = event {
                        // Create any pending windows
                        let created_windows = async_std::task::block_on(CreateWindowSystem::run(
                            table.clone(),
                            window_target,
                        ))
                        .into_iter()
                        .map(|window_id| WinitMainEvent::WindowEvent {
                            window_id,
                            event: WinitWindowEvent::Opened,
                        });

                        // Write new events to singleton
                        {
                            let mut events =
                                async_std::task::block_on(WriteSingleton::<WinitMainEvents>::new(
                                    table.deref(),
                                ));

                            events.extend(created_windows.chain(winit.main_events.drain(..)));
                        }

                        // If present, invoke callback
                        async_std::task::block_on((main_event_callback)(table.clone()));

                        // Write new events to singleton
                        {
                            let mut events =
                                async_std::task::block_on(WriteSingleton::<WinitMainEvents>::new(
                                    table.deref(),
                                ));

                            events.clear();
                        }

                        // Proceed to the next state
                        winit.state = State::RedrawEvents;
                    } else if let Event::WindowEvent { window_id, event } = event {
                        if let WindowEvent::CloseRequested = event {
                            let table = table.clone();
                            async_std::task::block_on(async move {
                                let mut windows =
                                    WriteSingleton::<WinitWindows>::new(table.deref()).await;
                                windows.remove(&window_id).expect("Invalid Window ID.");
                            });
                        }
                        winit.main_events.push(winit_event);
                    } else {
                        winit.main_events.push(winit_event);
                    }
                }
                State::RedrawEvents => match event {
                    Event::RedrawEventsCleared => {
                        // Write new events to singleton
                        {
                            let mut events =
                                async_std::task::block_on(
                                    WriteSingleton::<WinitRedrawEvents>::new(table.deref()),
                                );
                            events.extend(winit.redraw_events.drain(..));
                        }

                        // Invoke callback
                        async_std::task::block_on((redraw_event_callback)(table.clone()));

                        // Clear events
                        {
                            let mut events =
                                async_std::task::block_on(
                                    WriteSingleton::<WinitRedrawEvents>::new(table.deref()),
                                );
                            events.clear();
                        }

                        // Proceed to the next state
                        winit.state = State::Waiting;
                    }
                    Event::RedrawRequested(window_id) => {
                        winit.redraw_events.push(window_id.into());
                    }
                    e => panic!("Unexpected event type in RedrawEvents state: {:?}", e),
                },
            }
        });
    }
}
