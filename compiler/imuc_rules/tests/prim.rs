#[cfg(test)]
mod tests {
    use imuc_ast::*;
    use imuc_lexer::*;
    use imuc_parser::*;
    use imuc_rules::*;

    macro_rules! test_prim {
        ($name: literal, $content: expr, $prim: pat) => {
            test_prim!($name, $content, $prim, 0, 0)
        };
        ($name: literal, $content: expr, $prim: pat, $target: expr, $cmp: expr) => {
            let content = $content;
            let mut parser = Parser::new(FileReader::new(
                $name,
                content,
                Reader::new(content.chars()),
            ));
            let ast = rules::PrimRule
                .parse(&mut parser)
                .expect("no errors should occur");
            assert!(parser.is_empty().expect("parser should consume all tokens"));
            if let Some($prim) = ast {
                assert_eq!($target, $cmp);
            } else {
                panic!(
                    "failed to match ast to the required primitive {}",
                    stringify!($prim)
                );
            }
        };
    }

    #[test]
    fn parse_integer() {
        test_prim!(
            "parse_integer: dec",
            "4321",
            prim::Prim::Integer(prim::Integer::I64(4321))
        );
        test_prim!(
            "parse_integer: hex",
            "0x7890",
            prim::Prim::Integer(prim::Integer::I64(0x7890))
        );
        test_prim!(
            "parse_integer: bin",
            "0b10011011",
            prim::Prim::Integer(prim::Integer::I64(0b10011011))
        );
    }

    #[test]
    fn parse_float() {
        test_prim!(
            "parse_float: normal",
            "5.7812",
            prim::Prim::Float(prim::Float::F32(5.7812))
        );
        test_prim!(
            "parse_float: exponent",
            "2.34e4",
            prim::Prim::Float(prim::Float::F32(2.34e4))
        );
        test_prim!(
            "parse_float: inf",
            "inf",
            prim::Prim::Float(prim::Float::F32(f32::INFINITY))
        );
    }

    #[test]
    fn parse_string() {
        test_prim!(
            "parse_string: simple",
            "\"hello, world!\"",
            prim::Prim::String(value),
            value,
            "hello, world!"
        );
        test_prim!(
            "parse_string: slashes",
            "\"hello,\\n\\\"world!\\\"\"",
            prim::Prim::String(value),
            value,
            "hello,\n\"world!\""
        );
        test_prim!(
            "parse_string: multi-string",
            "\"\"\"Multiple\nLines\nHere\n\"quotes\" are available\"\"\"",
            prim::Prim::String(value),
            value,
            "Multiple\nLines\nHere\n\"quotes\" are available"
        );
        test_prim!(
            "parse_string: many quotes",
            "\"\"\"Run!\"\"\"\"\"\"",
            prim::Prim::String(value),
            value,
            "Run!\"\"\""
        );
    }
}
