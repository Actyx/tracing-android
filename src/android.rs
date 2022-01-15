use android_log_sys::LogPriority;
use std::{
    ffi::{CStr, CString},
    io::{self, BufWriter},
    ops::Deref,
};
use tracing::Level;

fn android_log(prio: android_log_sys::LogPriority, tag: &CStr, msg: &CStr) {
    unsafe {
        android_log_sys::__android_log_write(
            prio as android_log_sys::c_int,
            tag.as_ptr() as *const android_log_sys::c_char,
            msg.as_ptr() as *const android_log_sys::c_char,
        )
    };
}

const LOGGING_TAG_MAX_LEN: usize = 23;
const LOGGING_MSG_MAX_LEN: usize = 4000;

pub(crate) struct CappedTag(CString);
impl CappedTag {
    pub fn new(tag: &[u8]) -> io::Result<Self> {
        let tag = if tag.len() > LOGGING_TAG_MAX_LEN {
            CString::new(
                tag.iter()
                    .take(LOGGING_TAG_MAX_LEN - 2)
                    .chain(b"..\0".iter())
                    .copied()
                    .collect::<Vec<_>>(),
            )
        } else {
            CString::new(tag.to_vec())
        }?;
        Ok(Self(tag))
    }
}
impl Deref for CappedTag {
    type Target = CStr;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub(crate) struct AndroidWriter<'a> {
    inner: BufWriter<LogcatWriter<'a>>,
}
impl<'a> AndroidWriter<'a> {
    pub fn new(level: &Level, tag: &'a CappedTag) -> Self {
        let w = LogcatWriter {
            priority: match *level {
                Level::WARN => LogPriority::WARN,
                Level::INFO => LogPriority::INFO,
                Level::DEBUG => LogPriority::DEBUG,
                Level::ERROR => LogPriority::ERROR,
                Level::TRACE => LogPriority::VERBOSE,
            },
            tag,
        };
        let inner = BufWriter::with_capacity(LOGGING_MSG_MAX_LEN, w);
        Self { inner }
    }
}
impl<'a> io::Write for AndroidWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
struct LogcatWriter<'a> {
    priority: LogPriority,
    tag: &'a CappedTag,
}
impl<'a> LogcatWriter<'a> {
    fn log(&self, msg: &[u8]) -> io::Result<()> {
        let msg = CString::new(msg.to_vec())?;
        android_log(self.priority, self.tag, &msg);
        Ok(())
    }
}
impl<'a> io::Write for LogcatWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = buf.len().min(LOGGING_MSG_MAX_LEN);
        self.log(&buf[..written])?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
