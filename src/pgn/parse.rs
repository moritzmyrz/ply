use std::collections::BTreeMap;
use std::io::{BufRead, Lines};

use super::{read_to_games, PgnError, PgnGame};

pub fn parse_pgn(input: &str) -> Result<Vec<PgnGame>, PgnError> {
    let cursor = std::io::Cursor::new(input.as_bytes());
    read_to_games(cursor)
}

pub fn parse_pgn_reader<R: BufRead>(reader: R) -> Result<Vec<PgnGame>, PgnError> {
    read_to_games(reader)
}

pub struct PgnReader<R: BufRead> {
    lines: Lines<R>,
    buffered_line: Option<String>,
    finished: bool,
}

impl<R: BufRead> PgnReader<R> {
    pub fn new(reader: R) -> Self {
        Self { lines: reader.lines(), buffered_line: None, finished: false }
    }
}

impl<R: BufRead> Iterator for PgnReader<R> {
    type Item = Result<PgnGame, PgnError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let mut current = String::new();
        if let Some(line) = self.buffered_line.take() {
            current.push_str(&line);
            current.push('\n');
        }

        loop {
            match self.lines.next() {
                Some(Ok(line)) => {
                    if line.trim_start().starts_with("[Event ") && !current.trim().is_empty() {
                        self.buffered_line = Some(line);
                        break;
                    }
                    current.push_str(&line);
                    current.push('\n');
                }
                Some(Err(err)) => return Some(Err(PgnError::Parse(format!("failed reading PGN: {err}")))),
                None => {
                    self.finished = true;
                    break;
                }
            }
        }

        if current.trim().is_empty() {
            return None;
        }

        Some(parse_single_game(current.trim()))
    }
}

fn parse_single_game(input: &str) -> Result<PgnGame, PgnError> {
    let mut tags = BTreeMap::new();
    let mut movetext = String::new();
    for line in input.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            let (k, v) = parse_tag_line(t)?;
            tags.insert(k, v);
        } else if !t.is_empty() {
            movetext.push_str(t);
            movetext.push(' ');
        }
    }

    let tokens = tokenize_movetext(&movetext);
    let mut moves = Vec::new();
    let mut result = None;
    for tok in tokens {
        if is_result_token(&tok) {
            result = Some(tok);
            continue;
        }
        if is_move_number_token(&tok) || tok == "*" {
            continue;
        }
        moves.push(tok);
    }

    Ok(PgnGame { tags, moves, result })
}

fn parse_tag_line(line: &str) -> Result<(String, String), PgnError> {
    if !line.starts_with('[') || !line.ends_with(']') {
        return Err(PgnError::Parse(format!("invalid tag line: {line}")));
    }
    let inner = &line[1..line.len() - 1];
    let mut parts = inner.splitn(2, ' ');
    let key = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| PgnError::Parse(format!("missing tag key: {line}")))?;
    let raw_value =
        parts.next().ok_or_else(|| PgnError::Parse(format!("missing tag value: {line}")))?;
    let value = raw_value.trim();
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        return Err(PgnError::Parse(format!("invalid tag value: {line}")));
    }
    Ok((key.to_string(), value[1..value.len() - 1].to_string()))
}

fn tokenize_movetext(input: &str) -> Vec<String> {
    let mut cleaned = String::new();
    let mut in_brace_comment = false;
    let mut paren_depth = 0usize;
    let mut in_line_comment = false;
    for ch in input.chars() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }
        if in_brace_comment {
            if ch == '}' {
                in_brace_comment = false;
            }
            continue;
        }
        if paren_depth > 0 {
            if ch == '(' {
                paren_depth += 1;
            } else if ch == ')' {
                paren_depth -= 1;
            }
            continue;
        }
        match ch {
            '{' => in_brace_comment = true,
            ';' => in_line_comment = true,
            '(' => paren_depth = 1,
            '$' => continue,
            '\n' | '\r' | '\t' => cleaned.push(' '),
            _ => cleaned.push(ch),
        }
    }
    cleaned.split_whitespace().map(|s| s.trim().to_string()).collect()
}

fn is_result_token(tok: &str) -> bool {
    matches!(tok, "1-0" | "0-1" | "1/2-1/2" | "*")
}

fn is_move_number_token(tok: &str) -> bool {
    tok.ends_with('.') || tok.contains("...")
}
