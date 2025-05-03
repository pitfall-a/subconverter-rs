use std::str::FromStr;

use rquickjs::{function::Args, Context, Function, IntoJs, Runtime};

use crate::Settings;

use super::{Proxy, RegexMatchConfigs};

/// Settings for subscription export operations
pub struct ExtraSettings {
    /// Whether to enable the rule generator
    pub enable_rule_generator: bool,
    /// Whether to overwrite original rules
    pub overwrite_original_rules: bool,
    /// Rename operations to apply
    pub rename_array: RegexMatchConfigs,
    /// Emoji operations to apply
    pub emoji_array: RegexMatchConfigs,
    /// Whether to add emoji
    pub add_emoji: bool,
    /// Whether to remove emoji
    pub remove_emoji: bool,
    /// Whether to append proxy type
    pub append_proxy_type: bool,
    /// Whether to output as node list
    pub nodelist: bool,
    /// Whether to sort nodes
    pub sort_flag: bool,
    /// Whether to filter deprecated nodes
    pub filter_deprecated: bool,
    /// Whether to use new field names in Clash
    pub clash_new_field_name: bool,
    /// Whether to use scripts in Clash
    pub clash_script: bool,
    /// Path to Surge SSR binary
    pub surge_ssr_path: String,
    /// Prefix for managed configs
    pub managed_config_prefix: String,
    /// QuantumultX device ID
    pub quanx_dev_id: String,
    /// UDP support flag
    pub udp: Option<bool>,
    /// TCP Fast Open support flag
    pub tfo: Option<bool>,
    /// Skip certificate verification flag
    pub skip_cert_verify: Option<bool>,
    /// TLS 1.3 support flag
    pub tls13: Option<bool>,
    /// Whether to use classical ruleset in Clash
    pub clash_classical_ruleset: bool,
    /// Script for sorting nodes
    pub sort_script: String,
    /// Style for Clash proxies output
    pub clash_proxies_style: String,
    /// Style for Clash proxy groups output
    pub clash_proxy_groups_style: String,
    /// Whether the export is authorized
    pub authorized: bool,
    /// JavaScript runtime context (not implemented in Rust version)
    pub js_context: Option<Context>,
    /// JavaScript runtime
    pub js_runtime: Option<Runtime>,
}

impl std::fmt::Debug for ExtraSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ExtraSettings")
            .field("enable_rule_generator", &self.enable_rule_generator)
            .field("overwrite_original_rules", &self.overwrite_original_rules)
            .field("rename_array", &self.rename_array)
            .field("emoji_array", &self.emoji_array)
            .field("add_emoji", &self.add_emoji)
            .field("remove_emoji", &self.remove_emoji)
            .field("append_proxy_type", &self.append_proxy_type)
            .field("nodelist", &self.nodelist)
            .field("sort_flag", &self.sort_flag)
            .field("filter_deprecated", &self.filter_deprecated)
            .field("clash_new_field_name", &self.clash_new_field_name)
            .field("clash_script", &self.clash_script)
            .field("surge_ssr_path", &self.surge_ssr_path)
            .field("managed_config_prefix", &self.managed_config_prefix)
            .field("quanx_dev_id", &self.quanx_dev_id)
            .field("udp", &self.udp)
            .field("tfo", &self.tfo)
            .field("skip_cert_verify", &self.skip_cert_verify)
            .field("tls13", &self.tls13)
            .field("clash_classical_ruleset", &self.clash_classical_ruleset)
            .field("sort_script", &self.sort_script)
            .field("clash_proxies_style", &self.clash_proxies_style)
            .field("clash_proxy_groups_style", &self.clash_proxy_groups_style)
            .field("authorized", &self.authorized)
            .finish()
    }
}

impl Default for ExtraSettings {
    fn default() -> Self {
        let global = Settings::current();

        ExtraSettings {
            enable_rule_generator: global.enable_rule_gen,
            overwrite_original_rules: global.overwrite_original_rules,
            rename_array: Vec::new(),
            emoji_array: Vec::new(),
            add_emoji: false,
            remove_emoji: false,
            append_proxy_type: false,
            nodelist: false,
            sort_flag: false,
            filter_deprecated: false,
            clash_new_field_name: true,
            clash_script: false,
            surge_ssr_path: global.surge_ssr_path.clone(),
            managed_config_prefix: String::new(),
            quanx_dev_id: String::new(),
            udp: None,
            tfo: None,
            skip_cert_verify: None,
            tls13: None,
            clash_classical_ruleset: false,
            sort_script: String::new(),
            clash_proxies_style: if global.clash_proxies_style.is_empty() {
                "flow".to_string()
            } else {
                global.clash_proxies_style.clone()
            },
            clash_proxy_groups_style: if global.clash_proxy_groups_style.is_empty() {
                "flow".to_string()
            } else {
                global.clash_proxy_groups_style.clone()
            },
            authorized: false,
            js_context: None,
            js_runtime: None,
        }
    }
}

impl ExtraSettings {
    fn init_js_context(&mut self) {
        if self.js_runtime.is_none() {
            self.js_runtime = Some(Runtime::new().unwrap());
            self.js_context = Some(Context::full(&self.js_runtime.as_ref().unwrap()).unwrap());
        }
    }

    pub fn eval_filter_function(
        &mut self,
        nodes: &mut Vec<Proxy>,
        source_str: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.init_js_context();
        if let Some(context) = &mut self.js_context {
            let mut error_thrown = None;
            context.with(|ctx| {
                match ctx.eval(source_str) {
                    Ok(value) => value,
                    Err(e) => {
                        match e {
                            rquickjs::Error::Exception => {
                                log::error!(
                                    "JavaScript eval throw exception: {}",
                                    ctx.catch()
                                        .try_into_string()
                                        .unwrap()
                                        .to_string()
                                        .unwrap_or_default()
                                );
                            }
                            _ => {
                                log::error!("JavaScript eval error: {}", e);
                            }
                        }
                        error_thrown = Some(e);
                        return;
                    }
                };
                let filter_evaluated: rquickjs::Function =
                    match ctx.globals().get::<_, rquickjs::Function>("filter") {
                        Ok(value) => value,
                        Err(e) => {
                            log::error!("JavaScript eval get function error: {}", e);
                            return;
                        }
                    };

                nodes.retain_mut(|node| {
                    match filter_evaluated.call::<(Proxy,), bool>((node.clone(),)) {
                        Ok(value) => value,
                        Err(e) => {
                            log::error!("JavaScript eval call function error: {}", e);
                            false
                        }
                    }
                });
            });
            match error_thrown {
                Some(e) => Err(e.into()),
                None => {
                    log::info!("Filter function evaluated successfully");
                    Ok(())
                }
            }
        } else {
            Err("JavaScript context not initialized".into())
        }
    }
}
