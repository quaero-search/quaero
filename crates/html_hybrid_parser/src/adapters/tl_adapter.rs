use std::borrow::Cow;

use crate::{Node, NodeResolvable, Parser, Query};

impl<'a> NodeResolvable<'a> for &'a tl::Node<'a> {
    type Parser = tl::Parser<'a>;
    type Node = &'a tl::Node<'a>;

    #[inline(always)]
    fn resolve(self, _parser: &'a Self::Parser) -> Self::Node {
        self
    }
}

impl<'a> Query<'a> for tl::VDom<'_> {
    type Parser = tl::Parser<'a>;
    type NodeResolvable = &'a tl::Node<'a>;
    type Node = &'a tl::Node<'a>;

    fn get_nodes(
        &'a self,
        _parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.nodes().iter()
    }

    fn get_child_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.children().iter().filter_map(|this| this.get(parser))
    }
}

impl<'a> Query<'a> for Parser<tl::VDom<'a>> {
    type Parser = tl::Parser<'a>;
    type NodeResolvable = &'a tl::Node<'a>;
    type Node = &'a tl::Node<'a>;

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

impl<'a> Node<'a> for &'a tl::Node<'a> {
    fn class(&'_ self) -> Option<Cow<'_, str>> {
        let Some(tag) = self.as_tag() else {
            return None;
        };
        tag.attributes().class().map(tl::Bytes::as_utf8_str)
    }

    fn id(&'_ self) -> Option<Cow<'_, str>> {
        let Some(tag) = self.as_tag() else {
            return None;
        };
        tag.attributes().id().map(|this| this.as_utf8_str())
    }

    fn tag(&'_ self) -> Option<Cow<'_, str>> {
        let Some(tag) = self.as_tag() else {
            return None;
        };
        Some(tag.name().as_utf8_str())
    }

    fn text(&'a self, parser: &'a Self::Parser) -> Option<Cow<'a, str>> {
        let text = tl::Node::inner_text(self, parser);
        if text.len() == 0 { None } else { Some(text) }
    }

    fn children_raw_text(&'a self, parser: &'a Self::Parser) -> Option<Cow<'a, str>> {
        let Some(children) = self.children() else {
            return None;
        };

        let mut string = String::new();

        for child in children.top().iter() {
            let Some(child) = child.get(parser) else {
                continue;
            };
            let Some(child) = child.as_raw() else {
                continue;
            };
            string.push_str(child.as_utf8_str().as_ref());
        }

        if string.len() == 0 {
            None
        } else {
            Some(Cow::Owned(string))
        }
    }

    fn get_attribute(&'a self, key: &'a str) -> Option<Cow<'a, str>> {
        let Some(tag) = self.as_tag() else {
            return None;
        };
        tag.attributes()
            .get(key)
            .flatten()
            .map(move |this| this.as_utf8_str())
    }

    fn get_href(&'_ self) -> Option<Cow<'_, str>> {
        self.get_attribute("href")
            .map(|this| Cow::Owned(html_escape::decode_html_entities(this.as_ref()).to_string()))
    }
}

impl<'a> Query<'a> for &'a tl::Node<'a> {
    type Parser = tl::Parser<'a>;
    type NodeResolvable = &'a tl::Node<'a>;
    type Node = &'a tl::Node<'a>;

    fn get_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.children()
            .into_iter()
            .flat_map(|this| this.all(parser).iter())
    }

    fn get_child_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a {
        self.children()
            .into_iter()
            .flat_map(|this| this.top().iter().filter_map(|this| this.get(parser)))
    }
}
