#[cfg(test)]
mod tests {
    // use imuc_ast::*;
    use imuc_lexer::*;
    use imuc_parser::*;
    use imuc_rules::*;

    macro_rules! test_item {
        (parse $name: literal, $content: expr) => {{
            let content = $content;
            let mut parser = Parser::new(FileReader::new(
                $name,
                content,
                Reader::new(content.chars()),
            ));
            let result = rules::ItemRule.parse(&mut parser);
            //assert!(parser.is_empty().expect("parser should consume all tokens"));
            result
        }};
        (error $name: literal, $content: expr) => {
            assert!(test_item!(parse $name, $content).is_err());
        };
        (some $name: literal, $content: expr) => {
            assert!(test_item!(parse $name, $content).is_ok_and(|ast| ast.is_some()));
        };
        (none $name: literal, $content: expr) => {
            assert!(test_item!(parse $name, $content).is_ok_and(|ast| ast.is_none()));
        };
    }

    #[test]
    fn parse_item() {
        test_item!(error "parse_item: EOF", "pub");
        test_item!(some "parse_item: fun", "fun dog() {}");
    }
}
