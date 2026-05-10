use super::{
    acp_config_option_dropdown_items, acp_config_option_selected_value,
};
use crate::ai::acp::config_options::{AcpConfigOption, AcpConfigOptionValue};
use std::collections::HashMap;

#[test]
fn acp_config_dropdown_selects_current_option_without_default_item() {
    let option = AcpConfigOption {
        id: "model".to_string(),
        name: "Model".to_string(),
        description: None,
        category: None,
        current_value: "gpt-5.5".to_string(),
        values: vec![
            AcpConfigOptionValue {
                id: "gpt-5.4".to_string(),
                name: "GPT-5.4".to_string(),
            },
            AcpConfigOptionValue {
                id: "gpt-5.5".to_string(),
                name: "GPT-5.5".to_string(),
            },
        ],
    };

    assert_eq!(
        acp_config_option_selected_value(&option, &HashMap::new()).as_deref(),
        Some("gpt-5.5")
    );

    let items = acp_config_option_dropdown_items(&option);
    assert_eq!(
        items
            .iter()
            .map(|item| item.display_text.as_str())
            .collect::<Vec<_>>(),
        vec!["GPT-5.4", "GPT-5.5"]
    );
    assert!(items.iter().all(|item| item.display_text != "Default"));
}
