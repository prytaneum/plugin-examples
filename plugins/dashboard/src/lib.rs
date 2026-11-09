use hestia::proto::instance::InstanceList;
use hestia::proto::plugin::PageContext;
use hestia::proto::server::ServerList;
use hestia::proto::Empty;
use hestia::ui::{self, Action, Effect, FormField, Tone, UiNode};
use hestia::{Error, Plugin, Value};

struct Dashboard;

impl Plugin for Dashboard {
    fn render(page: &str, context: PageContext) -> Result<UiNode, Error> {
        match page {
            "overview" => overview(),
            "notes" => notes(
                &context
                    .entry
                    .ok_or_else(|| Error::Invalid("no entry".into()))?,
            ),
            _ => Err(Error::NotFound(format!("unknown page `{page}`"))),
        }
    }

    fn action(
        page: &str,
        action: Action,
        values: Value,
        context: PageContext,
    ) -> Result<Vec<Effect>, Error> {
        match (page, action.id.as_str()) {
            ("overview", "refresh") => Ok(vec![Effect::Refresh]),
            ("notes", "save") => {
                let entry = context
                    .entry
                    .ok_or_else(|| Error::Invalid("no entry".into()))?;
                hestia::data::save(&entry, &values["note"])?;
                Ok(vec![
                    Effect::Toast {
                        tone: Tone::Success,
                        text: "Note saved".into(),
                    },
                    Effect::Refresh,
                ])
            }
            _ => Err(Error::NotFound(format!("unknown action `{}`", action.id))),
        }
    }
}

fn overview() -> Result<UiNode, Error> {
    let servers = hestia::call::<ServerList>(&Empty {})?;
    let instances = hestia::call::<InstanceList>(&Empty {})?;
    let rows = instances
        .iter()
        .map(|entry| ui::row(["instance", &entry.name, &entry.game_version]))
        .chain(
            servers
                .iter()
                .map(|entry| ui::row(["server", &entry.name, &entry.game_version])),
        );
    Ok(ui::stack([
        ui::heading("Dashboard", 1),
        ui::detail([
            ("Instances", instances.len().to_string()),
            ("Servers", servers.len().to_string()),
        ]),
        ui::table(["Kind", "Name", "Version"], rows),
        ui::button("Refresh", "refresh"),
    ]))
}

fn notes(entry: &str) -> Result<UiNode, Error> {
    let note = hestia::data::load::<String>(entry)?.unwrap_or_default();
    Ok(ui::stack([
        ui::heading("Notes", 1),
        ui::form(
            "note",
            [FormField::Text {
                name: "note".into(),
                label: "Note".into(),
                value: note,
            }],
            "Save",
            "save",
        ),
    ]))
}

hestia::plugin!(Dashboard);
