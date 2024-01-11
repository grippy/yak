#![allow(dead_code)]
#![allow(unused_imports)]

mod test;

pub mod token;
use regex::Regex;
use token::TokenType;

#[cfg(not(test))]
use log::{debug, info};
#[cfg(test)]
use std::{println as debug, println as info};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    // TODO: disable Span (need to rework this)
    // pub span: Span,
    pub ty: TokenType,
}

#[derive(Debug, Clone)]
pub struct Span {
    // line number
    ln: usize,
    // char pos in the raw src
    pos: usize,
    // start column
    start: usize,
    // end column
    end: usize,
}

// literal
const COMMENT_PATTERN: &str = r"^#(.*)$";
const NUMBER_PATTERN: &str = r"^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)$";

//
// Identity patterns
//

// Package identities
const PACKAGE_PATTERN: &str = r"^[a-z]([a-z0-9_]*(\.{1}[a-z0-9_]+)*)$";
const PACKAGE_FUNC_PATTERN: &str =
    r"^([a-z]([a-z0-9_]*(\.{1}[a-z0-9_]+)*))?(\:[a-z]([a-zA-Z0-9_])*?)$";
const PACKAGE_TYPE_PATTERN: &str =
    r"^([a-z]([a-z0-9_]*(\.{1}[a-z0-9_]+)*)\.)?([A-Z]([a-zA-Z0-9_])*?)$";
const PACKAGE_TRAIT_PATTERN: &str =
    r"^([a-z]([a-z0-9_]*(\.{1}[a-z0-9_]+)*))?(\^[A-Z]([a-zA-Z0-9_])*?)$";
const PACKAGE_VAR_PATTERN: &str =
    r"^([a-z]([a-z0-9_]*(\.{1}[a-z0-9_]+)*)\.)?([a-z_]([a-zA-Z0-9_])*?)$";

// Type Identities
const TYPE_PATTERN: &str = r"^([A-Z]([a-zA-Z0-9_])*?)$";

// Trait Identities
const TRAIT_PATTERN: &str = r"^(\^[A-Z]([a-zA-Z0-9_])*?)$";

// Variable Identities and variable with package prefixes are ambiguous
const VAR_PATTERN: &str = r"^([a-z_]([a-zA-Z0-9_])*?)$";

// Function Identities
const FUNC_PATTERN: &str = r"^(\:[a-z]([a-zA-Z0-9_])*?)$";

static ASCII_LOWER: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

static ASCII_UPPER: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

static ASCII_DIGIT: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[derive(Debug)]
struct PatternMatcher {
    comment_id: Regex,
    package_id: Regex,
    package_func_id: Regex,
    package_type_id: Regex,
    package_trait_id: Regex,
    package_var_id: Regex,
    var_id: Regex,
    func_id: Regex,
    type_id: Regex,
    trait_id: Regex,
    num_literal: Regex,
}

impl PatternMatcher {
    fn new() -> Self {
        PatternMatcher {
            comment_id: Regex::new(COMMENT_PATTERN).unwrap(),
            package_id: Regex::new(PACKAGE_PATTERN).unwrap(),
            package_func_id: Regex::new(PACKAGE_FUNC_PATTERN).unwrap(),
            package_type_id: Regex::new(PACKAGE_TYPE_PATTERN).unwrap(),
            package_trait_id: Regex::new(PACKAGE_TRAIT_PATTERN).unwrap(),
            package_var_id: Regex::new(PACKAGE_VAR_PATTERN).unwrap(),
            var_id: Regex::new(VAR_PATTERN).unwrap(),
            func_id: Regex::new(FUNC_PATTERN).unwrap(),
            type_id: Regex::new(TYPE_PATTERN).unwrap(),
            trait_id: Regex::new(TRAIT_PATTERN).unwrap(),
            num_literal: Regex::new(NUMBER_PATTERN).unwrap(),
        }
    }

