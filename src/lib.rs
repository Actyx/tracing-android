//! # tracing-android
//!
//! Composable tracing layer which logs to logcat using the [Android NDK]'s
//! __android_log_write function. The provided tag will be capped at 23 bytes.
//! Logging events resulting in messages longer than 4000 bytes will result in
//! multiple log lines in logcat. This avoids running into logcat's truncation
//! behaviour.
//!
//! [Android NDK]: https://developer.android.com/ndk/reference/group/logging#__android_log_write
mod android;
mod layer;

/// Constructs a [`layer::Layer`] with the given `tag`.
/// ```no_run
/// // add the layer to an existing subscriber
/// let subscriber = {
///     use tracing_subscriber::layer::SubscriberExt;
///     subscriber.with(tracing_win_event_log::layer("com.example").unwrap())
/// }
// // .. install the subscriber ..
/// ```
pub fn layer(tag: &str) -> std::io::Result<layer::Layer> {
    layer::Layer::new(tag)
}
