use async_std::channel::{Receiver, Sender};
use tracing::{callsite::Identifier, Metadata};
use tracing_subscriber::registry::{LookupSpan, SpanRef};

use crate::{Event, Records, SelfDuration, TotalDuration, TraceInner, TraceTree};

#[derive(Debug, Default, Clone)]
pub struct TracePath(Vec<Identifier>);

impl TracePath {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn from_span<'a, R>(span: SpanRef<'a, R>) -> Self
    where
        R: LookupSpan<'a>,
    {
        let from_root = span.from_root().chain(std::iter::once(span));
        TracePath(from_root.map(|span| span.metadata().callsite()).collect())
    }
}

impl IntoIterator for TracePath {
    type Item = Identifier;

    type IntoIter = std::vec::IntoIter<Identifier>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone)]
pub enum EventType {
    NewSpan {
        metadata: &'static Metadata<'static>,
        records: Records,
    },
    Record(Records),
    Event(Event),
    SelfDuration(SelfDuration),
    TotalDuration(TotalDuration),
}

#[derive(Debug, Clone)]
pub struct TraceEvent {
    path: TracePath,
    ty: EventType,
}

impl TraceEvent {
    pub fn new_span(
        path: TracePath,
        metadata: &'static Metadata<'static>,
        records: Records,
    ) -> Self {
        TraceEvent {
            path,
            ty: EventType::NewSpan { metadata, records },
        }
    }

    pub fn on_record(path: TracePath, records: Records) -> Self {
        TraceEvent {
            path,
            ty: EventType::Record(records),
        }
    }

    pub fn on_event(path: TracePath, event: Event) -> Self {
        TraceEvent {
            path,
            ty: EventType::Event(event),
        }
    }

    pub fn self_duration(path: TracePath, self_duration: SelfDuration) -> Self {
        TraceEvent {
            path,
            ty: EventType::SelfDuration(self_duration),
        }
    }

    pub fn total_duration(path: TracePath, total_duration: TotalDuration) -> Self {
        TraceEvent {
            path,
            ty: EventType::TotalDuration(total_duration),
        }
    }
}

#[derive(Debug)]
pub struct TraceRoot {
    sender: Sender<TraceEvent>,
    receiver: Receiver<TraceEvent>,
    children: TraceInner,
}

impl Default for TraceRoot {
    fn default() -> Self {
        let (sender, receiver) = async_std::channel::unbounded();
        TraceRoot {
            sender,
            receiver,
            children: Default::default(),
        }
    }
}

impl TraceRoot {
    pub fn sender(&self) -> Sender<TraceEvent> {
        self.sender.clone()
    }

    pub fn children(&mut self) -> &TraceInner {
        self.try_update();
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut TraceInner {
        &mut self.children
    }
}

impl TraceRoot {
    fn try_update(&mut self) {
        let mut events = vec![];
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }

        for event in events.into_iter() {
            let path = event.path;
            match event.ty {
                EventType::NewSpan { metadata, records } => self.new_span(path, metadata, records),
                EventType::Record(records) => self.on_record(path, records),
                EventType::Event(event) => self.on_event(path, event),
                EventType::SelfDuration(self_duration) => {
                    self.on_self_duration(path, self_duration)
                }
                EventType::TotalDuration(total_duration) => {
                    self.on_total_duration(path, total_duration)
                }
            }
        }
    }

    fn new_span(
        &mut self,
        path: TracePath,
        metadata: &'static Metadata<'static>,
        records: Records,
    ) {
        let mut iter = path.into_iter();
        let first = iter.next().unwrap();

        let mut inner = self.children.entry(first).or_insert_with(|| {
            let mut tree = TraceTree::new(true, metadata);
            tree.records_mut().join(records.clone());
            tree
        });

        for id in iter {
            inner = inner.children_mut().entry(id).or_insert_with(|| {
                let mut tree = TraceTree::new(false, metadata);
                tree.records_mut().join(records.clone());
                tree
            });
        }

        inner.clear_events();
    }

    fn on_record(&mut self, path: TracePath, records: Records) {
        let mut iter = path.into_iter();
        let first = iter.next().unwrap();
        let mut inner = self.children.get_mut(&first).unwrap();
        for id in iter {
            inner = inner.children_mut().get_mut(&id).unwrap();
        }
        inner.records_mut().join(records);
    }

    fn on_event(&mut self, path: TracePath, event: Event) {
        let mut iter = path.into_iter();
        let first = iter.next().unwrap();
        let mut inner = self.children.get_mut(&first).unwrap();
        for id in iter {
            inner = inner.children_mut().get_mut(&id).unwrap();
        }
        inner.event(event);
    }

    fn on_self_duration(&mut self, path: TracePath, self_duration: SelfDuration) {
        let mut iter = path.into_iter();
        let first = iter.next().unwrap();
        let mut inner = self.children.get_mut(&first).unwrap();
        for id in iter {
            inner = inner.children_mut().get_mut(&id).unwrap();
        }
        inner.push_self_duration(self_duration);
    }

    fn on_total_duration(&mut self, path: TracePath, total_duration: TotalDuration) {
        let mut iter = path.into_iter();
        let first = iter.next().unwrap();
        let mut inner = self.children.get_mut(&first).unwrap();
        for id in iter {
            inner = inner.children_mut().get_mut(&id).unwrap();
        }
        inner.push_total_duration(total_duration);
    }
}
