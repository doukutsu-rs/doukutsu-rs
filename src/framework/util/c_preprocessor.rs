//! Generic C-style preprocessor for conditional compilation.
//!
//! # Supported
//!
//! ## Directives
//!
//! | Directive               | Behavior                                                      |
//! |-------------------------|---------------------------------------------------------------|
//! | `#if EXPR`              | Evaluates expression (see grammar below). Pushes a frame.     |
//! | `#ifdef NAME`           | Equivalent to `#if defined(NAME)`.                            |
//! | `#ifndef NAME`          | Equivalent to `#if !defined(NAME)`.                           |
//! | `#elif EXPR`            | Like `#if`, in an `#if` chain. Illegal after `#else`.         |
//! | `#else`                 | Inverts the chain. Only one per chain.                        |
//! | `#endif`                | Pops the current frame.                                       |
//! | `#define NAME`          | Presence-only define. Tracked **and** emitted verbatim.       |
//! | `#define NAME VALUE`    | Value-bearing define. Tracked **and** emitted verbatim.       |
//! | `#undef NAME`           | Removes the define. Emitted verbatim.                         |
//! | `#error MSG`            | Returns `Err` if reached in an active branch.                 |
//! | `#warning MSG`          | Logs via `log::warn!` if reached in an active branch.         |
//!
//! Source-level `#define` / `#undef` are tracked internally **and**
//! emitted verbatim, so any downstream consumer (a GLSL driver's own
//! preprocessor, a second pass, etc.) can act on them too. Externally
//! supplied defines (via [`Preprocessor::define`] or the [`preprocess`]
//! helper's slice argument) are tracked but **not** emitted — they're
//! pre-conditions, not source-level directives.
//!
//! ## `#if` / `#elif` expression grammar
//!
//! - `defined(NAME)` and `defined NAME` → `1` if defined, else `0`.
//! - Decimal integer literals.
//! - Identifiers — resolved against the define table:
//!   - value-bearing: parse value as `i64`, fall back to `0` on parse error;
//!   - presence-only: `1` (truthy — extension, C requires a value);
//!   - undefined: `0`.
//! - Unary `!`, parens, binary `&&`, `||`, `==`, `!=`.
//!
//! # Unsupported
//!
//! - `#include`, `#line`, `#pragma` — passed through verbatim if a `#`
//!   directive name we don't own appears (e.g. `#version`, `#extension`).
//! - Function-like macros (`#define X(a,b) ...`).
//! - Macro expansion in source bodies — identifiers in regular lines are
//!   never substituted. Use a downstream preprocessor pass for that.
//! - Token pasting (`##`) and stringification (`#x`).
//! - Line continuation (`\<newline>`).
//! - Comments inside directive lines (`#if /* foo */ X` is not parsed).
//! - Arithmetic and bitwise operators (`+`, `-`, `*`, `<`, `&`, `<<`, …)
//!   in `#if` expressions.
//! - `#elif` after `#else` (rejected, matching C).
//!
//! # Limits
//!
//! - Conditional nesting capped at [`MAX_DEPTH`] (64).
//! - Output size is bounded by input size by construction (no expansion,
//!   no recursion), so no DoS-via-expansion is possible.

use std::collections::HashMap;

use crate::framework::error::{GameError, GameResult};

pub const MAX_DEPTH: usize = 64;

