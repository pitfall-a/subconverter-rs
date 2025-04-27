use log::{debug, info};
use std::cmp::Ordering;

use crate::models::{
    extra_settings::ExtraSettings,
    proxy::{Proxy, ProxyType},
    regex_match_config::RegexMatchConfigs,
};
use crate::utils::{
    matcher::{apply_matcher, reg_find},
    reg_replace,
    string::{remove_emoji, trim},
};

use super::matcher::apply_compiled_rule;

/// Applies a rename configuration to a node
/// Similar to the C++ nodeRename function
fn node_rename(node: &mut Proxy, rename_array: &RegexMatchConfigs, _extra: &ExtraSettings) {
    let original_remark = node.remark.clone();

    for pattern in rename_array {
        // Skip script-based patterns since we're not implementing JavaScript support
        // here
        if !pattern._match.is_empty() {
            let mut real_rule = String::new();
            if apply_matcher(&pattern._match, &mut real_rule, node) && !real_rule.is_empty() {
                node.remark = reg_replace(&node.remark, &real_rule, &pattern.replace, true, false);
            }
        }
    }

    // If the remark is empty after processing, restore the original
    if node.remark.is_empty() {
        node.remark = original_remark;
    }
}

/// Adds emoji to node remark based on regex matching
fn add_emoji(node: &Proxy, emoji_array: &RegexMatchConfigs, _extra: &ExtraSettings) -> String {
    for pattern in emoji_array {
        // Skip patterns with empty replace
        if pattern.replace.is_empty() {
            continue;
        }

        // Use apply_compiled_rule to handle complex matching rules
        if let Some(compiled_rule) = &pattern.compiled_rule {
            if apply_compiled_rule(compiled_rule, node) {
                return format!("{} {}", pattern.replace, node.remark);
            }
        }
    }

    node.remark.clone()
}

/// Sorts nodes by a specified criterion
fn sort_nodes(nodes: &mut Vec<Proxy>, _sort_script: &str) {
    // Skip script-based sorting since we're not implementing JavaScript support
    // Default sort by remark
    nodes.sort_by(|a, b| {
        if a.proxy_type == ProxyType::Unknown {
            return Ordering::Greater;
        }
        if b.proxy_type == ProxyType::Unknown {
            return Ordering::Less;
        }
        a.remark.cmp(&b.remark)
    });
}

/// Preprocesses nodes before conversion
/// Based on the C++ preprocessNodes function
pub fn preprocess_nodes(
    nodes: &mut Vec<Proxy>,
    extra: &ExtraSettings,
    rename_patterns: &RegexMatchConfigs,
    emoji_patterns: &RegexMatchConfigs,
) {
    // Process each node
    for node in nodes.iter_mut() {
        // Remove emoji if needed
        if extra.remove_emoji {
            node.remark = trim(&remove_emoji(&node.remark)).to_string();
        }

        // Apply rename patterns
        node_rename(node, rename_patterns, extra);

        // Add emoji if needed
        if extra.add_emoji {
            node.remark = add_emoji(node, emoji_patterns, extra);
        }
    }

    // Sort nodes if needed
    if extra.sort_flag {
        info!("Sorting {} nodes", nodes.len());
        sort_nodes(nodes, &extra.sort_script);
    }

    debug!("Node preprocessing completed for {} nodes", nodes.len());
}

/// Appends proxy type to node remark
pub fn append_type_to_remark(nodes: &mut Vec<Proxy>) {
    for node in nodes.iter_mut() {
        match node.proxy_type {
            ProxyType::Unknown => {}
            _ => {
                let type_str = node.proxy_type.to_string();
                node.remark = format!("{} ({})", node.remark, type_str);
            }
        }
    }
}
