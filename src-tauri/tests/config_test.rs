#[cfg(test)]
mod config_test {
    use std::collections::HashMap;

    use macropad_console_lib::config::{Action, AppConfig, ApplicationAction, ApplicationProfile};

    #[test]
    fn test_serialize_and_deserialize_config() {
        let config = AppConfig {
            application_profiles: HashMap::from_iter(vec![(
                "test_profile".to_string(),
                ApplicationProfile {
                    bindings: vec![()],
                },
            )]),
        };

        dbg!(&config);

        let json = serde_json::to_string(&config).unwrap();
        dbg!(&json);

        let expected = serde_json::to_string(
            &serde_json::from_str::<AppConfig>(
                r#"{
                "applicationProfiles": {
                    "test_profile": {
                        "bindings": []
                    }
                }
            }"#,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(json, expected);
    }
}
