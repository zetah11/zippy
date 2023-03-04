use zippy_common::invalid::Reason;
use zippy_common::messages::MessageMaker;
use zippy_common::names::RawName;
use zippy_common::source::Span;

use super::cst;
use crate::messages::ParseMessages;
use crate::{ast, Db};

pub enum ItemOrImports {
    Item(ast::Item),
    Imports(Vec<ast::Import>),
    Neither,
}

/// Produce an abstract syntax tree for a declarative-level binding
pub fn abstract_item(db: &dyn Db, item: cst::Item) -> ItemOrImports {
    let span = item.span;
    match item.node {
        cst::ItemNode::Let { pattern, body } => {
            let (pattern, anno) = extract_annotation(db, *pattern);
            let pattern = abstract_pattern(db, pattern);
            let body = body.map(|item| abstract_expression(db, *item));

            let item = ast::Item::Let {
                pattern,
                anno,
                body,
            };

            ItemOrImports::Item(item)
        }

        cst::ItemNode::Import(item) => ItemOrImports::Imports(abstract_import(db, *item)),

        cst::ItemNode::Invalid => ItemOrImports::Neither,
        _ => {
            MessageMaker::new(db, span).expected_item();
            ItemOrImports::Neither
        }
    }
}

/// An import should consist of some path-like thing, potentially ending in a
/// list of aliases. All of the following should be okay:
///
/// ```zippy
/// import Name
/// import Name.Inner
/// import Name.(Inner)
/// import Name.Beep.(boops, baaps, beeps)
/// import (some funky expression).hello
/// import (some funky expression).(boops, boops)
/// ```
fn abstract_import(db: &dyn Db, item: cst::Item) -> Vec<ast::Import> {
    fn imported_name(db: &dyn Db, item: cst::Item) -> Vec<ast::ImportedName> {
        let span = item.span;
        match item.node {
            cst::ItemNode::Group(items) => items
                .into_iter()
                .flat_map(|item| imported_name(db, item))
                .collect(),
            cst::ItemNode::Name(name) => {
                let name = make_name(db, span, name);
                let name = ast::ImportedName {
                    span,
                    name,
                    alias: name,
                };

                vec![name]
            }

            _ => {
                MessageMaker::new(db, span).expected_name();
                Vec::new()
            }
        }
    }

    let span = item.span;
    match item.node {
        cst::ItemNode::Group(items) => items
            .into_iter()
            .flat_map(|item| abstract_import(db, item))
            .collect(),

        cst::ItemNode::Path(expression, names) => {
            let names = imported_name(db, *names);
            let from = Some(abstract_expression(db, *expression));
            let import = ast::Import { from, names };

            vec![import]
        }

        node => {
            let names = imported_name(db, cst::Item { node, span });
            let import = ast::Import { from: None, names };

            vec![import]
        }
    }
}

fn abstract_type(db: &dyn Db, item: cst::Item) -> ast::Type {
    let span = item.span;
    let node = match item.node {
        cst::ItemNode::Number(number) => {
            let lower = ast::Expression {
                span,
                node: ast::ExpressionNode::Number("0".into()),
            };

            let upper = ast::Expression {
                span,
                node: ast::ExpressionNode::Number(number),
            };

            ast::TypeNode::Range {
                clusivity: ast::Clusivity::exclusive(),
                lower,
                upper,
            }
        }

        cst::ItemNode::Group(mut items) if items.len() == 1 => {
            return abstract_type(db, items.pop().unwrap())
        }

        cst::ItemNode::Invalid => ast::TypeNode::Invalid(Reason::SyntaxError),

        _ => {
            MessageMaker::new(db, span).expected_type();
            ast::TypeNode::Invalid(Reason::SyntaxError)
        }
    };

    ast::Type { span, node }
}

