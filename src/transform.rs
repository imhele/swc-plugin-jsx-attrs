use crate::utils::json_to_es_ast::json_to_es_ast;
use std::collections::HashMap;
use swc_core::{
    cached::regex::CachedRegex,
    common::DUMMY_SP,
    ecma::visit::Fold,
    ecma::{ast::*, visit::FoldWith},
};

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub inject: HashMap<String, HashMap<String, Vec<InjectAttr>>>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectAttr {
    /// jsx attribute name
    pub name: String,
    /// inject rule
    pub rule: InjectRule,
    /// jsx attribute value
    pub value: serde_json::Value,
    /// ast cache of jsx attribute value
    #[serde(skip)]
    value_ast_cache: Option<Box<Expr>>,
}

impl InjectAttr {
    fn create_value_ast(&mut self) -> Box<Expr> {
        match &self.value_ast_cache {
            Some(ast) => ast.clone(),
            None => {
                let ast = json_to_es_ast(&self.value);
                self.value_ast_cache = Some(ast.clone());
                ast
            }
        }
    }

    fn inject_attr(&mut self, attrs: &mut Vec<JSXAttrOrSpread>) {
        match &self.rule {
            InjectRule::Prepend => {
                let attr = JSXAttr {
                    name: JSXAttrName::Ident(Ident::new(self.name.clone().into(), DUMMY_SP)),
                    span: DUMMY_SP,
                    value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                        expr: JSXExpr::Expr(self.create_value_ast()),
                        span: DUMMY_SP,
                    })),
                };
                attrs.insert(0, attr.into());
            }
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InjectRule {
    // Append,
    Prepend,
    // Replace,
}

#[derive(Debug)]
struct ImportedCompRef {
    /// imported id
    id: Id,
    /// member sub path like `Link` of `Dropdown.Link`
    sub: Vec<String>,
    /// jsx attributes
    attrs: Vec<InjectAttr>,
}

struct InjectComp {
    /// member path like `Button` and `Dropdown.Link`
    paths: Vec<String>,
    /// jsx attributes
    attrs: Vec<InjectAttr>,
}

impl InjectComp {
    fn collect_imported_refs(&self, decl: &ImportDecl) -> Vec<ImportedCompRef> {
        let comp_name = self
            .paths
            .get(0)
            .expect("[swc_plugin_jsx_attrs] invalid component name");
        decl.specifiers
            .iter()
            .filter_map(|spec| match spec {
                ImportSpecifier::Default(spec) => {
                    if comp_name == &"default" {
                        Some(ImportedCompRef {
                            id: spec.local.to_id(),
                            sub: self.paths[1..].into(),
                            attrs: self.attrs.clone(),
                        })
                    } else {
                        None
                    }
                }
                ImportSpecifier::Named(spec) => {
                    let import_name: &str = match &spec.imported {
                        Some(ModuleExportName::Ident(name)) => &name.sym,
                        Some(ModuleExportName::Str(name)) => &name.value,
                        None => &spec.local.sym,
                    };
                    if comp_name == import_name {
                        Some(ImportedCompRef {
                            id: spec.local.to_id(),
                            sub: self.paths[1..].into(),
                            attrs: self.attrs.clone(),
                        })
                    } else {
                        None
                    }
                }
                ImportSpecifier::Namespace(spec) => Some(ImportedCompRef {
                    id: spec.local.to_id(),
                    sub: self.paths.clone(),
                    attrs: self.attrs.clone(),
                }),
            })
            .collect()
    }
}

struct InjectPkg {
    // import source regex pattern
    import: CachedRegex,
    // target components
    comps: Vec<InjectComp>,
}

impl InjectPkg {
    fn collect_imported_refs(&self, decl: &ImportDecl) -> Vec<ImportedCompRef> {
        match self.import.is_match(&decl.src.value) {
            true => self
                .comps
                .iter()
                .flat_map(|comp| comp.collect_imported_refs(decl))
                .collect(),
            false => vec![],
        }
    }
}

struct FoldJSXAttrs {
    imported_refs: Vec<ImportedCompRef>,
    inject_config: Vec<InjectPkg>,
}

impl Fold for FoldJSXAttrs {
    fn fold_module_items(&mut self, nodes: Vec<ModuleItem>) -> Vec<ModuleItem> {
        nodes
            .iter()
            .filter_map(|node| -> Option<Vec<ImportedCompRef>> {
                match node {
                    ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) => Some(
                        self.inject_config
                            .iter()
                            .flat_map(|pkg| pkg.collect_imported_refs(decl))
                            .collect(),
                    ),
                    _ => None,
                }
            })
            .flatten()
            .collect_into(&mut self.imported_refs);
        nodes.fold_children_with(self)
    }

    fn fold_jsx_opening_element(&mut self, mut node: JSXOpeningElement) -> JSXOpeningElement {
        let name: Option<(Id, Vec<String>)> = match &node.name {
            JSXElementName::Ident(name) => Some((name.to_id(), vec![])),
            JSXElementName::JSXMemberExpr(member) => {
                let mut sub = vec![];
                let mut expr = member;
                loop {
                    sub.insert(0, expr.prop.sym.to_string());
                    match &expr.obj {
                        JSXObject::Ident(end) => {
                            break Some((end.to_id(), sub));
                        }
                        JSXObject::JSXMemberExpr(next) => {
                            expr = next;
                        }
                    }
                }
            }
            _ => None,
        };
        if let Some((id, sub)) = name {
            for imp_ref in &mut self.imported_refs {
                if (imp_ref.id == id) && (imp_ref.sub == sub) {
                    for attr in &mut imp_ref.attrs {
                        attr.inject_attr(&mut node.attrs);
                    }
                }
            }
        }
        node.fold_children_with(self)
    }
}

pub fn transform(config: Config) -> impl Fold {
    FoldJSXAttrs {
        imported_refs: vec![],
        inject_config: config
            .inject
            .into_iter()
            .map(|(ref import_ptn, comp_map)| InjectPkg {
                import: CachedRegex::new(import_ptn)
                    .expect("[swc_plugin_jsx_attrs] invalid regex pattern"),
                comps: comp_map
                    .into_iter()
                    .map(|(ref member, attrs)| InjectComp {
                        paths: member.split('.').map(|x| x.to_string()).collect(),
                        attrs,
                    })
                    .collect(),
            })
            .collect(),
    }
}
