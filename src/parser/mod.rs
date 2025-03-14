use crate::{
    ast::types::{ASTBody, ASTNode, ASTProp, PropType},
    iter::StringIter,
};

#[derive(Debug)]
enum ParseState {
    None,
    Tag,
    Props,
    Body,
    ClosingTag,
}

pub fn start_parse(contents: String) -> ASTNode {
    let mut iter = StringIter::from(&contents);
    parse(&mut iter, None, 0)
}

pub fn parse(iter: &mut StringIter, id: Option<usize>, mut current_id: usize) -> ASTNode {
    let mut parse_state = ParseState::None;
    let mut tag = ASTNode {
        id: current_id,
        parent_id: id,
        node_type: None,

        name: String::new(),
        children: Vec::new(),
        props: Vec::new(),
    };

    let mut buffer = String::new();
    let mut closing_tag = String::new();
    while let Some(char) = iter.next() {
        //println!("{}", char);
        match char {
            '<' => match parse_state {
                ParseState::None => parse_state = ParseState::Tag,
                ParseState::Body => {
                    buffer = buffer.trim().to_owned();
                    if !buffer.is_empty() {
                        tag.children.push(ASTBody::String(buffer.clone()));
                        buffer.clear();
                    }
                    if let Some('/') = iter.peek() {
                        iter.next(); // Consume '/'
                        parse_state = ParseState::ClosingTag;
                    } else {
                        current_id += 1;

                        iter.step_back();
                        tag.children.push(ASTBody::Tag(Box::new(parse(
                            iter,
                            Some(tag.id),
                            current_id,
                        ))));
                    }
                }
                _ => panic!("Unexpected `<` tag"),
            },
            '>' => match parse_state {
                ParseState::Props | ParseState::Tag => parse_state = ParseState::Body,
                ParseState::ClosingTag => {
                    if tag.name != closing_tag {
                        panic!(
                            "Unexpected closing tag: </{}>. Expected </{}>",
                            closing_tag, tag.name
                        );
                    }
                    return tag;
                }

                _ => panic!("Unexpected `>` tag"),
            },
            '/' => {
                if let Some('>') = iter.peek() {
                    iter.next();
                    return tag;
                }
            }
            char if !char.is_whitespace() => match parse_state {
                ParseState::Tag => tag.name.push(char),
                ParseState::Props => tag.props.push(process_prop(iter, tag.props.len())),
                ParseState::ClosingTag => closing_tag.push(char),
                ParseState::Body => buffer.push(char),
                _ => panic!("Unexpected literal"),
            },
            char if char.is_whitespace() => match parse_state {
                ParseState::Tag => parse_state = ParseState::Props,
                ParseState::Body => buffer.push(char),
                _ => {}
            },
            _ => {}
        }
    }

    panic!("Unexpected EOF");
}

#[derive(Debug)]
enum PropParseState {
    Name,
    Value,
    Eq,
}

enum PropValueType {
    None,
    Literal,
    Var,
}

fn process_prop(iter: &mut StringIter, prop_id: usize) -> ASTProp {
    iter.step_back();
    let mut parse_state = PropParseState::Name;
    let mut prop = ASTProp {
        id: prop_id,
        name: String::new(),
        value: None,
    };

    let mut buffer = String::new();
    let mut value_type = PropValueType::None;
    while let Some(char) = iter.peek() {
        match char {
            '=' => match parse_state {
                PropParseState::Name => {
                    parse_state = PropParseState::Eq;
                    iter.next();
                }
                _ => panic!("Unexpected `=`"),
            },
            '"' => match parse_state {
                PropParseState::Eq => {
                    value_type = PropValueType::Literal;
                    parse_state = PropParseState::Value;
                    iter.next();
                }
                PropParseState::Value => match value_type {
                    PropValueType::Literal => {
                        parse_state = PropParseState::Name;
                        iter.next();
                    }
                    _ => panic!("Unexpected `\"` "),
                },
                _ => panic!("Unexpected `\"`"),
            },
            '{' => match parse_state {
                PropParseState::Eq => {
                    value_type = PropValueType::Var;
                    parse_state = PropParseState::Value;
                    iter.next();
                }
                _ => panic!("Unexpected `{{` "),
            },
            '}' => match parse_state {
                PropParseState::Value => match value_type {
                    PropValueType::Var => {
                        parse_state = PropParseState::Name;
                        iter.next();
                    }
                    _ => panic!("Unexpected `}}` "),
                },
                _ => panic!("Unexpected `}}` "),
            },
            '>' | '/' => match parse_state {
                PropParseState::Name => break,
                _ => panic!("Unexpected closing"),
            },
            char if !char.is_whitespace() => {
                if let Some(next) = iter.next() {
                    match parse_state {
                        PropParseState::Name => prop.name.push(next),
                        PropParseState::Value => buffer.push(next),
                        _ => panic!("Unexpected literal"),
                    }
                }
            }
            _ => match parse_state {
                PropParseState::Name => break,
                PropParseState::Value => {
                    if let Some(next) = iter.next() {
                        buffer.push(next)
                    }
                }
                _ => {
                    iter.next();
                }
            },
        }
    }
    prop.value = match value_type {
        PropValueType::None => None,
        PropValueType::Literal => Some(PropType::Literal(buffer)),
        PropValueType::Var => Some(PropType::Var(buffer)),
    };

    prop
}
