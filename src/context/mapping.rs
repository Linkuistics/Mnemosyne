use crate::config::Config;
use crate::context::detect::Signal;
use crate::knowledge::entry::Tag;
use std::collections::HashSet;

pub struct SignalMapper<'a> {
    config: &'a Config,
}

impl<'a> SignalMapper<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn map_signals(&self, signals: &[Signal]) -> Vec<Tag> {
        let mut tags = HashSet::new();

        for signal in signals {
            match signal {
                Signal::Language(lang) => {
                    tags.insert(lang.clone());
                }
                Signal::Dependency { ecosystem, name } => {
                    if let Some(eco_mappings) = self.config.context_mappings.get(ecosystem) {
                        if let Some(mapped_tags) = eco_mappings.get(name) {
                            for tag in mapped_tags {
                                tags.insert(tag.clone());
                            }
                        } else {
                            tags.insert(name.clone());
                        }
                    } else {
                        tags.insert(name.clone());
                    }
                }
                Signal::ProjectName(_) => {}
            }
        }

        let mut result: Vec<Tag> = tags.into_iter().collect();
        result.sort();
        result
    }
}