pub struct Preprocessor {
    /// `None` value = presence-only define (`#define X`). `Some(s)` = `#define X s`.
    defines: HashMap<String, Option<String>>,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self { defines: HashMap::new() }
    }

    pub fn define(&mut self, name: &str, value: Option<&str>) {
        self.defines.insert(name.to_owned(), value.map(str::to_owned));
    }

    pub fn undefine(&mut self, name: &str) {
        self.defines.remove(name);
    }

    pub fn process(&mut self, source: &str) -> GameResult<String> {
        let mut stack: Vec<Frame> = Vec::new();
        let mut out = String::with_capacity(source.len());

        for (lineno, raw) in source.lines().enumerate() {
            let active = stack.last().map_or(true, |f| f.active);
            let trimmed = raw.trim_start();
            let directive = parse_directive_kind(trimmed);

            match directive {
                Some(DirKind::If) => {
                    push_frame(&mut stack, lineno, |parent| {
                        let expr = strip_directive(trimmed, "if");
                        Ok(parent && eval_expr(expr, &self.defines, lineno)? != 0)
                    })?;
                }
                Some(DirKind::Ifdef) => {
                    push_frame(&mut stack, lineno, |parent| {
                        let name = parse_ident_arg(trimmed, "ifdef", lineno)?;
                        Ok(parent && self.defines.contains_key(name))
                    })?;
                }
                Some(DirKind::Ifndef) => {
                    push_frame(&mut stack, lineno, |parent| {
                        let name = parse_ident_arg(trimmed, "ifndef", lineno)?;
                        Ok(parent && !self.defines.contains_key(name))
                    })?;
                }
                Some(DirKind::Elif) => {
                    let expr = strip_directive(trimmed, "elif");
                    let value = eval_expr(expr, &self.defines, lineno)? != 0;
                    flip_branch(&mut stack, lineno, value, false)?;
                }
                Some(DirKind::Else) => {
                    flip_branch(&mut stack, lineno, true, true)?;
                }
                Some(DirKind::Endif) => {
                    stack.pop().ok_or_else(|| err(lineno, "stray #endif with no matching #if/#ifdef/#ifndef"))?;
                }
                Some(DirKind::Define) if active => {
                    let (name, value) = parse_define(trimmed, lineno)?;
                    self.defines.insert(name.to_owned(), value.map(str::to_owned));
                    out.push_str(raw);
                    out.push('\n');
                }
                Some(DirKind::Undef) if active => {
                    let name = parse_ident_arg(trimmed, "undef", lineno)?;
                    self.defines.remove(name);
                    out.push_str(raw);
                    out.push('\n');
                }
                Some(DirKind::Error) if active => {
                    let msg = strip_directive(trimmed, "error").trim();
                    return Err(err(lineno, &format!("#error {}", msg)));
                }
                Some(DirKind::Warning) if active => {
                    let msg = strip_directive(trimmed, "warning").trim();
                    log::warn!("preprocessor warning (line {}): {}", lineno + 1, msg);
                }
                Some(_) => {
                    // Directive in inactive branch — drop.
                }
                None => {
                    if active {
                        out.push_str(raw);
                        out.push('\n');
                    }
                }
            }
        }

        if !stack.is_empty() {
            return Err(GameError::RenderError(format!(
                "preprocessor: {} unterminated conditional block(s) at end of source",
                stack.len()
            )));
        }

        Ok(out)
    }
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience: process `source` with `defines` as presence-only initial defines.
pub fn preprocess(source: &str, defines: &[&str]) -> GameResult<String> {
    let mut p = Preprocessor::new();
    for d in defines {
        p.define(d, None);
    }
    p.process(source)
}

struct Frame {
    active: bool,
    /// Has any branch in this if-chain matched? Drives `#elif` / `#else`.
    any_matched: bool,
    /// Has `#else` already been seen in this chain? `#elif` after `#else` is illegal.
    saw_else: bool,
}

fn push_frame<F>(stack: &mut Vec<Frame>, lineno: usize, eval: F) -> GameResult<()>
where
    F: FnOnce(bool) -> GameResult<bool>,
{
    if stack.len() >= MAX_DEPTH {
        return Err(err(lineno, "preprocessor stack too deep"));
    }
    let parent_active = stack.last().map_or(true, |f| f.active);
    let matched = eval(parent_active)?;
    stack.push(Frame { active: matched, any_matched: matched, saw_else: false });
    Ok(())
}

/// Apply `#elif EXPR` (cond=its value, is_else=false) or `#else` (cond=true, is_else=true).
fn flip_branch(stack: &mut Vec<Frame>, lineno: usize, cond: bool, is_else: bool) -> GameResult<()> {
    let parent_active = if stack.len() < 2 { true } else { stack[stack.len() - 2].active };
    let frame = stack.last_mut().ok_or_else(|| err(lineno, "stray #elif/#else with no matching #if"))?;
    if frame.saw_else {
        return Err(err(lineno, "#elif/#else after #else"));
    }
    let take = parent_active && !frame.any_matched && cond;
    frame.active = take;
    frame.any_matched = frame.any_matched || take;
    if is_else {
        frame.saw_else = true;
    }
    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum DirKind {
    If,
    Ifdef,
    Ifndef,
    Elif,
    Else,
    Endif,
    Define,
    Undef,
    Error,
    Warning,
}

fn parse_directive_kind(line: &str) -> Option<DirKind> {
    let rest = line.strip_prefix('#')?.trim_start();
    let (kw, _args) = split_token(rest);
    Some(match kw {
        "if" => DirKind::If,
        "ifdef" => DirKind::Ifdef,
        "ifndef" => DirKind::Ifndef,
        "elif" => DirKind::Elif,
        "else" => DirKind::Else,
        "endif" => DirKind::Endif,
        "define" => DirKind::Define,
        "undef" => DirKind::Undef,
        "error" => DirKind::Error,
        "warning" => DirKind::Warning,
        _ => return None,
    })
}

fn strip_directive<'a>(line: &'a str, kw: &str) -> &'a str {
    let line = line.trim_start().strip_prefix('#').unwrap_or(line).trim_start();
    line.strip_prefix(kw).unwrap_or("").trim_start()
}

