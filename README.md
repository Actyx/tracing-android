# tracing-android

## tracing-android

Composable tracing layer which logs to logcat using the [Android NDK]'s
__android_log_write function. The provided tag will be capped at 23 bytes.
Logging events resulting in messages longer than 4000 bytes will result in
multiple log lines in logcat. This avoids running into logcat's truncation
behaviour.

[Android NDK]: https://developer.android.com/ndk/reference/group/logging#__android_log_write

License: MIT OR Apache-2.0
