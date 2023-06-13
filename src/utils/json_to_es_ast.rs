use serde_json::Value;
use swc_core::{common::DUMMY_SP, ecma::ast::*};

/// Create a swc_ecma_ast::Expr from a serde_json::Value.
pub fn json_to_es_ast(input: &Value) -> Box<Expr> {
    let expr = match input {
        Value::Array(value) => Expr::Array(ArrayLit {
            span: DUMMY_SP,
            elems: value
                .iter()
                .map(|elem| Some(json_to_es_ast(&elem).into()))
                .collect(),
        }),
        Value::Bool(value) => Expr::Lit(Lit::Bool(value.clone().into())),
        Value::Null => Expr::Lit(Lit::Null(Null { span: DUMMY_SP })),
        Value::Number(value) => Expr::Lit(Lit::Num(value.as_f64().unwrap_or_default().into())),
        Value::Object(value) => Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props: value
                .iter()
                .map(|(key, value)| {
                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Str(key.clone().into()),
                        value: json_to_es_ast(&value),
                    })))
                })
                .collect(),
        }),
        Value::String(value) => Expr::Lit(Lit::Str(value.clone().into())),
    };
    Box::new(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use swc_core::{common::source_map, common::sync::Lrc, ecma::codegen};

    #[test]
    fn test_primitive() {
        test_fixture(json!(123), "123;\n");
        test_fixture(json!(null), "null;\n");
        test_fixture(json!(true), "true;\n");
        test_fixture(json!(false), "false;\n");
        test_fixture(json!("你好"), "\"你好\";\n");
    }

    #[test]
    fn test_array() {
        test_fixture(json!([]), "[];\n");
        test_fixture(
            json!([123, null, true, false, "hello"]),
            "[\n    123,\n    null,\n    true,\n    false,\n    \"hello\"\n];\n",
        );
    }

    #[test]
    fn test_object() {
        test_fixture(json!({}), "{};\n");
        test_fixture(
            json!({ "hello": [], "Xxx": false }),
            "{\n    \"Xxx\": false,\n    \"hello\": []\n};\n",
        );
    }

    fn test_fixture(json: Value, should_be: &str) {
        let s = print_module(&Module {
            span: DUMMY_SP,
            body: vec![ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: json_to_es_ast(&json),
            }))],
            shebang: None,
        });
        testing::assert_eq!(s.as_str(), should_be);
    }

    fn print_module(module: &Module) -> String {
        let sm = Lrc::from(source_map::SourceMap::new(Default::default()));
        let mut buf = vec![];
        let mut emitter = codegen::Emitter {
            cfg: Default::default(),
            comments: None,
            cm: sm.clone(),
            wr: Box::new(codegen::text_writer::JsWriter::new(
                sm.clone(),
                "\n",
                &mut buf,
                None,
            )),
        };
        emitter.emit_module(&module).unwrap();
        String::from_utf8_lossy(&buf).to_string()
    }
}
