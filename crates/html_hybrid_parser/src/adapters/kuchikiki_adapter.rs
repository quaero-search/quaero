use kuchikiki::traits::*;
use std::borrow::Cow;

use crate::{Empty, Node, NodeResolvable, Parser, Query};

impl<'a> NodeResolvable<'a> for kuchikiki::NodeDataRef<kuchikiki::NodeData> {
    type Parser = Empty;
    type Node = kuchikiki::NodeRef;

    fn resolve(self, _parser: &'a Self::Parser) -> Self::Node {
        self.to_node()
    }
}

impl<'a> NodeResolvable<'a> for kuchikiki::NodeRef {
    type Parser = Empty;
    type Node = kuchikiki::NodeRef;

    fn resolve(self, _parser: &'a Self::Parser) -> Self::Node {
        self
    }
}

impl<'a> Query<'a> for kuchikiki::NodeRef {
    type Parser = Empty;
    type NodeResolvable = kuchikiki::NodeRef;
    type Node = kuchikiki::NodeRef;

    fn get_nodes(
        &'a self,
        _parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.descendants()
    }

    fn get_child_nodes(
        &'a self,
        _parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.children()
    }
}

impl<'a> Query<'a> for Parser<kuchikiki::NodeRef> {
    type Parser = Empty;
    type NodeResolvable = kuchikiki::NodeRef;
    type Node = kuchikiki::NodeRef;

    fn get_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.dom.get_nodes(parser)
    }

    fn get_child_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.dom.get_child_nodes(parser)
    }
}

impl<'a> Node<'a> for kuchikiki::NodeRef {
    fn class<'b>(&'b self) -> Option<Cow<'b, str>> {
        let element = self.as_element()?;
        let attributes = element.attributes.borrow();
        let class = attributes.get("class")?;
        Some(Cow::Owned(class.to_string()))
    }

    fn id(&'_ self) -> Option<std::borrow::Cow<'_, str>> {
        let element = self.as_element()?;
        let attributes = element.attributes.borrow();
        let id = attributes.get("id")?;
        Some(Cow::Owned(id.to_string()))
    }

    fn tag(&'_ self) -> Option<std::borrow::Cow<'_, str>> {
        let element = self.as_element()?;
        let tag = element.name.expanded();

        Some(Cow::Owned(tag.local.to_string()))
    }

    fn text(&'a self, _parser: &'a Self::Parser) -> Option<std::borrow::Cow<'a, str>> {
        let text = self.text_contents();
        if text.len() == 0 {
            None
        } else {
            Some(Cow::Owned(text))
        }
    }

    fn children_raw_text(&'a self, _parser: &'a Self::Parser) -> Option<std::borrow::Cow<'a, str>> {
        let mut string = String::new();

        for text_node in self.children().text_nodes() {
            string.push_str(&text_node.borrow())
        }

        if string.len() == 0 {
            None
        } else {
            Some(Cow::Owned(string))
        }
    }

    fn get_attribute(&'a self, key: &'a str) -> Option<std::borrow::Cow<'a, str>> {
        let Some(element) = self.as_element() else {
            return None;
        };
        element
            .attributes
            .borrow()
            .get(key)
            .map(|this| Cow::Owned(this.to_string()))
    }

    fn get_href(&'a self) -> Option<std::borrow::Cow<'a, str>> {
        let Some(element) = self.as_element() else {
            return None;
        };
        element
            .attributes
            .borrow()
            .get("href")
            .map(|this| Cow::Owned(this.to_string()))
    }
}

impl<'a> Query<'a> for kuchikiki::NodeDataRef<kuchikiki::NodeData> {
    type Parser = Empty;
    type NodeResolvable = kuchikiki::NodeRef;
    type Node = kuchikiki::NodeRef;

    fn get_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.clone().resolve(parser).descendants()
    }

    fn get_child_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.clone().resolve(parser).children()
    }
}