fn parse_ident_arg<'a>(line: &'a str, kw: &str, lineno: usize) -> GameResult<&'a str> {
    let arg = strip_directive(line, kw).trim();
    parse_ident(arg).ok_or_else(|| err(lineno, &format!("#{} requires an identifier argument", kw)))
}

fn parse_define(line: &str, lineno: usize) -> GameResult<(&str, Option<&str>)> {
    let rest = strip_directive(line, "define");
    let (name, value) = split_token(rest);
    if !is_ident(name) {
        return Err(err(lineno, "#define requires an identifier argument"));
    }
    let value = value.trim();
    Ok((name, if value.is_empty() { None } else { Some(value) }))
}

fn split_token(s: &str) -> (&str, &str) {
    let end = s.find(|c: char| c.is_ascii_whitespace()).unwrap_or(s.len());
    (&s[..end], s[end..].trim_start())
}

fn parse_ident(s: &str) -> Option<&str> {
    if is_ident(s) {
        Some(s)
    } else {
        None
    }
}

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn err(lineno: usize, msg: &str) -> GameError {
    GameError::RenderError(format!("preprocessor (line {}): {}", lineno + 1, msg))
}

// ---- Expression evaluator for #if / #elif --------------------------------
//
// Streaming lexer + recursive-descent parser. No `Vec<Tok>` is built —
// the parser pulls tokens via `peek`/`advance` directly off the input
// `&str`, so each `#if` / `#elif` evaluation allocates nothing on the
// happy path (errors still allocate a `String` message, which is fine).

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tok<'a> {
    Ident(&'a str),
    Int(i64),
    LParen,
    RParen,
    Not,
    And,
    Or,
    Eq,
    Ne,
}

struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    /// One-token lookahead. `None` slot means "not filled"; an `Option<Tok>`
    /// inside means "filled, possibly with EOF".
    peeked: Option<Option<Tok<'a>>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0, peeked: None }
    }

    fn peek(&mut self) -> Result<Option<Tok<'a>>, String> {
        if self.peeked.is_none() {
            self.peeked = Some(self.scan()?);
        }
        Ok(self.peeked.unwrap())
    }

    fn advance(&mut self) -> Result<Option<Tok<'a>>, String> {
        if let Some(t) = self.peeked.take() {
            Ok(t)
        } else {
            self.scan()
        }
    }

    fn scan(&mut self) -> Result<Option<Tok<'a>>, String> {
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() && matches!(bytes[self.pos], b' ' | b'\t') {
            self.pos += 1;
        }
        if self.pos >= bytes.len() {
            return Ok(None);
        }
        let i = self.pos;
        let c = bytes[i];
        let two = |a: u8, b: u8| i + 1 < bytes.len() && bytes[i] == a && bytes[i + 1] == b;
        let tok = match c {
            b'(' => { self.pos += 1; Tok::LParen }
            b')' => { self.pos += 1; Tok::RParen }
            b'!' if two(b'!', b'=') => { self.pos += 2; Tok::Ne }
            b'!' => { self.pos += 1; Tok::Not }
            b'=' if two(b'=', b'=') => { self.pos += 2; Tok::Eq }
            b'&' if two(b'&', b'&') => { self.pos += 2; Tok::And }
            b'|' if two(b'|', b'|') => { self.pos += 2; Tok::Or }
            b'0'..=b'9' => {
                let start = i;
                while self.pos < bytes.len() && bytes[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
                let s = &self.input[start..self.pos];
                let n: i64 = s.parse().map_err(|_| format!("invalid integer literal '{}'", s))?;
                Tok::Int(n)
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                let start = i;
                while self.pos < bytes.len() && (bytes[self.pos].is_ascii_alphanumeric() || bytes[self.pos] == b'_') {
                    self.pos += 1;
                }
                Tok::Ident(&self.input[start..self.pos])
            }
            _ => return Err(format!("unexpected character '{}'", c as char)),
        };
        Ok(Some(tok))
    }
}

