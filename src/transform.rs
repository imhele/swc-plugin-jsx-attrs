use crate::config;
use crate::utils::json_to_es_ast::json_to_es_ast;
use swc_core::{
    cached::regex::CachedRegex,
    common::DUMMY_SP,
    ecma::visit::Fold,
    ecma::{ast::*, visit::FoldWith},
};

#[derive(Debug)]
struct ImportedCompRef {
    /// imported id
    id: Id,
    /// member sub path like `Link` of `Dropdown.Link`
    sub: Vec<String>,
    /// jsx attributes
    attrs: Vec<InjectAttr>,
}

#[derive(Debug)]
struct InjectAttr {
    cfg: config::InjectAttrConfig,
    value_ast_cache: Option<Box<Expr>>,
}

impl InjectAttr {
    fn create_attr(&mut self) -> JSXAttr {
        let expr = match &self.value_ast_cache {
            Some(ast) => ast.clone(),
            None => {
                let ast = json_to_es_ast(&self.cfg.value);
                self.value_ast_cache = Some(ast.clone());
                ast
            }
        };
        let expr = JSXExprContainer {
            expr: JSXExpr::Expr(expr),
            span: DUMMY_SP,
        };
        JSXAttr {
            name: Ident::new(self.cfg.name.clone().into(), DUMMY_SP).into(),
            span: DUMMY_SP,
            value: Some(expr.into()),
        }
    }

    fn inject_attr(&mut self, attrs: &mut Vec<JSXAttrOrSpread>) {
        match &self.cfg.rule {
            config::InjectAttrRule::Append => attrs.push(self.create_attr().into()),
            config::InjectAttrRule::Prepend => attrs.insert(0, self.create_attr().into()),
        }
    }
}

impl Clone for InjectAttr {
    fn clone(&self) -> Self {
        InjectAttr {
            cfg: self.cfg.clone(),
            value_ast_cache: None,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.cfg.clone_from(&source.cfg);
        self.value_ast_cache = None;
    }
}

impl From<config::InjectAttrConfig> for InjectAttr {
    fn from(cfg: config::InjectAttrConfig) -> Self {
        InjectAttr {
            cfg,
            value_ast_cache: None,
        }
    }
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
    /// import source regex pattern
    import: CachedRegex,
    /// target components
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

impl FoldJSXAttrs {
    fn flat_jsx_element_name(name: &JSXElementName) -> Option<(Id, Vec<String>)> {
        match name {
            JSXElementName::Ident(name) => Some((name.to_id(), vec![])),
            JSXElementName::JSXMemberExpr(name) => {
                let mut sub = vec![];
                let mut expr = name;
                loop {
                    sub.insert(0, expr.prop.sym.to_string());
                    match &expr.obj {
                        JSXObject::Ident(end) => break Some((end.to_id(), sub)),
                        JSXObject::JSXMemberExpr(next) => expr = next,
                    }
                }
            }
            _ => None,
        }
    }
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
        if !self.imported_refs.is_empty() {
            match FoldJSXAttrs::flat_jsx_element_name(&node.name) {
                Some((id, sub)) => {
                    for imp_ref in &mut self.imported_refs {
                        if (imp_ref.id == id) && (imp_ref.sub == sub) {
                            for attr in &mut imp_ref.attrs {
                                attr.inject_attr(&mut node.attrs);
                            }
                        }
                    }
                }
                _ => {}
            };
        }
        node.fold_children_with(self)
    }
}

pub fn transform(config: config::Config) -> impl Fold {
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
                        attrs: attrs.into_iter().map(|attr| attr.into()).collect(),
                    })
                    .collect(),
            })
            .collect(),
    }
}
