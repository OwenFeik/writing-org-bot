use std::path::Path;

use crate::msg;

use super::Result;

type Csv = Vec<Vec<String>>;

enum ParseState {
    Column,
    Escape,
    Quote,
}

pub fn parse_csv(input: &str) -> Result<Csv> {
    let mut prev = None;
    let mut state = ParseState::Column;
    let mut csv = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();

    for c in input.chars() {
        match state {
            ParseState::Column => match c {
                ',' => {
                    row.push(field);
                    field = String::new();
                }
                '\n' => {
                    row.push(field);
                    field = String::new();
                    csv.push(row);
                    row = Vec::new();
                }
                '\\' => {
                    prev = Some(ParseState::Column);
                    state = ParseState::Escape;
                }
                '"' => {
                    state = ParseState::Quote;
                }
                _ => field.push(c),
            },
            ParseState::Escape => {
                match c {
                    'n' => {
                        field.push('\n');
                    }
                    _ => {
                        field.push(c);
                    }
                }
                state = prev.take().unwrap();
            }
            ParseState::Quote => match c {
                '\\' => {
                    prev = Some(ParseState::Quote);
                    state = ParseState::Escape;
                }
                '"' => {
                    state = ParseState::Column;
                }
                _ => field.push(c),
            },
        }
    }

    if !field.is_empty() {
        row.push(field);
    }
    if !row.is_empty() {
        csv.push(row);
    }

    Ok(csv)
}

fn format_col(col: &str) -> String {
    // Escape quotes.
    let text = col.replace('"', "\\\"");
    if text.contains(',') {
        format!("\"{}\"", col)
    } else {
        text
    }
}

pub fn format_csv(csv: &Csv) -> String {
    csv.iter()
        .map(|row| {
            row.iter()
                .map(|col| format_col(col))
                .reduce(|acc, e| format!("{acc},{e}"))
        })
        .reduce(|acc, e| match (acc, e) {
            (Some(acc), Some(e)) => Some(format!("{acc}\n{e}")),
            (None, Some(e)) => Some(e),
            (Some(acc), None) => Some(acc),
            (None, None) => None,
        })
        .flatten()
        .unwrap_or("".to_string())
}

pub async fn load_csv<P: AsRef<Path>>(file: P) -> Result<Csv> {
    let csv = tokio::fs::read_to_string(file).await.map_err(msg)?;
    parse_csv(&csv)
}

pub async fn write_csv<P: AsRef<Path>>(csv: &Csv, file: P) -> Result<()> {
    tokio::fs::write(file, format_csv(csv)).await.map_err(msg)
}

#[cfg(test)]
mod test {
    use crate::csv::format_csv;

    use super::parse_csv;

    #[test]
    fn test_parse() {
        let csv = parse_csv("a,b,c,d\n1,2,3,4\n").unwrap();
        assert_eq!(
            csv,
            vec![
                vec![
                    "a".to_string(),
                    "b".to_string(),
                    "c".to_string(),
                    "d".to_string()
                ],
                vec![
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                    "4".to_string()
                ]
            ]
        );
    }

    #[test]
    fn test_quote() {
        let csv = parse_csv("\"a,b,\\\"c\\\",d\",b").unwrap();
        assert_eq!(csv, vec![vec!["a,b,\"c\",d".to_string(), "b".to_string()]])
    }

    #[test]
    fn test_format() {
        let csv = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        assert_eq!(format_csv(&csv), "a,b\nc,d");
        assert_eq!(csv, parse_csv(&format_csv(&csv)).unwrap());
    }
}
