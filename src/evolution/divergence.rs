use crate::knowledge::entry::Entry;
use crate::knowledge::tags::TagMatcher;
use std::collections::HashSet;

pub struct DivergenceFlag {
    pub entry: Entry,
    pub diverging_count: usize,
    pub diverging_projects: Vec<String>,
}

pub struct DivergenceDetector {
    tag_threshold: f64,
    project_threshold: usize,
}

impl DivergenceDetector {
    pub fn new(tag_threshold: f64, project_threshold: usize) -> Self {
        Self {
            tag_threshold,
            project_threshold,
        }
    }

    pub fn detect(&self, global: &[Entry], recent: &[Entry]) -> Vec<DivergenceFlag> {
        let mut flags = Vec::new();

        for global_entry in global {
            let mut diverging_projects = HashSet::new();

            for recent_entry in recent {
                let overlap = TagMatcher::overlap_score(&global_entry.tags, &recent_entry.tags);
                if overlap >= self.tag_threshold {
                    for origin in &recent_entry.origins {
                        let global_projects: HashSet<&str> = global_entry
                            .origins
                            .iter()
                            .map(|o| o.project.as_str())
                            .collect();
                        if !global_projects.contains(origin.project.as_str()) {
                            diverging_projects.insert(origin.project.clone());
                        }
                    }
                }
            }

            if diverging_projects.len() >= self.project_threshold {
                let projects: Vec<String> = diverging_projects.into_iter().collect();
                flags.push(DivergenceFlag {
                    entry: global_entry.clone(),
                    diverging_count: projects.len(),
                    diverging_projects: projects,
                });
            }
        }

        flags
    }
}
