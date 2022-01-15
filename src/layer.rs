use fmt::Display;
use std::{
    fmt,
    io::{self, Write},
};
use tracing::{
    event::Event,
    field::Field,
    field::Visit,
    span::{Attributes, Id, Record},
    Metadata, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan};

use crate::android::{AndroidWriter, CappedTag};

pub struct Layer {
    tag: CappedTag,
}

impl Layer {
    pub fn new(name: &str) -> io::Result<Self> {
        let tag = CappedTag::new(name.as_bytes())?;
        Ok(Self { tag })
    }
}

impl<S> tracing_subscriber::Layer<S> for Layer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<S>) {
        let span = ctx.span(id).expect("unknown span");
        let mut buf = Vec::with_capacity(256);

        let depth = span.parent().into_iter().flat_map(|x| x.scope()).count();

        write!(buf, "s{}_name: ", depth).unwrap();
        write_value(&mut buf, span.name());
        put_metadata(&mut buf, span.metadata(), Some(depth));

        attrs.record(&mut SpanVisitor::new(&mut buf, depth));

        span.extensions_mut().insert(SpanFields(buf));
    }

    fn on_record(&self, id: &Id, values: &Record, ctx: Context<S>) {
        let span = ctx.span(id).expect("unknown span");
        let depth = span.parent().into_iter().flat_map(|x| x.scope()).count();
        let mut exts = span.extensions_mut();
        let buf = &mut exts.get_mut::<SpanFields>().expect("missing fields").0;
        values.record(&mut SpanVisitor::new(buf, depth));
    }

    fn on_event(&self, event: &Event, ctx: Context<S>) {
        let mut writer = AndroidWriter::new(event.metadata().level(), &self.tag); //PlatformLogWriter::new(event.metadata().level(), &self.tag);

        // add the target
        let _ = write!(&mut writer, "{}: ", event.metadata().target());

        // Record span fields
        let maybe_scope = ctx
            .current_span()
            .id()
            .and_then(|id| ctx.span_scope(id).map(|x| x.from_root()));
        if let Some(scope) = maybe_scope {
            for span in scope {
                let exts = span.extensions();
                if let Some(fields) = exts.get::<SpanFields>() {
                    let _ = writer.write_all(&fields.0[..]);
                }
            }
        }

        // Record event fields
        // TODO: make thius configurable
        // put_metadata(&mut writer, event.metadata(), None);
        event.record(&mut writer);
    }
}
struct SpanFields(Vec<u8>);

struct SpanVisitor<'a> {
    buf: &'a mut Vec<u8>,
    depth: usize,
}

impl<'a> SpanVisitor<'a> {
    fn new(buf: &'a mut Vec<u8>, depth: usize) -> Self {
        Self { buf, depth }
    }
}

impl Visit for SpanVisitor<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        write!(self.buf, "s{}_", self.depth).unwrap();
        write_debug(&mut self.buf, field.name(), value);
    }
}

impl Visit for AndroidWriter<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            // omit message field value
            let _ = write!(self, "{:?}", value);
        } else {
            let _ = write_debug(self, field.name(), value);
        }
    }
}

fn put_metadata(mut buf: impl io::Write, meta: &Metadata, span: Option<usize>) {
    write_name_with_value(&mut buf, "target", meta.target(), span);
    if let Some(file) = meta.file() {
        write_name_with_value(&mut buf, "file", file, span);
    }
    if let Some(line) = meta.line() {
        write_name_with_value(&mut buf, "line", line, span);
    }
}

fn write_debug(mut buf: impl io::Write, name: &str, value: &dyn fmt::Debug) {
    let _ = write!(&mut buf, "{}={:?}", name, value);
}

fn write_name_with_value<T>(mut buf: impl io::Write, name: &str, value: T, span: Option<usize>)
where
    T: Display,
{
    if let Some(n) = span {
        let _ = write!(&mut buf, "s{}_", n);
    }
    let _ = write!(buf, "{}={}", name, value);
}

fn write_value<T>(mut buf: impl io::Write, value: T)
where
    T: Display,
{
    let _ = write!(&mut buf, "{}", value);
}