    fn resolve(&self, this: &str) -> Option<TokenType> {
        // Comment
        // example: # this a comment
        if self.comment_id.is_match(this) {
            return Some(TokenType::Comment(this.to_owned()));
        }

        // Variable
        // example: _mything1IsGood_123
        if self.var_id.is_match(&this) {
            return Some(TokenType::IdVar(this.to_owned()));
        }
        // This is ambiguous with packages
        // with the exception when the variable starts with
        // an underscore... For consistency, resolve as a package
        // and we can figure out later if it's package + variable or not
        if this.contains("._") {
            if self.package_var_id.is_match(&this) {
                return Some(TokenType::IdVar(this.to_owned()));
            }
        }

        // Function
        // example: :func1
        if self.func_id.is_match(&this) {
            return Some(TokenType::IdFunc(this.to_owned()));
        }
        // example: my.pkg:func1
        if self.package_func_id.is_match(&this) {
            return Some(TokenType::IdFunc(this.to_owned()));
        }

        // Trait
        // example: ^MyType_1
        if self.trait_id.is_match(&this) {
            return Some(TokenType::IdTrait(this.to_owned()));
        }
        // example: my.pkg^MyType_1
        if self.package_trait_id.is_match(&this) {
            return Some(TokenType::IdTrait(this.to_owned()));
        }

        // Type
        // example: MyType_1
        if self.type_id.is_match(&this) {
            return Some(TokenType::IdType(this.to_owned()));
        }
        // example: my.pkg.MyType_1
        if self.package_type_id.is_match(&this) {
            return Some(TokenType::IdType(this.to_owned()));
        }

        // Package
        // single-word packages will resolve as Token::VarId
        // example: my.package.rules
        if self.package_id.is_match(&this) {
            return Some(TokenType::IdPackage(this.to_owned()));
        }

        // Number
        // example: .1 1 100 -100
        if self.num_literal.is_match(&this) {
            return Some(TokenType::LitNumber(this.to_owned()));
        }

        return None;
    }
}

#[derive(Debug)]
pub struct Lexer<'a> {
    pub raw: &'a str,
    pub tokens: Vec<Token>,
    stack: Vec<char>,
    matcher: PatternMatcher,
}

impl<'a> Lexer<'a> {
    pub fn from_source(source: &'a str) -> Self {
        Lexer {
            raw: source,
            stack: vec![],
            tokens: vec![],
            matcher: PatternMatcher::new(),
        }
    }

    pub fn tokens_as_stack(&self) -> Vec<Token> {
        let mut stack = vec![];
        let mut tokens = self.tokens.clone();
        tokens.reverse();
        for token in tokens {
            stack.push(token);
        }
        stack
    }

