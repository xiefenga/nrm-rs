use std::{collections::HashMap, fs, path::PathBuf};

use clap::{arg, Command};
use ini::Ini;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;

lazy_static! {
    static ref NPMRC: PathBuf = dirs::home_dir().expect("home dir not found").join(".nrm");
}

#[derive(Debug, Serialize, Deserialize)]
struct Registry {
    home: String,
    registry: String,
}

impl TryFrom<Value> for Registry {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if value.is_object() {
            let home = value["home"].as_str();
            let registry = value["registry"].as_str();
            if home.is_some() && registry.is_some() {
                Ok(Registry {
                    home: home.unwrap().to_string(),
                    registry: registry.unwrap().to_string(),
                })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

fn main() {
    let app = Command::new("nrm-rs")
        .version("0.0.1")
        .author("xiefeng")
        .about("a rust version of nrm")
        .subcommand(Command::new("ls").about("list all the registries"))
        .subcommand(Command::new("current").about("show current registry url"))
        .subcommand(
            Command::new("use")
                .about("set current registry url")
                .arg(arg!(<Registry> "The registry name")),
        );

    let matches = app.get_matches();

    let flags = HashMap::from([(true, "*"), (false, " ")]);

    match matches.subcommand() {
        Some(("ls", _)) => {
            let current_registry = get_current_registry();
            let registries = get_registries();
            for (registry, registry_config) in registries {
                let flag = flags[&(current_registry == registry_config.registry)];
                let width = 13 - registry.len();
                println!(
                    "{} {} {:-<width$} {}",
                    flag, registry, "", registry_config.registry
                );
            }
        }
        Some(("current", _)) => {
            let registry = get_current_registry();
            println!("{}", registry);
        }
        Some(("use", sub_matches)) => {
            let name = sub_matches.value_of("Registry").unwrap();
            let registries = get_registries();
            let registry_config = registries.get(name);
            if let Some(config) = registry_config {
                let registry = config.registry.as_ref();
                set_current_registry(registry);
                println!("Registry has been set to: {}", registry)
            } else {
                eprintln!("Not find registry: {}", name);
            }
        }
        _ => {}
    }
}

fn get_current_registry() -> String {
    let npmrc = Ini::load_from_file(NPMRC.as_path()).unwrap();
    let registry = npmrc.get_from(None::<String>, "registry").unwrap();
    registry.to_string()
}

fn get_registries() -> HashMap<String, Registry> {
    let data = fs::read_to_string("./registries.json").unwrap();
    let registries: Value = serde_json::from_str(&data).unwrap();
    let registries = registries.as_object().unwrap().to_owned();
    registries
        .into_iter()
        .map(|(k, v)| (k, v.try_into().unwrap()))
        .collect()
}

fn set_current_registry(registry: &str) {
    let mut npmrc = Ini::load_from_file(NPMRC.as_path()).unwrap();
    npmrc.with_section(None::<String>).set("registry", registry);
    npmrc.write_to_file(NPMRC.as_path()).unwrap();
}
