/*
formula = "!((!q ^ (p -> r)) ^ (r -> q))"

order = [
    ["!"],
    ["&", "V", "^"],
    ["->", "<->"]
]
*/

use std::{collections::HashSet, str::Chars};

#[derive(Clone, Debug)]
struct Tree {
    is_sub_fn: bool,
    value: NodeValue,
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
}

impl Tree {
    fn new(value: NodeValue) -> Self {
        Self { is_sub_fn: false, left: None, right: None, value }
    }

    fn new_with_left(value: NodeValue, left: Tree) -> Self {
        Self { is_sub_fn: false, left: Some(Box::new(left)), right: None, value }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum NodeValue {
    Not,
    And,
    Or,
    Xor,
    Impl,
    Eq,
    Var(char)
}

impl From<&NodeValue> for char {
    fn from(value: &NodeValue) -> Self {
        match *value {
            NodeValue::Not => '¬',
            NodeValue::And => '∧',
            NodeValue::Or => '∨',
            NodeValue::Xor => '⊕',
            NodeValue::Impl => '→',
            NodeValue::Eq => '↔',
            NodeValue::Var(c) => c,
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
                    _ => Greater
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
                    Self::Var(_) | Self::Not => Less,
                    Self::And | Self::Or | Self::Xor => Equal,
                    Self::Impl | Self::Eq => Greater
                }
            }
            Self::Impl | Self::Eq => {
                match *other {
                    Self::Impl | Self::Eq => Equal,
                    _ => Less
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
    println!("Sub function started with {}", chars.clone().collect::<String>());
    let mut root = match chars.into() {
        NodeValueFromChars::NodeValue(root) => Tree::new(root),
        NodeValueFromChars::BracketOpen => process_formula(chars),
        _ => panic!()
    };
    let mut current: Option<&mut Tree> = None;

    loop {
        match chars.into() {
            NodeValueFromChars::End => {
                println!("Sub function ended {}", chars.clone().collect::<String>());
                break;
            }

            NodeValueFromChars::BracketOpen => {
                let sub_tree = process_formula(chars);
                println!("Sub function stored");
                if let Some(some_current) = current {
                    some_current.right = Some(Box::new(sub_tree));
                    current = None;
                } else {
                    root.right = Some(Box::new(sub_tree));
                    current = Some(root.right.as_mut().unwrap());
                }
            }

            NodeValueFromChars::NodeValue(value) => {
                if value < root.value || root.is_sub_fn {
                    // If this operator has a lower priority, take over the root
                    println!("{} < {}", char::from(&value), char::from(&root.value));
                    root = Tree::new_with_left(value, root);
                    current = None;
                } else {
                    if let Some(some_current) = current {
                        if value < some_current.value || root.is_sub_fn {
                            root.right = Some(Box::new(Tree::new_with_left(value, some_current.clone())));
                            current = None;
                        } else {
                            some_current.right = Some(Box::new(Tree::new(value)));
                            current = None;
                        }
                    } else {
                        root.right = Some(Box::new(Tree::new(value)));
                        current = Some(root.right.as_mut().unwrap());
                    }
                }
            }
        }
    }
    println!("Sub function returned");
    root.is_sub_fn = true;
    root
}

fn print_tree(tree: Tree) {
    let mut result = 
r#"```mermaid
flowchart TB
"#.to_string();

    print_sub_tree(tree, &mut result, 0);

    println!("{}", result);
}

fn print_sub_tree(tree: Tree, result: &mut String, id: u8) -> u8 {
    let logic_char: char = (&tree.value).into();
    result.push_str(&format!("node{id}[\"{logic_char}\"]\n"));
    
    let left_size = if let Some(left) = tree.left.clone() {
        println!("Has left");
        let left_size = print_sub_tree(*left, result, id + 1);
        result.push_str(&format!("node{} --> node{}\n", id, id + 1));
        left_size
    } else {0};
    
    if let Some(right) = tree.right {
        println!("Has right");
        let right_size = print_sub_tree(*right, result, id + 1 + left_size);
        result.push_str(&format!("node{} --> node{}\n", id, id + 1 + left_size));
        left_size + right_size + 1
    } else {left_size + 1}
}

fn get_variables(tree: &Tree) -> HashSet<char> {
    let mut result = HashSet::new();
    get_variables_sub_tree(tree, &mut result);
    result
}

fn get_variables_sub_tree(tree: &Tree, result: &mut HashSet<char>) {
    
}

fn print_truth_table(tree: Tree) {

}

fn main() {
    let res = process_formula(&mut "!((!q & (p -> r)) & (r -> q))".chars());
    println!("{:#?}", res);
    print_tree(res);
}