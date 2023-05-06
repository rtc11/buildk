use std::collections::BTreeMap;
use std::str::FromStr;

use toml_edit::{Document, Item, Table, Value};

use crate::dependencies::dependency::Dependency;
use crate::dependencies::kind::Kind;
use crate::section::Section;

#[cfg(test)]
mod test;
pub mod dependency;
mod kind;

pub(crate) fn dependencies(data: &Document) -> Vec<Dependency> {
    data.as_table().into_iter().flat_map(|(key, value)| {
        match Section::from_str(key) {
            Ok(Section::Dependencies) =>
                match value.as_table() {
                    None => vec![],
                    Some(table) => dependencies_for(table, Kind::Source)
                }
            Ok(Section::TestDependencies) =>
                match value.as_table() {
                    None => vec![],
                    Some(table) => dependencies_for(table, Kind::Test)
                }
            _ => vec![]
        }
    }).collect::<Vec<Dependency>>()
}

fn dependencies_for(table: &Table, kind: Kind) -> Vec<Dependency> {
    let mut map = BTreeMap::new();
    table.iter().for_each(|(key, value)| {
        map = decend(map.clone(), vec![key], value);
    });
    map.into_iter()
        .filter_map(|(key, value)| Dependency::from_toml(&kind, &key, value))
        .collect()
}

/// In TOML syntax a dot (.) represents an inline table and not part of the field name.
/// This is a workaround to get a list of all keys until the value field (that should be the version).
fn decend<'a>(
    mut map: BTreeMap<String, &'a Value>,
    keys: Vec<&'a str>,
    value: &'a Item,
) -> BTreeMap<String, &'a Value> {
    match value {
        Item::Value(value) => {
            map.insert(keys.join("."), value);
        }
        Item::Table(table) => {
            table.iter().for_each(|(key, value)| {
                let mut branching_keys = keys.clone();
                branching_keys.push(key);
                map = decend(map.clone(), branching_keys, value);
            });
        }
        _ => {} // do nothing
    }
    map
}
