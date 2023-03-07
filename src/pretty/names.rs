use zippy_common::names::{
    DeclarableName, ItemName, LocalName, Name, UnnamableName, UnnamableNameKind,
};

use super::Prettier;
use crate::span::{find_span_start, SpanStartInfo};

impl Prettier<'_> {
    pub fn pretty_name(&self, name: Name) -> String {
        match name {
            Name::Item(name) => self.pretty_item_name(name),
            Name::Local(name) => self.pretty_local_name(name),
        }
    }

    pub fn pretty_declarable_name(&self, name: DeclarableName) -> String {
        match name {
            DeclarableName::Item(name) => self.pretty_item_name(name),
            DeclarableName::Local(name) => self.pretty_local_name(name),
            DeclarableName::Unnamable(name) => self.pretty_unnamable_name(name),
        }
    }

    pub fn pretty_item_name(&self, name: ItemName) -> String {
        let this = name.name(self.db).text(self.db);

        if self.full_name {
            if let Some(parent) = name.parent(self.db) {
                return format!("{}.{this}", self.pretty_declarable_name(parent));
            }
        }

        this.clone()
    }

    pub fn pretty_local_name(&self, name: LocalName) -> String {
        let this = name.name(self.db).text(self.db);

        if self.full_name {
            if let Some(parent) = name.parent(self.db) {
                return format!("{}.{this}", self.pretty_declarable_name(parent));
            }
        }

        this.clone()
    }

    pub fn pretty_unnamable_name(&self, name: UnnamableName) -> String {
        let kind = match name.kind(self.db) {
            UnnamableNameKind::Entry => "entry",
            UnnamableNameKind::Lambda => "function",
            UnnamableNameKind::Pattern => "item",
        };

        if self.include_span {
            let span = name.span(self.db);
            let source = span.source.content(self.db);

            let SpanStartInfo { line, column, .. } = find_span_start(source, span);

            let name = self.db.get_source_path(span.source.name(self.db));
            match name.file_name() {
                Some(name) => format!("<{kind} in {}:{line}:{column}>", name.to_string_lossy()),
                None => format!("<{kind} in {}:{line}:{column}>", name.display()),
            }
        } else {
            format!("<{kind}>")
        }
    }
}
