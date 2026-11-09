# Hestia plugin examples

Small, working [Hestia](https://github.com/prytaneum/hestia) plugins to learn from. Each one shows one part of the
plugin API; none is meant to be installed for real use.

| Plugin | Shows |
|---|---|
| [`hello`](plugins/hello) | the smallest component: `init`, logging, the data directory, a command (`Plugin::run`) and an event (`Plugin::on_event`) |
| [`static-index`](plugins/static-index) | a `content` provider: `Plugin::provide` routing each operation with `.on::<Op>(…)` |
| [`tunnel-demo`](plugins/tunnel-demo) | `process:spawn`: a declared program started and stopped with a server |
| [`launch-flags`](plugins/launch-flags) | a `before-launch` hook: JVM flags on the host's allowlist, chosen by a plugin setting |
| [`dashboard`](plugins/dashboard) | pages: a `nav` page and an `entry` tab with `hestia::ui` builders and an action's effects |
| [`discord-presence`](plugins/discord-presence) | `ipc:connect`: Discord Rich Presence written entirely in the plugin, over Discord's local socket |
| [`theme-ember-night`](plugins/theme-ember-night) | a theme-only plugin: no component, only CSS variables |

The repository follows the [plugin repository shape](https://github.com/prytaneum/hestia/blob/main/docs/plugins/repository.md).
To start your own plugin, use `hestia plugin new <id>` instead of copying this repository.

## Build

Needs Rust with the `wasm32-wasip2` target and the `hestia` CLI.

```bash
rustup target add wasm32-wasip2
hestia plugin check
hestia plugin pack                      # dist/<id>-<version>.hplugin for every plugin
hestia plugin add dist/hello-0.1.0.hplugin
```

The SDK is not released yet, so the plugins track `hestia-sdk` from the `feature/plugins` branch of the Hestia
repository. Run `cargo update -p hestia-sdk` to pick up a newer SDK.

## License

`MIT OR Apache-2.0`, like the Hestia SDK.