struct Parser<'a, 'b> {
    lex: Lexer<'a>,
    defines: &'b HashMap<String, Option<String>>,
}

impl<'a, 'b> Parser<'a, 'b> {
    fn parse_or(&mut self) -> Result<i64, String> {
        let mut left = self.parse_and()?;
        while matches!(self.lex.peek()?, Some(Tok::Or)) {
            self.lex.advance()?;
            let right = self.parse_and()?;
            left = (left != 0 || right != 0) as i64;
        }
        Ok(left)
    }
    fn parse_and(&mut self) -> Result<i64, String> {
        let mut left = self.parse_eq()?;
        while matches!(self.lex.peek()?, Some(Tok::And)) {
            self.lex.advance()?;
            let right = self.parse_eq()?;
            left = (left != 0 && right != 0) as i64;
        }
        Ok(left)
    }
    fn parse_eq(&mut self) -> Result<i64, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.lex.peek()? {
                Some(Tok::Eq) => true,
                Some(Tok::Ne) => false,
                _ => return Ok(left),
            };
            self.lex.advance()?;
            let right = self.parse_unary()?;
            left = if op { (left == right) as i64 } else { (left != right) as i64 };
        }
    }
    fn parse_unary(&mut self) -> Result<i64, String> {
        if matches!(self.lex.peek()?, Some(Tok::Not)) {
            self.lex.advance()?;
            let v = self.parse_unary()?;
            Ok((v == 0) as i64)
        } else {
            self.parse_atom()
        }
    }
    fn parse_atom(&mut self) -> Result<i64, String> {
        match self.lex.advance()? {
            Some(Tok::Int(n)) => Ok(n),
            Some(Tok::LParen) => {
                let v = self.parse_or()?;
                match self.lex.advance()? {
                    Some(Tok::RParen) => Ok(v),
                    _ => Err("expected ')'".into()),
                }
            }
            Some(Tok::Ident("defined")) => {
                let paren = matches!(self.lex.peek()?, Some(Tok::LParen));
                if paren {
                    self.lex.advance()?;
                }
                let name = match self.lex.advance()? {
                    Some(Tok::Ident(n)) => n,
                    _ => return Err("expected identifier after 'defined'".into()),
                };
                if paren {
                    match self.lex.advance()? {
                        Some(Tok::RParen) => {}
                        _ => return Err("expected ')' after 'defined' argument".into()),
                    }
                }
                Ok(self.defines.contains_key(name) as i64)
            }
            Some(Tok::Ident(name)) => match self.defines.get(name) {
                Some(Some(value)) => Ok(value.trim().parse::<i64>().unwrap_or(0)),
                Some(None) => Ok(1),
                None => Ok(0),
            },
            None => Err("unexpected end of expression".into()),
            other => Err(format!("unexpected token: {:?}", other)),
        }
    }
}

