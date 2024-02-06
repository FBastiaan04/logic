/*
formula = "!((!q ^ (p -> r)) ^ (r -> q))"

order = [
    ["!"],
    ["&", "V", "^"],
    ["->", "<->"]
]
*/

use std::str::Chars;

struct Tree {
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,

    value: NodeValue
}

impl Tree {
    fn new(value: NodeValue) -> Self {
        Self { left: None, right: None, value }
    }

    fn new_with_left(value: NodeValue, left: Tree) -> Self {
        Self { left: Some(Box::new(left)), right: None, value }
    }
}

#[derive(PartialEq, Eq)]
enum NodeValue {
    Not,
    And,
    Or,
    Xor,
    Impl,
    Eq,
    Var(char)
}

impl NodeValue {
    fn has_left(&self) -> bool {
        match self {
            Self::Not | Self::Var(_) => true,
            _ => false
        }
    }
}

impl PartialOrd for NodeValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        match *self {
            Self::Var(_) => {
                match *other {
                    Self::Var(_) => Equal,
                    _ => Less
                }
            }
            Self::Not => {
                match *other {
                    Self::Var(_) => Less,
                    Self::Not => Equal,
                    _ => Greater
                }
            }
            Self::And | Self::Or | Self::Xor => {
                match *other {
                    Self::Var(_) | Self::Not => Greater,
                    Self::And | Self::Or | Self::Xor => Equal,
                    Self::Impl | Self::Eq => Less
                }
            }
            Self::Impl | Self::Eq => {
                match *other {
                    Self::Var(_) | Self::Not => Greater,
                    Self::And | Self::Or | Self::Xor => Greater,
                    Self::Impl | Self::Eq => Equal
                }
            }
        }
    }
}

enum NodeValueFromChars {
    NodeValue(NodeValue),
    BracketOpen,
    End
}

impl From<&mut Chars<'_>> for NodeValueFromChars {
    fn from(value: &mut Chars<'_>) -> Self {
        match value.next() {
            Some(' ') => value.into(),
            Some('(') => Self::BracketOpen,
            Some(')') | None => Self::End,
            Some('!') => Self::NodeValue(NodeValue::Not),
            Some('&') => Self::NodeValue(NodeValue::And),
            Some('V') => Self::NodeValue(NodeValue::Or),
            Some('^') => Self::NodeValue(NodeValue::Xor),
            Some('-') => {
                value.next();
                Self::NodeValue(NodeValue::Impl)
            },
            Some('<') => {
                value.next();
                value.next();
                Self::NodeValue(NodeValue::Eq)
            },
            Some(c) => Self::NodeValue(NodeValue::Var(c))
        }
    }
}

fn process_formula(chars: &mut std::str::Chars) -> Tree {
    let mut root = Tree::new(if let NodeValueFromChars::NodeValue(root) = chars.into() {root} else {panic!()});
    let mut current = &root;

    loop {
        match chars.into() {
            NodeValueFromChars::End => {
                break
            }

            NodeValueFromChars::BracketOpen => {
                let sub_tree = process_formula(chars);
            }

            NodeValueFromChars::NodeValue(value) => {
                if value < root.value {
                    // If this operator has a lower priority, take over the root
                    root = Tree::new_with_left(value, root);
                    current = &root;
                } else {
                    if root.right.is_none() {
                        root.right = Some(Box::new(Tree::new(value)));
                        current = root.right;
                    } else {
                        if value < current.value {
                            root.right = Some(Box::new(Tree::new_with_left(value, current)));
                            current = root.right;
                        } else {
                            current.right = Some(Box::new(Tree::new(value)));
                            current = current.right;
                        }
                    }
                }
            }
        }
    }
    todo!()
}

fn main() {
    process_formula(&mut "!((!q ^ (p -> r)) ^ (r -> q))".chars());
}