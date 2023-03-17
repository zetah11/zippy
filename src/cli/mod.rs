mod diagnostic;
mod dot;
mod format;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;

use itertools::Itertools;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use zippy_common::messages::Messages;
use zippy_common::names::Name;
use zippy_common::source::Project;
use zippy_frontend::check::{get_bound, Template, Type};
use zippy_frontend::dependencies::get_dependencies;
use zippy_frontend::flattened::{
    Entry, Expression, ExpressionNode, Item, Module, Pattern, PatternNode,
};

use self::diagnostic::print_diagnostic;
use self::dot::GraphViz;
use crate::database::Database;
use crate::pretty::Prettier;
use crate::project;
use crate::project::{source_name_from_path, FsProject, DEFAULT_ROOT_NAME};

/// Perform checks on the project.
pub fn check(dot: bool) -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_module_level("salsa_2022", LevelFilter::Warn)
        .init()
        .unwrap();

    let cwd = std::env::current_dir()?;
    let mut database = Database::new();

    let project_name = cwd
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| DEFAULT_ROOT_NAME.to_string());
    let project = Project::new(&database, project_name);
    let project = FsProject::new(project).with_root(&cwd);

    let sources = project::get_project_sources(&cwd)
        .into_iter()
        .filter_map(|path| {
            let content = fs::read_to_string(&path).ok()?;
            let name = source_name_from_path(&database, Some(&project), &path);
            Some((path, name, content))
        })
        .collect();

    database.init_sources(sources);

    let mut messages = Vec::new();
    let prettier = Prettier::new(&database).with_full_name(false);
    let mut all_deps = HashMap::new();

    let mut context = HashMap::new();
    let mut constraints = Vec::new();

    for module in database.get_modules() {
        let dependencies = get_dependencies(&database, module);
        messages.extend(get_dependencies::accumulated::<Messages>(&database, module));

        for (name, depends) in dependencies.dependencies(&database) {
            assert!(all_deps.insert(*name, depends.clone()).is_none());
        }

        let bound = get_bound(&database, module);
        constraints.extend(bound.constraints(&database).iter().cloned());
        context.extend(
            bound
                .context(&database)
                .iter()
                .map(|(name, ty)| (*name, ty.clone())),
        );
    }

    for message in messages {
        print_diagnostic(&database, Some(&project), &prettier, message)?;
    }

    print_context(&database, &prettier, context);

    if dot {
        let graph = std::fs::File::create("dependencies.dot")?;
        let mut writer = std::io::BufWriter::new(graph);
        GraphViz::new(&database, &prettier, all_deps).render(&mut writer)?;
    }

    Ok(())
}

// hacky, bodgy pretty-printing of types below

fn print_context(db: &Database, prettier: &Prettier, context: HashMap<Name, Template>) {
    for (name, ty) in context {
        let name = prettier.pretty_name(name);
        let ty = pretty_ty(db, prettier, &ty.ty);

        println!("{name}: {ty}");
    }
}

fn pretty_ty(db: &Database, prettier: &Prettier, ty: &Type) -> String {
    match ty {
        Type::Trait { values } => {
            let values = values
                .iter()
                .map(|(name, ty)| {
                    let name = prettier.pretty_item_name(*name);
                    let ty = pretty_ty(db, prettier, &ty.ty);
                    format!("let {name}: {ty}")
                })
                .join("; ");
            format!("trait ({values})")
        }

        Type::Range {
            clusivity,
            lower,
            upper,
            module,
        } => {
            let kw = match (clusivity.includes_start, clusivity.includes_end) {
                (true, true) => "thru",
                (true, false) => "upto",
                (false, true) => "+ 1 thru",
                (false, false) => "+ 1 upto",
            };

            let lower = module.type_expression(db, lower);
            let upper = module.type_expression(db, upper);

            let lower = pretty_expr(db, module, prettier, lower);
            let upper = pretty_expr(db, module, prettier, upper);

            format!("{lower} {kw} {upper}")
        }

        Type::Var(_) => "<var>".to_string(),
        Type::Unit => "1".to_string(),
        Type::Invalid(_) => "<err>".to_string(),
    }
}

fn pretty_item(prettier: &Prettier, item: &Item) -> String {
    match item {
        Item::Let { pattern, .. } => {
            let pattern = pretty_pattern(prettier, pattern);
            format!("let {pattern}")
        }
    }
}

fn pretty_expr(db: &Database, module: &Module, prettier: &Prettier, expr: &Expression) -> String {
    match &expr.node {
        ExpressionNode::Entry(Entry { items, imports }) => {
            let imports = imports.iter().map(|index| {
                let import = module.import(db, index);

                let from = pretty_expr(db, module, prettier, &import.from);

                let names = import
                    .names
                    .iter()
                    .map(|name| {
                        let alias = name.alias.name;
                        let actual = name.name.name;
                        if actual != alias {
                            format!("{} as {}", actual.text(db), alias.text(db))
                        } else {
                            actual.text(db).clone()
                        }
                    })
                    .join("; ");

                format!("import {from}.({names})")
            });

            let items = items.iter().map(|index| {
                let item = module.item(db, index);
                pretty_item(prettier, item)
            });

            let children = imports.chain(items).join("; ");
            format!("entry ({children})")
        }

        ExpressionNode::Let { pattern, body, .. } => {
            let pattern = pretty_pattern(prettier, pattern);
            let body = body
                .as_ref()
                .map(|expr| format!("= {}", pretty_expr(db, module, prettier, expr)))
                .unwrap_or_default();
            format!("let {pattern} {body}")
        }

        ExpressionNode::Block(exprs, last) => {
            let exprs = exprs
                .iter()
                .chain(std::iter::once(&**last))
                .map(|expr| pretty_expr(db, module, prettier, expr))
                .join("; ");
            format!("({exprs})")
        }

        ExpressionNode::Annotate(expr, _) => pretty_expr(db, module, prettier, expr),
        ExpressionNode::Path(at, field) => format!(
            "{}.{}",
            pretty_expr(db, module, prettier, at),
            field.name.text(db)
        ),

        ExpressionNode::Alias(alias) => format!("<import {}>", alias.name.text(db)),
        ExpressionNode::Name(name) => prettier.pretty_name(*name),
        ExpressionNode::Number(number) => number.literal(db).clone(),
        ExpressionNode::String(string) => string.literal(db).clone(),
        ExpressionNode::Unit => "()".to_string(),
        ExpressionNode::Invalid(_) => "<err>".to_string(),
    }
}

fn pretty_pattern<N>(prettier: &Prettier, pattern: &Pattern<N>) -> String
where
    N: Copy + Into<Name>,
{
    match &pattern.node {
        PatternNode::Annotate(pattern, _) => pretty_pattern(prettier, pattern),
        PatternNode::Name(name) => prettier.pretty_name((*name).into()),
        PatternNode::Unit => "()".to_string(),
        PatternNode::Invalid(_) => "<err>".to_string(),
    }
}
