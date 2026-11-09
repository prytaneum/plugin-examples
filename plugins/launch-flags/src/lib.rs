use hestia::hooks::{Intercept, Intercepted};
use hestia::proto::hook::{BeforeLaunch, LaunchPatch};
use hestia::Plugin;

/// Adds JVM flags on the host's allowlist; anything else it asked for would be dropped with a warning.
struct LaunchFlags;

impl Plugin for LaunchFlags {
    fn intercept(call: Intercept) -> Intercepted {
        call.on::<BeforeLaunch>(|context| {
            let collector = hestia::settings::get::<String>("collector")
                .ok()
                .flatten()
                .unwrap_or_default();
            let flags: &[&str] = match collector.as_str() {
                "zgc" => &["-XX:+UseZGC"],
                _ => &["-XX:+UseG1GC", "-XX:MaxGCPauseMillis=130"],
            };
            hestia::log::info(&format!("{collector} flags for {}", context.instance_name));
            Ok(LaunchPatch {
                extra_jvm_args: flags.iter().map(|flag| flag.to_string()).collect(),
                ..LaunchPatch::default()
            })
        })
        .finish()
    }
}

hestia::plugin!(LaunchFlags);
