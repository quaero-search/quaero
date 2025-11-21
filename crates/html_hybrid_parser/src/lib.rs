use kuchikiki::traits::*;
use std::borrow::Cow;

mod adapters;

pub struct Empty;

pub struct Parser<D> {
    pub dom: D,
}

impl<'a> Parser<tl::VDom<'a>> {
    pub fn fast_but_constrained(data: &'a str) -> Parser<tl::VDom<'a>> {
        let dom: tl::VDom<'_> = tl::parse(data, tl::ParserOptions::default()).unwrap();

        Parser { dom }
    }

    pub fn parser(&'a self) -> &'a tl::Parser<'a> {
        self.dom.parser()
    }
}

impl<'a> Parser<kuchikiki::NodeRef> {
    pub fn comprehensive_but_slow(data: &'a str) -> Parser<kuchikiki::NodeRef> {
        let dom: kuchikiki::NodeRef = kuchikiki::parse_html().one(data);

        Parser { dom }
    }

    pub fn parser(&'a self) -> &'a Empty {
        &Empty
    }
}

pub trait NodeResolvable<'a> {
    type Parser;
    type Node: Node<'a>;

    fn resolve(self, parser: &'a Self::Parser) -> Self::Node;
}

pub trait Node<'a>: Query<'a> {
    fn class(&'_ self) -> Option<Cow<'_, str>>;

    fn id(&'_ self) -> Option<Cow<'_, str>>;

    fn tag(&'_ self) -> Option<Cow<'_, str>>;

    fn text(&'a self, parser: &'a Self::Parser) -> Option<Cow<'a, str>>;

    fn children_raw_text(&'a self, parser: &'a Self::Parser) -> Option<Cow<'a, str>>;

    fn get_attribute(&'a self, key: &'a str) -> Option<Cow<'a, str>>;

    fn get_href(&'a self) -> Option<Cow<'a, str>>;
}

pub trait Query<'a> {
    type Parser;
    type NodeResolvable: NodeResolvable<'a, Parser = Self::Parser, Node = Self::Node>;
    type Node: Node<'a>;

    fn get_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a;

    fn get_child_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a;

    fn get_first_node_with_classes(
        &'a self,
        classes: &impl QueryClassNames<'a>,
        parser: &'a Self::Parser,
    ) -> Option<Self::Node> {
        self.get_nodes(parser).find_map(move |node| {
            let resolved_node = node.resolve(parser);
            if classes.matches(resolved_node.class()) {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_first_child_node_with_classes(
        &'a self,
        classes: &impl QueryClassNames<'a>,
        parser: &'a Self::Parser,
    ) -> Option<Self::Node> {
        self.get_child_nodes(parser).find_map(move |node| {
            let resolved_node = node.resolve(parser);
            if classes.matches(resolved_node.class()) {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_nodes_with_classes(
        &'a self,
        classes: &'a impl QueryClassNames<'a>,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::Node> + 'a {
        self.get_nodes(parser).filter_map(move |node| {
            let resolved_node = node.resolve(parser);
            if classes.matches(resolved_node.class()) {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_child_nodes_with_classes(
        &'a self,
        classes: &'a impl QueryClassNames<'a>,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::Node> + 'a {
        self.get_child_nodes(parser).filter_map(move |node| {
            let resolved_node = node.resolve(parser);
            if classes.matches(resolved_node.class()) {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_first_node_with_id(
        &'a self,
        id: &'a str,
        parser: &'a Self::Parser,
    ) -> Option<Self::Node> {
        self.get_nodes(parser).find_map(move |node| {
            let resolved_node = node.resolve(parser);
            let Some(node_id) = resolved_node.id() else {
                return None;
            };
            if id == node_id {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_first_child_node_with_id(
        &'a self,
        id: &'a str,
        parser: &'a Self::Parser,
    ) -> Option<Self::Node> {
        self.get_child_nodes(parser).find_map(move |node| {
            let resolved_node = node.resolve(parser);
            let Some(node_id) = resolved_node.id() else {
                return None;
            };
            if id == node_id {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_nodes_with_id(
        &'a self,
        id: &'a str,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::Node> + 'a {
        self.get_nodes(parser).filter_map(move |node| {
            let resolved_node = node.resolve(parser);
            let Some(node_id) = resolved_node.id() else {
                return None;
            };
            if id == node_id {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_child_nodes_with_id(
        &'a self,
        id: &'a str,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::Node> + 'a {
        self.get_child_nodes(parser).filter_map(move |node| {
            let resolved_node = node.resolve(parser);
            let Some(node_id) = resolved_node.id() else {
                return None;
            };
            if id == node_id {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_first_node_with_tag(
        &'a self,
        tag: &'a str,
        parser: &'a Self::Parser,
    ) -> Option<Self::Node> {
        self.get_nodes(parser).find_map(move |node| {
            let resolved_node = node.resolve(parser);
            let Some(node_tag) = resolved_node.tag() else {
                return None;
            };
            if tag == node_tag {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_first_child_node_with_tag(
        &'a self,
        tag: &'a str,
        parser: &'a Self::Parser,
    ) -> Option<Self::Node> {
        self.get_child_nodes(parser).find_map(move |node| {
            let resolved_node = node.resolve(parser);
            let Some(node_tag) = resolved_node.tag() else {
                return None;
            };
            if tag == node_tag {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_nodes_with_tag(
        &'a self,
        tag: &'a str,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::Node> + 'a {
        self.get_nodes(parser).filter_map(move |node| {
            let resolved_node = node.resolve(parser);
            let Some(node_tag) = resolved_node.tag() else {
                return None;
            };
            if tag == node_tag {
                Some(resolved_node)
            } else {
                None
            }
        })
    }

    fn get_child_nodes_with_tag(
        &'a self,
        tag: &'a str,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::Node> + 'a {
        self.get_child_nodes(parser).filter_map(move |node| {
            let resolved_node = node.resolve(parser);
            let Some(node_tag) = resolved_node.tag() else {
                return None;
            };
            if tag == node_tag {
                Some(resolved_node)
            } else {
                None
            }
        })
    }
}

pub enum QueryClassNamesCriteria {
    Exact,
    All,
    Any,
}

pub trait QueryClassNames<'a> {
    fn len(&self) -> usize;

    fn has(&self, value: &str) -> bool;

    fn criteria(&self) -> &QueryClassNamesCriteria;

    fn matches(&self, class: Option<Cow<str>>) -> bool {
        let Some(class) = class else { return false };

        match self.criteria() {
            QueryClassNamesCriteria::Exact => {
                let mut count = 0;
                for item in class.trim().split(" ") {
                    if !self.has(&item) {
                        return false;
                    }
                    count += 1;
                }
                count == self.len()
            }

            QueryClassNamesCriteria::All => {
                let mut count = 0;
                for item in class.trim().split(" ") {
                    if self.has(&item) {
                        count += 1
                    }
                }
                count == self.len()
            }

            QueryClassNamesCriteria::Any => {
                for item in class.trim().split(" ") {
                    if self.has(&item) {
                        return true;
                    }
                }
                false
            }
        }
    }
}

impl<'a> QueryClassNames<'a> for ClassNames {
    fn criteria(&self) -> &QueryClassNamesCriteria {
        &self.1
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn has(&self, value: &str) -> bool {
        self.0.get_key(value.trim()).is_some()
    }
}

impl<'a> QueryClassNames<'a> for ClassName {
    fn criteria(&self) -> &QueryClassNamesCriteria {
        &self.1
    }

    fn len(&self) -> usize {
        1
    }

    fn has(&self, value: &str) -> bool {
        &self.0 == &value.trim()
    }
}

pub use phf::phf_set;

pub type ClassNames = (phf::Set<&'static str>, QueryClassNamesCriteria);
pub type ClassName = (&'static str, QueryClassNamesCriteria);

#[macro_export]
macro_rules! class_names_exact {
    ( $name:literal ) => {
        // For one element `Any` has the same effect as `Exact` but its slightly faster.
        ($name, $crate::QueryClassNamesCriteria::Any)
    };

    ( $($name:literal),+ ) => {
        ($crate::phf_set! { $($name),+ }, $crate::QueryClassNamesCriteria::Exact)
    };
}

#[macro_export]
macro_rules! class_names_all {
    ( $name:literal ) => {
        ($name, $crate::QueryClassNamesCriteria::All)
    };

    ( $($name:literal),+ ) => {
        ($crate::phf_set! { $($name),+ }, $crate::QueryClassNamesCriteria::All)
    };
}

#[macro_export]
macro_rules! class_names_any {
    ( $name:literal ) => {
        ($name, $crate::QueryClassNamesCriteria::Any)
    };

    ( $($name:literal),+ ) => {
        ($crate::phf_set! { $($name),+ }, $crate::QueryClassNamesCriteria::Any)
    };
}
