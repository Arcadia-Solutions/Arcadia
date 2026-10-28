use serde_json::{json, Value};

/// Bounds the parser recursion depth, which grows with the amount of tokens.
const MAXIMUM_TOKENS: usize = 100;
/// Bounds the DNF expansion, every AND over ORs multiplies the amount of clauses
/// and the search query evaluates each clause for every title group.
const MAXIMUM_CLAUSES: usize = 64;

fn sanitize_tag_name(name: &str) -> String {
    name.trim().to_lowercase()
}

// AST

#[derive(Debug, Clone)]
enum Expression {
    Tag(String),
    Not(Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
}

// Tokenizer

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Tag(String),
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&character) = chars.peek() {
        match character {
            ' ' | '\t' | '\n' => {
                chars.next();
            }
            '&' => {
                chars.next();
                tokens.push(Token::And);
            }
            '|' => {
                chars.next();
                tokens.push(Token::Or);
            }
            '!' => {
                chars.next();
                tokens.push(Token::Not);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LeftParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RightParen);
            }
            _ => {
                let mut name = String::new();
                while let Some(&next_character) = chars.peek() {
                    if "&|!() \t".contains(next_character) {
                        break;
                    }
                    name.push(next_character);
                    chars.next();
                }
                let sanitized = sanitize_tag_name(&name);
                if sanitized.is_empty() {
                    return Err("empty tag name".to_string());
                }
                tokens.push(Token::Tag(sanitized));
            }
        }
    }

    // Insert implicit OR between adjacent atoms
    // e.g. "science fiction" -> "science | fiction", ")tag" -> ") | tag"
    let mut result = Vec::new();
    for token in tokens {
        if let Some(previous) = result.last() {
            let needs_implicit_or = matches!(previous, Token::Tag(_) | Token::RightParen)
                && matches!(token, Token::Tag(_) | Token::LeftParen | Token::Not);
            if needs_implicit_or {
                result.push(Token::Or);
            }
        }
        result.push(token);
    }

    Ok(result)
}

// Recursive descent parser
// Grammar (precedence: ! > & > | / space):
//   expr     = and_expr ('|' and_expr)*
//   and_expr = unary (unary)*
//   unary    = '!' unary | atom
//   atom     = '(' expr ')' | tag_name

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and_expression()?;

        while self.peek() == Some(&Token::Or) {
            self.advance();
            let right = self.parse_and_expression()?;
            left = Expression::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;

        while self.peek() == Some(&Token::And) {
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.peek() == Some(&Token::Not) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::Not(Box::new(operand)));
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Expression, String> {
        match self.peek().cloned() {
            Some(Token::LeftParen) => {
                self.advance();
                let expression = self.parse_expression()?;
                if self.peek() != Some(&Token::RightParen) {
                    return Err("expected closing parenthesis".to_string());
                }
                self.advance();
                Ok(expression)
            }
            Some(Token::Tag(name)) => {
                self.advance();
                Ok(Expression::Tag(name))
            }
            Some(_) => Err("unexpected token".to_string()),
            None => Err("unexpected end of expression".to_string()),
        }
    }
}

// DNF conversion
// A DNF clause is a set of included and excluded tags.

#[derive(Debug, Clone)]
struct DnfClause {
    include: Vec<String>,
    exclude: Vec<String>,
}

/// Convert an AST to DNF (OR of AND clauses).
/// Each clause contains included tags (positive) and excluded tags (negated).
fn to_dnf(expression: &Expression) -> Result<Vec<DnfClause>, String> {
    match expression {
        Expression::Tag(name) => Ok(vec![DnfClause {
            include: vec![name.clone()],
            exclude: vec![],
        }]),
        Expression::Not(inner) => {
            // Push negation inward using De Morgan's laws
            match inner.as_ref() {
                Expression::Tag(name) => Ok(vec![DnfClause {
                    include: vec![],
                    exclude: vec![name.clone()],
                }]),
                Expression::Not(double_inner) => {
                    // !!X = X
                    to_dnf(double_inner)
                }
                Expression::And(left, right) => {
                    // !(A & B) = !A | !B
                    let negated = Expression::Or(
                        Box::new(Expression::Not(left.clone())),
                        Box::new(Expression::Not(right.clone())),
                    );
                    to_dnf(&negated)
                }
                Expression::Or(left, right) => {
                    // !(A | B) = !A & !B
                    let negated = Expression::And(
                        Box::new(Expression::Not(left.clone())),
                        Box::new(Expression::Not(right.clone())),
                    );
                    to_dnf(&negated)
                }
            }
        }
        Expression::Or(left, right) => {
            let mut left_clauses = to_dnf(left)?;
            let right_clauses = to_dnf(right)?;
            if left_clauses.len() + right_clauses.len() > MAXIMUM_CLAUSES {
                return Err("tag expression is too complex".to_string());
            }
            left_clauses.extend(right_clauses);
            Ok(left_clauses)
        }
        Expression::And(left, right) => {
            let left_clauses = to_dnf(left)?;
            let right_clauses = to_dnf(right)?;
            if left_clauses.len() * right_clauses.len() > MAXIMUM_CLAUSES {
                return Err("tag expression is too complex".to_string());
            }
            // Cartesian product: distribute AND over OR
            let mut result = Vec::new();
            for left_clause in &left_clauses {
                for right_clause in &right_clauses {
                    let mut merged = DnfClause {
                        include: left_clause.include.clone(),
                        exclude: left_clause.exclude.clone(),
                    };
                    merged.include.extend(right_clause.include.clone());
                    merged.exclude.extend(right_clause.exclude.clone());
                    result.push(merged);
                }
            }
            Ok(result)
        }
    }
}