fn abstract_expression(db: &dyn Db, item: cst::Item) -> ast::Expression {
    let span = item.span;
    let node = match item.node {
        cst::ItemNode::Annotation(expr, ty) => {
            let expr = Box::new(abstract_expression(db, *expr));
            let ty = Box::new(abstract_type(db, *ty));
            ast::ExpressionNode::Annotate(expr, ty)
        }

        cst::ItemNode::Path(expr, field) => {
            let expr = Box::new(abstract_expression(db, *expr));

            let field_span = field.span;
            match abstract_name(db, *field) {
                Some(name) => ast::ExpressionNode::Path(expr, name),
                None => {
                    MessageMaker::new(db, field_span).expected_name();
                    ast::ExpressionNode::Invalid(Reason::SyntaxError)
                }
            }
        }

        cst::ItemNode::Group(mut items) => match items.len() {
            0 => ast::ExpressionNode::Unit,
            1 => return abstract_expression(db, items.pop().unwrap()),
            _ => ast::ExpressionNode::Block(
                items
                    .into_iter()
                    .map(|item| abstract_expression(db, item))
                    .collect(),
            ),
        },

        cst::ItemNode::Name(name) => ast::ExpressionNode::Name(make_name(db, span, name)),
        cst::ItemNode::Number(number) => ast::ExpressionNode::Number(number),
        cst::ItemNode::String(string) => ast::ExpressionNode::String(string),
        cst::ItemNode::Invalid => ast::ExpressionNode::Invalid(Reason::SyntaxError),

        _ => {
            MessageMaker::new(db, span).expected_expression();
            ast::ExpressionNode::Invalid(Reason::SyntaxError)
        }
    };

    ast::Expression { span, node }
}

fn abstract_pattern(db: &dyn Db, item: cst::Item) -> ast::Pattern {
    let span = item.span;
    let node = match item.node {
        cst::ItemNode::Annotation(pat, ty) => {
            let pat = Box::new(abstract_pattern(db, *pat));
            let ty = abstract_type(db, *ty);
            ast::PatternNode::Annotate(pat, ty)
        }

        cst::ItemNode::Group(mut items) => match items.len() {
            0 => ast::PatternNode::Unit,
            1 => return abstract_pattern(db, items.pop().unwrap()),
            _ => {
                MessageMaker::new(db, span).expected_pattern();
                ast::PatternNode::Invalid(Reason::SyntaxError)
            }
        },

        cst::ItemNode::Name(name) => ast::PatternNode::Name(make_name(db, span, name)),
        cst::ItemNode::Invalid => ast::PatternNode::Invalid(Reason::SyntaxError),

        _ => {
            MessageMaker::new(db, span).expected_pattern();
            ast::PatternNode::Invalid(Reason::SyntaxError)
        }
    };

    ast::Pattern { span, node }
}

/// Attempt to produce a name from this item, or return `None` if it is
/// something else.
fn abstract_name(db: &dyn Db, item: cst::Item) -> Option<ast::Identifier> {
    let span = item.span;
    match item.node {
        cst::ItemNode::Name(name) => Some(make_name(db, span, name)),
        cst::ItemNode::Group(mut items) if items.len() == 1 => {
            abstract_name(db, items.pop().unwrap())
        }

        _ => None,
    }
}

/// "Strip" an item off its type annotation, if any.
fn extract_annotation(db: &dyn Db, item: cst::Item) -> (cst::Item, Option<ast::Type>) {
    let span = item.span;
    match item.node {
        cst::ItemNode::Annotation(right, ty) => {
            let ty = abstract_type(db, *ty);
            (*right, Some(ty))
        }

        cst::ItemNode::Group(mut items) if items.len() == 1 => {
            extract_annotation(db, items.pop().unwrap())
        }

        node => (cst::Item { span, node }, None),
    }
}

/// Create an identifier from the given name and span.
fn make_name(db: &dyn Db, span: Span, name: String) -> ast::Identifier {
    let db = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);
    let name = RawName::new(db, name);
    ast::Identifier { span, name }
}
