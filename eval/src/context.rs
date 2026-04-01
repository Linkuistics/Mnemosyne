use std::collections::HashSet;

use mnemosyne::config::Config;
use mnemosyne::context::detect::ProjectDetector;
use mnemosyne::context::detect::Signal;
use mnemosyne::context::mapping::SignalMapper;

use crate::corpus::Corpus;

#[derive(Debug)]
pub struct ContextMetrics {
    pub language_accuracy: f64,
    pub dependency_accuracy: f64,
    pub tag_mapping_accuracy: f64,
    pub project_count: usize,
}

pub fn evaluate_context(corpus: &Corpus) -> ContextMetrics {
    let config = Config::default();
    let detector = ProjectDetector::new(&config);
    let mapper = SignalMapper::new(&config);

    let mut language_correct = 0;
    let mut language_total = 0;
    let mut dep_correct = 0;
    let mut dep_total = 0;
    let mut tag_correct = 0;
    let mut tag_total = 0;

    for project in &corpus.projects {
        let signals = match detector.detect(&project.path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Language accuracy
        let detected_languages: HashSet<String> = signals
            .iter()
            .filter_map(|s| match s {
                Signal::Language(lang) => Some(lang.clone()),
                _ => None,
            })
            .collect();
        let expected_languages: HashSet<String> =
            project.expected.languages.iter().cloned().collect();

        language_total += 1;
        if detected_languages == expected_languages {
            language_correct += 1;
        }

        // Dependency accuracy
        let detected_deps: HashSet<String> = signals
            .iter()
            .filter_map(|s| match s {
                Signal::Dependency { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();
        let expected_deps: HashSet<String> =
            project.expected.dependencies.iter().cloned().collect();

        for dep in &expected_deps {
            dep_total += 1;
            if detected_deps.contains(dep) {
                dep_correct += 1;
            }
        }

        // Tag mapping accuracy
        let mapped_tags: HashSet<String> =
            mapper.map_signals(&signals).into_iter().collect();
        let expected_tags: HashSet<String> =
            project.expected.expected_tags.iter().cloned().collect();

        for tag in &expected_tags {
            tag_total += 1;
            if mapped_tags.contains(tag) {
                tag_correct += 1;
            }
        }
    }

    ContextMetrics {
        language_accuracy: if language_total > 0 {
            language_correct as f64 / language_total as f64
        } else {
            0.0
        },
        dependency_accuracy: if dep_total > 0 {
            dep_correct as f64 / dep_total as f64
        } else {
            0.0
        },
        tag_mapping_accuracy: if tag_total > 0 {
            tag_correct as f64 / tag_total as f64
        } else {
            0.0
        },
        project_count: corpus.projects.len(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_accuracy_computation() {
        let correct = 3.0_f64;
        let total = 4.0_f64;
        assert!((correct / total - 0.75).abs() < 1e-10);
    }
}