fn dnf_to_json(clauses: Vec<DnfClause>) -> Value {
    let array: Vec<Value> = clauses
        .into_iter()
        .map(|clause| {
            json!({
                "include": clause.include,
                "exclude": clause.exclude,
            })
        })
        .collect();
    Value::Array(array)
}

/// Parse a tag expression string into a JSONB-compatible value in DNF form.
///
/// Returns `Ok(None)` for empty/whitespace-only input (filter skipped).
/// Returns `Ok(Some(json))` with DNF encoded as `[{"include":["t1"],"exclude":["t2"]}, ...]`.
/// Returns `Err(message)` for parse errors.
pub fn parse_tag_expression(input: &str) -> Result<Option<Value>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let tokens = tokenize(trimmed)?;
    if tokens.is_empty() {
        return Ok(None);
    }
    if tokens.len() > MAXIMUM_TOKENS {
        return Err("tag expression is too long".to_string());
    }

    let mut parser = Parser::new(tokens);
    let ast = parser.parse_expression()?;

    if parser.position < parser.tokens.len() {
        return Err("unexpected token after expression".to_string());
    }

    let clauses = to_dnf(&ast)?;
    Ok(Some(dnf_to_json(clauses)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tag() {
        let result = parse_tag_expression("action").unwrap().unwrap();
        assert_eq!(result, json!([{"include": ["action"], "exclude": []}]));
    }

    #[test]
    fn test_or_expression() {
        let result = parse_tag_expression("action | comedy").unwrap().unwrap();
        assert_eq!(
            result,
            json!([
                {"include": ["action"], "exclude": []},
                {"include": ["comedy"], "exclude": []}
            ])
        );
    }

    #[test]
    fn test_and_expression() {
        let result = parse_tag_expression("action & comedy").unwrap().unwrap();
        assert_eq!(
            result,
            json!([{"include": ["action", "comedy"], "exclude": []}])
        );
    }

    #[test]
    fn test_not_expression() {
        let result = parse_tag_expression("!horror").unwrap().unwrap();
        assert_eq!(result, json!([{"include": [], "exclude": ["horror"]}]));
    }

    #[test]
    fn test_complex_expression() {
        let result = parse_tag_expression("action | (comedy & !horror)")
            .unwrap()
            .unwrap();
        assert_eq!(
            result,
            json!([
                {"include": ["action"], "exclude": []},
                {"include": ["comedy"], "exclude": ["horror"]}
            ])
        );
    }

    #[test]
    fn test_precedence_and_binds_tighter_than_or() {
        // "a b & c" should be "a | (b AND c)" since & binds tighter than space
        let result = parse_tag_expression("a b & c").unwrap().unwrap();
        assert_eq!(
            result,
            json!([
                {"include": ["a"], "exclude": []},
                {"include": ["b", "c"], "exclude": []}
            ])
        );
    }

    #[test]
    fn test_double_negation() {
        let result = parse_tag_expression("!!action").unwrap().unwrap();
        assert_eq!(result, json!([{"include": ["action"], "exclude": []}]));
    }

    #[test]
    fn test_de_morgan_not_and() {
        // !(a & b) = !a | !b
        let result = parse_tag_expression("!(a & b)").unwrap().unwrap();
        assert_eq!(
            result,
            json!([
                {"include": [], "exclude": ["a"]},
                {"include": [], "exclude": ["b"]}
            ])
        );
    }

    #[test]
    fn test_de_morgan_not_or() {
        // !(a | b) = !a & !b
        let result = parse_tag_expression("!(a | b)").unwrap().unwrap();
        assert_eq!(result, json!([{"include": [], "exclude": ["a", "b"]}]));
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(parse_tag_expression("").unwrap(), None);
        assert_eq!(parse_tag_expression("   ").unwrap(), None);
    }

    #[test]
    fn test_parse_error_missing_closing_paren() {
        assert!(parse_tag_expression("(action").is_err());
    }

    #[test]
    fn test_parse_error_unexpected_end() {
        assert!(parse_tag_expression("action |").is_err());
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        assert!(parse_tag_expression("| action").is_err());
    }

    #[test]
    fn test_spaces_are_implicit_or() {
        // "science fiction" is treated as "science | fiction"
        let result = parse_tag_expression("Science Fiction").unwrap().unwrap();
        assert_eq!(
            result,
            json!([
                {"include": ["science"], "exclude": []},
                {"include": ["fiction"], "exclude": []}
            ])
        );
    }

    #[test]
    fn test_dotted_tag_name() {
        let result = parse_tag_expression("epic.movie").unwrap().unwrap();
        assert_eq!(result, json!([{"include": ["epic.movie"], "exclude": []}]));
    }

    #[test]
    fn test_oversized_expressions_are_rejected() {
        // deep nesting used to overflow the stack
        let nested = format!("{}action{}", "(".repeat(10_000), ")".repeat(10_000));
        assert!(parse_tag_expression(&nested).is_err());
        // every (x|y)& group doubles the amount of clauses
        let exploding = (0..7)
            .map(|index| format!("(a{index}|b{index})"))
            .collect::<Vec<_>>()
            .join("&");
        assert!(parse_tag_expression(&exploding).is_err());
        let accepted = (0..6)
            .map(|index| format!("(a{index}|b{index})"))
            .collect::<Vec<_>>()
            .join("&");
        assert!(parse_tag_expression(&accepted).is_ok());
    }

    #[test]
    fn test_and_not_combined() {
        let result = parse_tag_expression("action & !horror & comedy")
            .unwrap()
            .unwrap();
        assert_eq!(
            result,
            json!([{"include": ["action", "comedy"], "exclude": ["horror"]}])
        );
    }
}
