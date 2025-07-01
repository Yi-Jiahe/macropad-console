#[cfg(test)]
mod config_test {
    use std::collections::{HashMap, HashSet};

    use paste::paste;
    use serde_test::{assert_tokens, Token};

    use macropad_console_lib::config::{Action, AppConfig, ApplicationProfile, KeyCombination};

    #[test]
    fn test_serialize_and_deserialize_config() {
        let config = AppConfig {
            application_profiles: HashMap::from_iter(vec![(
                "test_profile".to_string(),
                ApplicationProfile { bindings: vec![] },
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

    #[test]
    fn test_ser_de_key_combination() {
        let expected = "BTN_4+ENC_0_INC";
        let c = KeyCombination {
            modifiers: Some(HashSet::from_iter(vec![4])),
            action: Action::EncoderIncrement { id: 0 },
        };

        dbg!(&c);

        assert_tokens(&c, &[Token::Str(expected)]);
    }

    macro_rules! ser_de_key_combination_test {
        ($name:ident, $expected:expr, $c:expr) => {
            paste! {
                #[test]
                fn [<test_ser_de_key_combination_ $name>]() {
                    assert_tokens(&$c, $expected);
                }
            }
        };
    }

    ser_de_key_combination_test!(
        button_press,
        &[Token::Str("BTN_0"),],
        KeyCombination {
            modifiers: None,
            action: Action::ButtonPress { id: 0 },
        }
    );
    ser_de_key_combination_test!(
        button_press_10,
        &[Token::Str("BTN_10"),],
        KeyCombination {
            modifiers: None,
            action: Action::ButtonPress { id: 10 },
        }
    );
    ser_de_key_combination_test!(
        encoder_increment,
        &[Token::Str("ENC_0_INC"),],
        KeyCombination {
            modifiers: None,
            action: Action::EncoderIncrement { id: 0 },
        }
    );
    ser_de_key_combination_test!(
        encoder_decrement,
        &[Token::Str("ENC_0_DEC"),],
        KeyCombination {
            modifiers: None,
            action: Action::EncoderDecrement { id: 0 },
        }
    );
    ser_de_key_combination_test!(
        two_modifiers_btn_press,
        &[Token::Str("BTN_4+BTN_8+BTN_7"),],
        KeyCombination {
            modifiers: Some(HashSet::from_iter(vec![4, 8])),
            action: Action::ButtonPress { id: 7 },
        }
    );

    #[test]
    fn test_key_combination_eq() {
        let l = KeyCombination {
            modifiers: Some(HashSet::from_iter(vec![4, 8])),
            action: Action::ButtonPress { id: 7 },
        };

        let json = r#""BTN_4+BTN_8+BTN_7""#;
        let r = serde_json::from_str::<KeyCombination>(&json).unwrap();

        assert_eq!(l, r);
    }

    macro_rules! key_combination_eq_test {
        ($name:ident, $l:expr, $r:expr) => {
            paste! {
                #[test]
                fn [<test_key_combination_eq_ $name>]() {
                    let l = $l;

                    let json = $r;
                    let r = serde_json::from_str::<KeyCombination>(&json).unwrap();

                    assert_eq!(l, r);
                }
            }
        };
    }

    key_combination_eq_test!(
        button_press,
        KeyCombination {
            modifiers: None,
            action: Action::ButtonPress { id: 0 },
        },
        r#""BTN_0""#
    );
    key_combination_eq_test!(
        encoder_increment,
        KeyCombination {
            modifiers: None,
            action: Action::EncoderIncrement { id: 0 },
        },
        r#""ENC_0_INC""#
    );
    key_combination_eq_test!(
        encoder_decrement,
        KeyCombination {
            modifiers: None,
            action: Action::EncoderDecrement { id: 0 },
        },
        r#""ENC_0_DEC""#
    );
    key_combination_eq_test!(
        two_modifiers_unordered_btn_press,
        KeyCombination {
            modifiers: Some(HashSet::from_iter(vec![4, 8])),
            action: Action::ButtonPress { id: 7 },
        },
        r#""BTN_8+BTN_4+BTN_7""#
    );
}
