use super::Result;

enum ParseState {
    Column,
}

fn parse_csv(input: &str) -> Result<Vec<Vec<String>>> {
    let state = ParseState::Column;
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

#[cfg(test)]
mod test {
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
}
