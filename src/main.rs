use std::{collections::{HashMap, HashSet}, str::Chars};

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

    fn nth_right_child(&mut self, n: usize) -> &mut Self {
        let mut result = self;
        for _ in 0..n {
            result = result.right.as_deref_mut().unwrap();
        }
        result
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
    let mut root = match chars.into() {
        NodeValueFromChars::NodeValue(root) => Tree::new(root),
        NodeValueFromChars::BracketOpen => process_formula(chars),
        _ => panic!()
    };

    loop {
        match chars.into() {
            NodeValueFromChars::End => {
                break;
            }

            NodeValueFromChars::BracketOpen => {
                let sub_tree = process_formula(chars);
                let mut current = &mut root;

                loop {
                    if current.right.is_some() {
                        current = current.right.as_mut().unwrap();
                    } else {
                        break;
                    }
                }

                current.right = Some(Box::new(sub_tree));
            }

            NodeValueFromChars::NodeValue(value) => {
                let mut depth = 0;
                let mut current = &root;

                loop {
                    if value > current.value {
                        depth += 1;
                        if let Some(right) = &current.right {
                            current = &right;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if depth == 0 {
                    root = Tree::new_with_left(value, root);
                } else {
                    let to_change = root.nth_right_child(depth - 1);
                    if let Some(old_right) = &to_change.right {
                        to_change.right = Some(Box::new(Tree::new_with_left(value, *old_right.to_owned())));
                    } else {
                        to_change.right = Some(Box::new(Tree::new(value)));
                    }
                }
            }
        }
    }
    root.is_sub_fn = true;
    root
}

fn print_tree(tree: Tree) {
    let mut result = 
r#"```mermaid
flowchart TB
"#.to_string();

    print_sub_tree(tree, &mut result, 0);
    result.push_str("```");

    println!("{}", result);
}

fn print_sub_tree(tree: Tree, result: &mut String, id: u8) -> u8 {
    let logic_char: char = (&tree.value).into();
    result.push_str(&format!("node{id}[\"{logic_char}\"]\n"));
    
    let left_size = if let Some(left) = tree.left.clone() {
        let left_size = print_sub_tree(*left, result, id + 1);
        result.push_str(&format!("node{} --> node{}\n", id, id + 1));
        left_size
    } else {0};
    
    if let Some(right) = tree.right {
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
    if let NodeValue::Var(c) = tree.value {
        result.insert(c);
    }
    if let Some(left) = &tree.left {
        get_variables_sub_tree(left, result)
    }
    if let Some(right) = &tree.right {
        get_variables_sub_tree(right, result)
    }
}

fn print_truth_table(tree: &Tree) {
    let mut variables: HashMap<char, bool> = get_variables(&tree).into_iter().map(|var| (var, false)).collect();

    let mut result = String::new();
    for variable in variables.keys() {
        result.push('|');
        result.push(*variable);
    }
    result.push_str("|res|\n|---");
    for _ in 0..variables.len() {
        result.push_str("|---");
    }

    for values in 0..(1_u8 << variables.len()) {
        result.push_str("|\n|");
        for _ in 0..variables.len() {
        }

        for (i, (_, value)) in variables.iter_mut().enumerate() {
            *value = ((values >> i) & 1) != 0;
            result.push_str(if *value {"<span style=\"color:green\">T</span>"} else {"<span style=\"color:red\">F</span>"});
            result.push('|');
        }

        let eq_res = calc_sub_tree(tree, &variables);
        result.push_str(if eq_res.unwrap() {"<span style=\"color:green\">T</span>"} else {"<span style=\"color:red\">F</span>"});
    }
    result.push('|');
    println!("{}", result);
}

fn calc_sub_tree(tree: &Tree, variables: &HashMap<char, bool>) -> Option<bool> {
    Some(match tree.value {
        NodeValue::Not => !calc_sub_tree(tree.right.as_ref()?, variables)?,
        NodeValue::And => calc_sub_tree(tree.left.as_ref()?, variables)? && calc_sub_tree(tree.right.as_ref()?, variables)?,
        NodeValue::Or => calc_sub_tree(tree.left.as_ref()?, variables)? || calc_sub_tree(tree.right.as_ref()?, variables)?,
        NodeValue::Xor => calc_sub_tree(tree.left.as_ref()?, variables)? ^ calc_sub_tree(tree.right.as_ref()?, variables)?,
        NodeValue::Impl => !calc_sub_tree(tree.left.as_ref()?, variables)? || calc_sub_tree(tree.right.as_ref()?, variables)?,
        NodeValue::Eq => calc_sub_tree(tree.left.as_ref()?, variables)? == calc_sub_tree(tree.right.as_ref()?, variables)?,
        NodeValue::Var(c) => *variables.get(&c)?,
    })
}

fn main() {
    print!(r#"!   ¬
&   ∧   and
V   ∨   or
^   ⊕   xor
->  →   implies
<-> ↔   equals
"#);
    let mut formula = String::new();
    std::io::stdin().read_line(&mut formula).expect("Cannot read input");
    
    let res = process_formula(&mut formula.trim().chars());
    print_truth_table(&res);
    print_tree(res);
}