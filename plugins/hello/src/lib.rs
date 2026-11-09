use hestia::{json, Error, Plugin, Value};

struct Hello;

impl Plugin for Hello {
    fn init() -> Result<(), Error> {
        hestia::log::info("hello from a plugin");
        hestia::data::save("greeting", &"hello")
    }

    fn run(command: &str, args: Value) -> Result<Value, Error> {
        match command {
            "greet" => {
                let name = args["name"].as_str().unwrap_or("world");
                Ok(json!(format!("hello, {name}")))
            }
            _ => Err(Error::NotFound(format!("unknown command `{command}`"))),
        }
    }

    fn on_event(topic: &str, payload: Value) {
        hestia::log::info(&format!("{topic}: {payload}"));
    }
}

hestia::plugin!(Hello);
