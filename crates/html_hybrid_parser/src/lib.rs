//! Combines `tl` and `kuchikiki` into a unified API.

#![warn(missing_docs)]

use kuchikiki::traits::*;
use std::borrow::Cow;

mod adapters;

/// Used for when theres no seperate parser.
pub struct Empty;

/// A HTML Parser.
pub struct Parser<D> {
    /// The parsed DOM.
    pub dom: D,
}

impl<'a> Parser<tl::VDom<'a>> {
    /// This parser is fast but only supports a subset of the HTML spec.
    pub fn fast_but_constrained(data: &'a str) -> Parser<tl::VDom<'a>> {
        let dom: tl::VDom<'_> = tl::parse(data, tl::ParserOptions::default()).unwrap();

        Parser { dom }
    }

    /// Returns a reference to the underlying parser.
    pub fn parser(&'a self) -> &'a tl::Parser<'a> {
        self.dom.parser()
    }
}

impl<'a> Parser<kuchikiki::NodeRef> {
    /// This parser is slow but supports the entire HTML spec.
    pub fn comprehensive_but_slow(data: &'a str) -> Parser<kuchikiki::NodeRef> {
        let dom: kuchikiki::NodeRef = kuchikiki::parse_html().one(data);

        Parser { dom }
    }

    /// Returns a reference to the underlying parser.
    pub fn parser(&'a self) -> &'a Empty {
        &Empty
    }
}

/// A reference or handle to a node that can be resolved.
pub trait NodeResolvable<'a> {
    /// The underlying parser type.
    type Parser;
    /// The underlying parser type.
    type Node: Node<'a>;

    /// Turns the reference or handle into a node.
    fn resolve(self, parser: &'a Self::Parser) -> Self::Node;
}

/// A node from the tree.
pub trait Node<'a>: Query<'a> {
    /// Gets the node's class name.
    fn class(&'_ self) -> Option<Cow<'_, str>>;

    /// Gets the node's id.
    fn id(&'_ self) -> Option<Cow<'_, str>>;

    /// Gets the node's tag name.
    fn tag(&'_ self) -> Option<Cow<'_, str>>;

    /// Gets the node's inner text.
    fn text(&'a self, parser: &'a Self::Parser) -> Option<Cow<'a, str>>;

    /// Gets only the text from the node's direct raw text children.
    fn children_raw_text(&'a self, parser: &'a Self::Parser) -> Option<Cow<'a, str>>;

    /// Gets a specific attribute from the node.
    fn get_attribute(&'a self, key: &'a str) -> Option<Cow<'a, str>>;

    /// Gets the href attribute from the node.
    fn get_href(&'a self) -> Option<Cow<'a, str>>;
}

/// Queries that can be performed on a node or the parser.
pub trait Query<'a> {
    /// The underlying parser type.
    type Parser;
    /// The underlying node referance or handle type.
    type NodeResolvable: NodeResolvable<'a, Parser = Self::Parser, Node = Self::Node>;
    /// The underlying node type.
    type Node: Node<'a>;

    /// Gets all nodes.
    fn get_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a;

    /// Gets only the child nodes.
    fn get_child_nodes(
        &'a self,
        parser: &'a Self::Parser,
    ) -> impl Iterator<Item = Self::NodeResolvable> + 'a;

    /// Gets the first node which has the specified classes.
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

    /// Gets the first child node which has the specified classes.
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

    /// Gets all nodes which have the specified classes.
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

    /// Gets all child node which have the specified classes.
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

    /// Gets the first node which has the specified id.
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

    /// Gets the first child node which has the specified id.
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

    /// Gets all nodes which have the specified id.
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

    /// Gets all child nodes which have the specified id.
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

    /// Gets first node which has the specified tag.
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

    /// Gets first child node which has the specified tag.
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

    /// Gets all nodes which have the specified tag.
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

    /// Gets all child nodes node which have the specified tag.
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

/// The criteria the node's classes need to meet.
pub enum QueryClassNamesCriteria {
    /// The node must exclusively have the specific classes.
    Exact,
    /// The node must non-exclusively have the specific classes.
    All,
    /// The node must have at least one of the specific classes.
    Any,
}

/// A list of class names that can be queried
pub trait QueryClassNames<'a> {
    /// The amount of classes in this query.
    fn len(&self) -> usize;

    /// Returns true if the query has the specific class.
    fn has(&self, value: &str) -> bool;

    /// Gets the criteria for this query.
    fn criteria(&self) -> &QueryClassNamesCriteria;

    /// Performs the query.
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

/// A type for a query with multiple classes.
pub type ClassNames = (phf::Set<&'static str>, QueryClassNamesCriteria);
/// A type for a query with one class.
pub type ClassName = (&'static str, QueryClassNamesCriteria);

/// Constructs a class name query where the node must exclusively have the specific classes.
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

/// Constructs a class name query where the node must exclusively have the specific classes.
#[macro_export]
macro_rules! class_names_all {
    ( $name:literal ) => {
        ($name, $crate::QueryClassNamesCriteria::All)
    };

    ( $($name:literal),+ ) => {
        ($crate::phf_set! { $($name),+ }, $crate::QueryClassNamesCriteria::All)
    };
}

/// Constructs a class name query where the node must have at least one of the specific classes.
#[macro_export]
macro_rules! class_names_any {
    ( $name:literal ) => {
        ($name, $crate::QueryClassNamesCriteria::Any)
    };

    ( $($name:literal),+ ) => {
        ($crate::phf_set! { $($name),+ }, $crate::QueryClassNamesCriteria::Any)
    };
}