    #[allow(unused_assignments)]
    pub fn parse(&mut self) {
        // convert raw to stack
        let mut chars: Vec<char> = self.raw.chars().collect();
        chars.reverse();
        for c in chars {
            self.stack.push(c);
        }

        // TODO: make some of these instance variables
        // so we can clean up having to pass all these around
        let mut prev = '\n';
        let mut indent: usize = 0;
        let mut indent_on: bool = true;
        let mut quote_on = false;
        let mut buf: Vec<char> = vec![];

        // the char position in the raw string
        let mut pos = 0usize;
        // the line number of the char
        let mut ln = 1usize;
        // the column of the char
        let mut col = 1usize;

        while let Some(this) = self.stack.pop() {
            pos += 1; // this prob needs to be after we do any work?
            col += 1;

            match this {
                ' ' => {
                    // we have a word boundary or indent
                    if indent_on {
                        indent += 1;
                        prev = this;
                        continue;
                    }
                    if quote_on {
                        buf.push(this);
                    } else {
                        self.buf_to_token(&mut buf, true, pos, ln, col);
                        pos += 1;
                        col += 1;
                        self.push_token(TokenType::Sp, pos, ln, col);
                    }
                }
                '\n' => {
                    if quote_on {
                        buf.push(this);
                    } else {
                        indent = 0;
                        indent_on = true;
                        // check if the buf matches anything... should flush
                        // the buf only if not in the middle of a quote
                        self.buf_to_token(&mut buf, !quote_on, pos, ln, col);
                        self.push_token(TokenType::NL, pos, ln, col);
                        col = 0;
                    }
                    ln += 1;
                }
                '"' => {
                    if !quote_on {
                        // flush buffer here
                        self.buf_to_token(&mut buf, true, pos, ln, col);
                        quote_on = true;
                        buf.push(this);
                    } else {
                        // quote on... check if
                        // prev is an escape char
                        if prev == '\\' {
                            buf.push(this);
                        } else {
                            quote_on = false;
                            buf.push(this);
                            let token_type = TokenType::LitString(String::from_iter(buf.clone()));
                            self.push_token(token_type, pos, ln, col);
                            buf.clear();
                        }
                    }
                }
                '#' => {
                    buf.push(this);
                    if quote_on {
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    while let Some(next) = self.stack.pop() {
                        if next == '\n' {
                            self.stack.push(next);
                            break;
                        }
                        pos += 1;
                        col += 1;
                        buf.push(next);
                        prev = next;
                    }

                    self.buf_to_token(&mut buf, true, pos, ln, col);
                    continue;
                }
                '=' => {
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match: = == =>
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "==" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpEqEq, pos, ln, col);
                                prev = next;
                            }
                            "=>" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::PunctFatArrow, pos, ln, col);
                                prev = next;
                            }
                            _ => {
                                // put next back on the stack
                                self.stack.push(next);
                                self.push_token(TokenType::OpAssignEq, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpAssignEq, pos, ln, col);
                    }
                }
                '!' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match ! !=
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "!=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpNotEq, pos, ln, col);
                                prev = next;
                            }
                            "!(" => {
                                // unary expression
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpUnaryNot, pos, ln, col);
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::PunctParenL, pos, ln, col);
                                prev = next;
                            }
                            _ => {
                                self.stack.push(next);
                                self.push_token(TokenType::PunctExclamation, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::PunctExclamation, pos, ln, col);
                    }
                }
                '>' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match > >= >> >>=
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];

                        match sym {
                            ">=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpGte, pos, ln, col);
                                prev = next;
                            }
                            ">>" => {
                                // >>=
                                if self.stack.last().map_or(false, |next| next.eq(&'=')) {
                                    if let Some(next) = self.stack.pop() {
                                        pos += 1;
                                        col += 1;
                                        self.push_token(
                                            TokenType::OpAssignBitwiseShiftR,
                                            pos,
                                            ln,
                                            col,
                                        );
                                        prev = next;
                                    }
                                } else {
                                    self.push_token(TokenType::OpBitwiseShiftR, pos, ln, col);
                                    prev = next;
                                }
                            }
                            _ => {
                                self.stack.push(next);
                                self.push_token(TokenType::OpGt, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpGt, pos, ln, col);
                    }
                }
                '<' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match < <= << <<=
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "<=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpLte, pos, ln, col);
                                prev = next;
                            }
                            "<<" => {
                                // >>=
                                if self.stack.last().map_or(false, |next| next.eq(&'=')) {
                                    if let Some(next) = self.stack.pop() {
                                        pos += 1;
                                        col += 1;
                                        self.push_token(
                                            TokenType::OpAssignBitwiseShiftL,
                                            pos,
                                            ln,
                                            col,
                                        );
                                        prev = next;
                                    }
                                } else {
                                    self.push_token(TokenType::OpBitwiseShiftL, pos, ln, col);
                                    prev = next;
                                }
                            }
                            _ => {
                                self.stack.push(next);
                                self.push_token(TokenType::OpLt, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpLt, pos, ln, col);
                    }
                }
                '+' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match: + +=
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "+=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpAssignAdd, pos, ln, col);
                                prev = next;
                            }
                            "+(" => {
                                // unary expression
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpUnaryPlus, pos, ln, col);
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::PunctParenL, pos, ln, col);
                                prev = next;
                            }
                            _ => {
                                // check if "+" followed by a digit or decimal
                                if !ASCII_DIGIT.contains(&next) && next != '.' {
                                    self.stack.push(next);
                                    self.push_token(TokenType::OpAdd, pos, ln, col);
                                } else {
                                    pos += 1;
                                    col += 1;
                                    buf.push(this);
                                    buf.push(next);
                                    prev = next;
                                }
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpAdd, pos, ln, col);
                    }
                }
                '-' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match: - -=
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "-=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpAssignSub, pos, ln, col);
                                prev = next;
                            }
                            "-(" => {
                                // unary expression
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpUnaryMinus, pos, ln, col);
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::PunctParenL, pos, ln, col);
                                prev = next;
                            }
                            _ => {
                                // check if "-" followed by a digit or decimal
                                if !ASCII_DIGIT.contains(&next) && next != '.' {
                                    self.stack.push(next);
                                    self.push_token(TokenType::OpSub, pos, ln, col);
                                } else {
                                    pos += 1;
                                    col += 1;
                                    buf.push(this);
                                    buf.push(next);
                                    prev = next;
                                }
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpSub, pos, ln, col);
                    }
                }
                '/' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match: / /= // //=
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "/=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpAssignDiv, pos, ln, col);
                                prev = next;
                            }
                            "//" => {
                                // //=
                                if self.stack.last().map_or(false, |next| next.eq(&'=')) {
                                    if let Some(next) = self.stack.pop() {
                                        pos += 1;
                                        col += 1;
                                        self.push_token(TokenType::OpAssignFloorDiv, pos, ln, col);
                                        prev = next;
                                    }
                                } else {
                                    self.push_token(TokenType::OpFloorDiv, pos, ln, col);
                                    prev = next;
                                }
                            }
                            _ => {
                                self.stack.push(next);
                                self.push_token(TokenType::OpDiv, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpDiv, pos, ln, col);
                    }
                }
                '*' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match: * *= ** **=
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "*=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpAssignMul, pos, ln, col);
                                prev = next;
                            }
                            "**" => {
                                pos += 1;
                                col += 1;

                                // **=
                                if self.stack.last().map_or(false, |next| next.eq(&'=')) {
                                    if let Some(next) = self.stack.pop() {
                                        pos += 1;
                                        col += 1;
                                        self.push_token(TokenType::OpAssignPow, pos, ln, col);
                                        prev = next;
                                    }
                                } else {
                                    self.push_token(TokenType::OpPow, pos, ln, col);
                                    prev = next;
                                }
                            }

                            _ => {
                                self.stack.push(next);
                                self.push_token(TokenType::OpMul, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpMul, pos, ln, col);
                    }
                }
                '%' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match: % %=
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "%=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpAssignMod, pos, ln, col);
                                prev = next;
                            }
                            _ => {
                                self.stack.push(next);
                                self.push_token(TokenType::OpMod, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpMod, pos, ln, col);
                    }
                }
                '&' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match: & && &=
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "&&" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpLogicalAnd, pos, ln, col);
                                prev = next;
                            }
                            "&=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpAssignBitwiseAnd, pos, ln, col);
                                prev = next;
                            }
                            _ => {
                                self.stack.push(next);
                                self.push_token(TokenType::OpBitwiseAnd, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpBitwiseAnd, pos, ln, col);
                    }
                }
                '|' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // match: flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // | ||
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            "||" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpLogicalOr, pos, ln, col);
                                prev = next;
                            }
                            "|=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpAssignBitwiseOr, pos, ln, col);
                                prev = next;
                            }
                            _ => {
                                self.stack.push(next);
                                self.push_token(TokenType::OpBitwiseOr, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpBitwiseOr, pos, ln, col);
                    }
                }
                ':' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }

                    // This will flush the package if it exists as a prefix
                    // and then the IdFunc is second token
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // match: : :: field: :func
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            // match enum variant
                            "::" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::PunctDoubleColon, pos, ln, col);
                                prev = next;
                            }
                            _ => {
                                if !ASCII_LOWER.contains(&next) {
                                    // :
                                    self.stack.push(next);
                                    self.push_token(TokenType::PunctColon, pos, ln, col);
                                } else {
                                    // sniff :func here
                                    // if the next char is a-z
                                    // take until space or newline
                                    pos += 1;
                                    col += 1;
                                    buf.push(this);
                                    buf.push(next);
                                    prev = next;
                                    while let Some(next) = self.stack.pop() {
                                        if next == '\n' || next == ' ' {
                                            self.stack.push(next);
                                            break;
                                        }
                                        pos += 1;
                                        col += 1;
                                        buf.push(next);
                                        prev = next;
                                    }
                                    self.buf_to_token(&mut buf, true, pos, ln, col);
                                    continue;
                                }
                            }
                        }
                    } else {
                        self.push_token(TokenType::PunctColon, pos, ln, col);
                    }
                }
                '^' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // match: flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);

                    // ^ ^= ^Trait
                    if let Some(next) = self.stack.pop() {
                        let pair = [this, next];
                        let sym = &String::from_iter(pair)[..];
                        match sym {
                            // Assign
                            "^=" => {
                                pos += 1;
                                col += 1;
                                self.push_token(TokenType::OpAssignBitwiseXOr, pos, ln, col);
                                prev = next;
                            }
                            // ^Trait
                            "^A" | "^B" | "^C" | "^D" | "^E" | "^F" | "^G" | "^H" | "^I" | "^J"
                            | "^K" | "^L" | "^M" | "^N" | "^O" | "^P" | "^Q" | "^R" | "^S"
                            | "^T" | "^U" | "^V" | "^W" | "^X" | "^Y" | "^Z" => {
                                buf.push(this);
                                buf.push(next);
                                prev = next;
                                while let Some(next) = self.stack.pop() {
                                    if next == '\n' || next == ' ' {
                                        self.stack.push(next);
                                        break;
                                    }
                                    pos += 1;
                                    col += 1;
                                    buf.push(next);
                                    prev = next;
                                }
                                self.buf_to_token(&mut buf, true, pos, ln, col);
                            }
                            // ^
                            _ => {
                                self.stack.push(next);
                                self.push_token(TokenType::OpBitwiseXOr, pos, ln, col);
                            }
                        }
                    } else {
                        self.push_token(TokenType::OpBitwiseXOr, pos, ln, col);
                    }
                }
                '{' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);
                    self.push_token(TokenType::PunctBraceL, pos, ln, col);
                }
                '}' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);
                    self.push_token(TokenType::PunctBraceR, pos, ln, col);
                }
                '[' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);
                    self.push_token(TokenType::PunctBracketL, pos, ln, col);
                }
                ']' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);
                    self.push_token(TokenType::PunctBracketR, pos, ln, col);
                }
                '(' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);
                    self.push_token(TokenType::PunctParenL, pos, ln, col);
                }
                ')' => {
                    // inside quote
                    if quote_on {
                        buf.push(this);
                        prev = this;
                        continue;
                    }
                    // clear indent
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    // flush buffer
                    self.buf_to_token(&mut buf, true, pos, ln, col);
                    self.push_token(TokenType::PunctParenR, pos, ln, col);
                }
                _ => {
                    if indent_on {
                        self.push_token(TokenType::Indent(indent), pos, ln, col);
                        indent_on = false;
                    }
                    buf.push(this);
                    // println!("no match: {}", this);
                }
            }
            prev = this;
        }

        if buf.len() > 0 {
            self.buf_to_token(&mut buf, true, pos, ln, col);
            // println!("buf: {:?}", &buf);
        }
        info!("======TOK======");
        info!("{:#?}", self.tokens);
        info!("===============");
    }

    #[allow(unused_assignments)]
    fn push_token(&mut self, token_type: TokenType, _pos: usize, _line: usize, col: usize) {
        let mut _start = 0usize;
        match &token_type {
            // len=1
            TokenType::Sp
            | TokenType::NL
            | TokenType::PunctBraceL
            | TokenType::PunctBraceR
            | TokenType::PunctBracketL
            | TokenType::PunctBracketR
            | TokenType::PunctParenL
            | TokenType::PunctParenR
            | TokenType::PunctColon
            | TokenType::PunctExclamation
            | TokenType::OpAssignEq
            | TokenType::OpGt
            | TokenType::OpLt
            | TokenType::OpAdd
            | TokenType::OpSub
            | TokenType::OpMul
            | TokenType::OpDiv
            | TokenType::OpMod
            | TokenType::OpUnaryMinus
            | TokenType::OpUnaryPlus
            | TokenType::OpUnaryNot
            | TokenType::OpBitwiseXOr
            | TokenType::OpLogicalNot => _start = col - 1,

            // len=2
            TokenType::PunctDoubleColon
            | TokenType::PunctFatArrow
            | TokenType::OpEqEq
            | TokenType::OpNotEq
            | TokenType::OpGte
            | TokenType::OpLte
            | TokenType::OpFloorDiv
            | TokenType::OpPow
            | TokenType::OpAssignAdd
            | TokenType::OpAssignSub
            | TokenType::OpAssignDiv
            | TokenType::OpAssignMul
            | TokenType::OpAssignMod
            | TokenType::OpAssignBitwiseAnd
            | TokenType::OpAssignBitwiseOr
            | TokenType::OpAssignBitwiseXOr
            | TokenType::OpLogicalAnd
            | TokenType::OpLogicalOr
            | TokenType::OpBitwiseAnd
            | TokenType::OpBitwiseOr
            | TokenType::OpBitwiseShiftL
            | TokenType::OpBitwiseShiftR => _start = col - 2,

            // len=3
            TokenType::OpAssignPow
            | TokenType::OpAssignFloorDiv
            | TokenType::OpAssignBitwiseShiftL
            | TokenType::OpAssignBitwiseShiftR => _start = col - 3,

            // len=fixed
            TokenType::PrBool => _start = col - 4,
            TokenType::PrByte => _start = col - 4,
            TokenType::PrChar => _start = col - 4,
            TokenType::PrFloat => _start = col - 5,
            TokenType::PrFloat32 => _start = col - 7,
            TokenType::PrFloat64 => _start = col - 7,
            TokenType::PrInt => _start = col - 3,
            TokenType::PrInt8 => _start = col - 4,
            TokenType::PrInt16 => _start = col - 5,
            TokenType::PrInt32 => _start = col - 5,
            TokenType::PrInt64 => _start = col - 5,
            TokenType::PrStr => _start = col - 3,
            TokenType::PrUInt => _start = col - 4,
            TokenType::PrUInt8 => _start = col - 5,
            TokenType::PrUInt16 => _start = col - 6,
            TokenType::PrUInt32 => _start = col - 6,
            TokenType::PrUInt64 => _start = col - 6,

            TokenType::KwAs => _start = col - 2,
            TokenType::KwBreak => _start = col - 5,
            TokenType::KwCase => _start = col - 4,
            TokenType::KwConst => _start = col - 5,
            TokenType::KwContinue => _start = col - 8,
            TokenType::KwDependencies => _start = col - 12,
            TokenType::KwDescription => _start = col - 11,
            TokenType::KwElse => _start = col - 4,
            TokenType::KwElseIf => _start = col - 4, // elif
            TokenType::KwEnum => _start = col - 4,
            TokenType::KwExport => _start = col - 6,
            TokenType::KwFiles => _start = col - 5,
            TokenType::KwFn => _start = col - 2,
            TokenType::KwFor => _start = col - 3,
            TokenType::KwIf => _start = col - 2,
            TokenType::KwImpl => _start = col - 4,
            TokenType::KwImport => _start = col - 6,
            TokenType::KwIn => _start = col - 2,
            TokenType::KwLazy => _start = col - 4,
            TokenType::KwLet => _start = col - 3,
            TokenType::KwMatch => _start = col - 5,
            TokenType::KwPackage => _start = col - 7,
            TokenType::KwPrimitive => _start = col - 8,
            TokenType::KwReturn => _start = col - 6,
            TokenType::KwSelf => _start = col - 4,
            TokenType::KwStruct => _start = col - 6,
            TokenType::KwTest => _start = col - 4,
            TokenType::KwTestCase => _start = col - 8,
            TokenType::KwThen => _start = col - 4,
            TokenType::KwTrait => _start = col - 5,
            TokenType::KwType => _start = col - 4,
            TokenType::KwVersion => _start = col - 7,
            TokenType::KwWhile => _start = col - 5,

            // Builtin Types, Fn, const, etc
            TokenType::BuiltinTypeList => _start = col - 4,
            TokenType::BuiltinTypeMap => _start = col - 3,
            TokenType::BuiltinTypeMaybe => _start = col - 5,
            TokenType::BuiltinTypeNone => _start = col - 4,
            TokenType::BuiltinTypeOption => _start = col - 6,
            TokenType::BuiltinTypeSet => _start = col - 3,
            TokenType::BuiltinTypeString => _start = col - 6,

            // Special
            TokenType::SpecialTypeSelf => _start = col - 4,

            // len=unknown
            TokenType::Comment(text)
            | TokenType::LitString(text)
            | TokenType::LitBoolean(text)
            | TokenType::LitNumber(text)
            | TokenType::IdPackage(text)
            | TokenType::IdType(text)
            | TokenType::IdVar(text)
            | TokenType::IdFunc(text)
            | TokenType::IdTrait(text) => _start = col - text.len(),

            // indent
            TokenType::Indent(size) => _start = col - size,
        }
        // push token
        let token = Token {
            // span: Span {
            //     ln: line,
            //     start: start,
            //     end: col,
            //     pos: pos,
            // },
            ty: token_type,
        };
        self.tokens.push(token);
    }

    fn buf_to_token(
        &mut self,
        buf: &mut Vec<char>,
        flush: bool,
        pos: usize,
        line: usize,
        col: usize,
    ) {
        if buf.len() == 0 {
            return;
        }
        let this = &String::from_iter(buf.clone())[..];
        match this {
            // Keywords
            "as" => self.push_token(TokenType::KwAs, pos, line, col),
            "break" => self.push_token(TokenType::KwBreak, pos, line, col),
            "case" => self.push_token(TokenType::KwCase, pos, line, col),
            "const" => self.push_token(TokenType::KwConst, pos, line, col),
            "continue" => self.push_token(TokenType::KwContinue, pos, line, col),
            "elif" => self.push_token(TokenType::KwElseIf, pos, line, col),
            "else" => self.push_token(TokenType::KwElse, pos, line, col),
            "enum" => self.push_token(TokenType::KwEnum, pos, line, col),
            "fn" => self.push_token(TokenType::KwFn, pos, line, col),
            "for" => self.push_token(TokenType::KwFor, pos, line, col),
            "if" => self.push_token(TokenType::KwIf, pos, line, col),
            "impl" => self.push_token(TokenType::KwImpl, pos, line, col),
            "in" => self.push_token(TokenType::KwIn, pos, line, col),
            "lazy" => self.push_token(TokenType::KwLazy, pos, line, col),
            "let" => self.push_token(TokenType::KwLet, pos, line, col),
            "match" => self.push_token(TokenType::KwMatch, pos, line, col),
            "primitive" => self.push_token(TokenType::KwPrimitive, pos, line, col),
            "return" => self.push_token(TokenType::KwReturn, pos, line, col),
            "self" => self.push_token(TokenType::KwSelf, pos, line, col),
            "struct" => self.push_token(TokenType::KwStruct, pos, line, col),
            "test" => self.push_token(TokenType::KwTest, pos, line, col),
            "testcase" => self.push_token(TokenType::KwTestCase, pos, line, col),
            "then" => self.push_token(TokenType::KwThen, pos, line, col),
            "trait" => self.push_token(TokenType::KwTrait, pos, line, col),
            "type" => self.push_token(TokenType::KwType, pos, line, col),
            "while" => self.push_token(TokenType::KwWhile, pos, line, col),

            // package keywords
            "package" => self.push_token(TokenType::KwPackage, pos, line, col),
            "version" => self.push_token(TokenType::KwVersion, pos, line, col),
            "description" => self.push_token(TokenType::KwDescription, pos, line, col),
            "dependencies" => self.push_token(TokenType::KwDependencies, pos, line, col),
            "export" => self.push_token(TokenType::KwExport, pos, line, col),
            "import" => self.push_token(TokenType::KwImport, pos, line, col),
            "files" => self.push_token(TokenType::KwFiles, pos, line, col),

            // Literals
            "true" | "false" => {
                self.push_token(TokenType::LitBoolean(this.to_owned()), pos, line, col)
            }

            // Primitives
            "bool" => self.push_token(TokenType::PrBool, pos, line, col),
            "byte" => self.push_token(TokenType::PrByte, pos, line, col),
            "char" => self.push_token(TokenType::PrChar, pos, line, col),
            "float" => self.push_token(TokenType::PrFloat32, pos, line, col),
            "float32" => self.push_token(TokenType::PrFloat32, pos, line, col),
            "float64" => self.push_token(TokenType::PrFloat64, pos, line, col),
            "int" => self.push_token(TokenType::PrInt32, pos, line, col),
            "int16" => self.push_token(TokenType::PrInt16, pos, line, col),
            "int32" => self.push_token(TokenType::PrInt32, pos, line, col),
            "int64" => self.push_token(TokenType::PrInt64, pos, line, col),
            "int8" => self.push_token(TokenType::PrInt8, pos, line, col),
            "str" => self.push_token(TokenType::PrStr, pos, line, col),
            "uint" => self.push_token(TokenType::PrUInt32, pos, line, col),
            "uint16" => self.push_token(TokenType::PrUInt16, pos, line, col),
            "uint32" => self.push_token(TokenType::PrUInt32, pos, line, col),
            "uint64" => self.push_token(TokenType::PrUInt64, pos, line, col),
            "uint8" => self.push_token(TokenType::PrUInt8, pos, line, col),

            // BuiltIn types
            "List" => self.push_token(TokenType::BuiltinTypeList, pos, line, col),
            "Map" => self.push_token(TokenType::BuiltinTypeMap, pos, line, col),
            "Maybe" => self.push_token(TokenType::BuiltinTypeMaybe, pos, line, col),
            "None" => self.push_token(TokenType::BuiltinTypeNone, pos, line, col),
            "Option" => self.push_token(TokenType::BuiltinTypeOption, pos, line, col),
            "Set" => self.push_token(TokenType::BuiltinTypeSet, pos, line, col),

            // SpecialType
            "Self" => self.push_token(TokenType::SpecialTypeSelf, pos, line, col),

            // Pattern
            // "" => self.push_token(Token::..., pos, line, col),
            _ => {
                if !flush {
                    return;
                }
                // debug!("buf_to_token this: {}", &this);
                if let Some(token_type) = self.matcher.resolve(this) {
                    self.push_token(token_type, pos, line, col)
                } else {
                    panic!(
                        "Expected literal or identity token. Unknown token: {:?}",
                        this
                    );
                }
            }
        }
        buf.clear();
    }
}
