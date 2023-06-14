use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use swc_core::{
    common::{chain, Mark},
    ecma::{
        parser::{Syntax, TsConfig},
        transforms::{base::resolver, testing::test_fixture},
    },
};
use swc_plugin_jsx_attrs::{config::Config, transform::transform as jsx_attrs_transform};

#[testing::fixture("tests/fixture/**/input.tsx")]
fn test(input: PathBuf) {
    let config = input.parent().unwrap().join("config.json");
    let output = input.parent().unwrap().join("output.txt");
    test_fixture(
        Syntax::Typescript(TsConfig {
            tsx: true,
            ..Default::default()
        }),
        &|_| {
            chain!(
                // This transformer analyze and inject syntax contexts.
                resolver(Mark::new(), Mark::new(), true),
                jsx_attrs_transform(Config {
                    inject: serde_json::from_reader(BufReader::new(File::open(&config).unwrap()))
                        .unwrap(),
                })
            )
        },
        &input,
        &output,
        Default::default(),
    );
}