fn eval_expr(expr: &str, defines: &HashMap<String, Option<String>>, lineno: usize) -> GameResult<i64> {
    let mut p = Parser { lex: Lexer::new(expr), defines };
    let v = p.parse_or().map_err(|e| err(lineno, &format!("expression parse: {}", e)))?;
    if p.lex.advance().map_err(|e| err(lineno, &e))?.is_some() {
        return Err(err(lineno, "trailing tokens after expression"));
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str, defs: &[&str]) -> String {
        preprocess(src, defs).unwrap()
    }

    #[test]
    fn ifdef_active_branch_emitted() {
        let src = "#ifdef A\nyes\n#endif\n";
        assert_eq!(run(src, &["A"]), "yes\n");
        assert_eq!(run(src, &[]), "");
    }

    #[test]
    fn ifndef_inverts() {
        let src = "#ifndef A\nyes\n#endif\n";
        assert_eq!(run(src, &[]), "yes\n");
        assert_eq!(run(src, &["A"]), "");
    }

    #[test]
    fn else_takes_when_if_does_not() {
        let src = "#ifdef A\nA\n#else\nB\n#endif\n";
        assert_eq!(run(src, &["A"]), "A\n");
        assert_eq!(run(src, &[]), "B\n");
    }

    #[test]
    fn elif_chain() {
        let src = "#ifdef A\nA\n#elif defined(B)\nB\n#elif defined(C)\nC\n#else\nfallback\n#endif\n";
        assert_eq!(run(src, &["A"]), "A\n");
        assert_eq!(run(src, &["B"]), "B\n");
        assert_eq!(run(src, &["C"]), "C\n");
        assert_eq!(run(src, &[]), "fallback\n");
        // Earlier match wins:
        assert_eq!(run(src, &["A", "B"]), "A\n");
    }

    #[test]
    fn nested_ifdef() {
        let src = "#ifdef A\n#ifdef B\nAB\n#else\nA_only\n#endif\n#endif\n";
        assert_eq!(run(src, &["A", "B"]), "AB\n");
        assert_eq!(run(src, &["A"]), "A_only\n");
        assert_eq!(run(src, &["B"]), "");
    }

    #[test]
    fn else_does_not_activate_when_parent_inactive() {
        let src = "#ifdef A\n#ifdef B\ntop\n#else\nbottom\n#endif\n#endif\n";
        assert_eq!(run(src, &[]), "");
    }

    #[test]
    fn if_defined_expr() {
        let src = "#if defined(A) && !defined(B)\nyes\n#endif\n";
        assert_eq!(run(src, &["A"]), "yes\n");
        assert_eq!(run(src, &["A", "B"]), "");
        assert_eq!(run(src, &[]), "");
    }

    #[test]
    fn if_zero_strips_block() {
        // Classic "comment out a block" idiom.
        let src = "kept\n#if 0\nstripped\n#endif\nalso kept\n";
        assert_eq!(run(src, &[]), "kept\nalso kept\n");
    }

    #[test]
    fn if_one_keeps_block() {
        let src = "#if 1\nkept\n#endif\n";
        assert_eq!(run(src, &[]), "kept\n");
    }

    #[test]
    fn if_value_lookup() {
        // External (caller-supplied) defines are tracked but not emitted —
        // they're pre-conditions, not source-level directives.
        let mut p = Preprocessor::new();
        p.define("VERSION", Some("3"));
        let out = p.process("#if VERSION == 3\nv3\n#elif VERSION == 4\nv4\n#endif\n").unwrap();
        assert_eq!(out, "v3\n");
    }

    #[test]
    fn presence_only_define_is_truthy_in_if() {
        let src = "#define A\n#if A\nyes\n#endif\n";
        assert_eq!(run(src, &[]), "#define A\nyes\n");
    }

    #[test]
    fn source_define_is_emitted_and_tracked() {
        let src = "#define A 1\nbody\n#ifdef A\nseen\n#endif\n";
        assert_eq!(run(src, &[]), "#define A 1\nbody\nseen\n");
    }

    #[test]
    fn undef_in_active_branch_removes() {
        let src = "#define A\n#undef A\n#ifdef A\nno\n#endif\n";
        assert_eq!(run(src, &[]), "#define A\n#undef A\n");
    }

    #[test]
    fn version_passes_through() {
        let src = "#version 300 es\nbody\n";
        assert_eq!(run(src, &[]), "#version 300 es\nbody\n");
    }

    #[test]
    fn unbalanced_ifdef_errors() {
        assert!(preprocess("#ifdef A\nbody\n", &["A"]).is_err());
    }

    #[test]
    fn stray_endif_errors() {
        assert!(preprocess("body\n#endif\n", &[]).is_err());
    }

    #[test]
    fn elif_after_else_errors() {
        assert!(preprocess("#ifdef A\n1\n#else\n2\n#elif defined(B)\n3\n#endif\n", &["A"]).is_err());
    }

    #[test]
    fn error_directive_returns_err() {
        assert!(preprocess("#ifdef A\n#error nope\n#endif\n", &["A"]).is_err());
        // Inactive branch: not triggered.
        assert!(preprocess("#ifdef A\n#error nope\n#endif\n", &[]).is_ok());
    }

    #[test]
    fn deeply_nested_rejected_at_cap() {
        let mut src = String::new();
        for _ in 0..(MAX_DEPTH + 1) {
            src.push_str("#ifdef A\n");
        }
        assert!(preprocess(&src, &["A"]).is_err());
    }

    #[test]
    fn whitespace_before_directive_ok() {
        assert_eq!(run("   #ifdef A\nyes\n#endif\n", &["A"]), "yes\n");
    }
}
