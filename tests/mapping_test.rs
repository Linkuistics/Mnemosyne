use mnemosyne::config::Config;
use mnemosyne::context::detect::Signal;
use mnemosyne::context::mapping::SignalMapper;

#[test]
fn test_map_language_signal_to_tags() {
    let config = Config::default();
    let mapper = SignalMapper::new(&config);

    let signals = vec![Signal::Language("rust".to_string())];
    let tags = mapper.map_signals(&signals);

    assert!(tags.contains(&"rust".to_string()));
}

#[test]
fn test_map_dependency_signal_to_tags() {
    let config = Config::default();
    let mapper = SignalMapper::new(&config);

    let signals = vec![
        Signal::Language("rust".to_string()),
        Signal::Dependency {
            ecosystem: "cargo_dependencies".to_string(),
            name: "tokio".to_string(),
        },
    ];
    let tags = mapper.map_signals(&signals);

    assert!(tags.contains(&"rust".to_string()));
    assert!(tags.contains(&"async".to_string()));
    assert!(tags.contains(&"tokio".to_string()));
}

#[test]
fn test_map_unknown_dependency_uses_name_as_tag() {
    let config = Config::default();
    let mapper = SignalMapper::new(&config);

    let signals = vec![Signal::Dependency {
        ecosystem: "cargo_dependencies".to_string(),
        name: "obscure-crate".to_string(),
    }];
    let tags = mapper.map_signals(&signals);

    assert!(tags.contains(&"obscure-crate".to_string()));
}
