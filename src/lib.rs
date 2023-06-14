#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![feature(iter_collect_into)]

use swc_core::{
    ecma::{ast::Program, visit::FoldWith},
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

pub mod config;
pub mod transform;
mod utils;

#[plugin_transform]
fn swc_plugin_jsx_attrs(program: Program, data: TransformPluginProgramMetadata) -> Program {
    // Deserialize the configuration from a JSON string using serde_json::from_str.
    // If the configuration is invalid, panic with an error message.
    let cfg = serde_json::from_str(
        &data
            .get_transform_plugin_config()
            .expect("[swc_plugin_jsx_attrs] parse json config failed"),
    )
    .expect("[swc_plugin_jsx_attrs] invalid config");

    program.fold_with(&mut transform::transform(cfg))
}
