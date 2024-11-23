#[cfg(test)]
pub mod tests {
    use imuc_lexer::*;

    #[test]
    fn test_aho_corasick() {
        let mut ac = AhoCorasickBuilder::default();
        ac.insert("dog", "cute");
        ac.insert("doggy", "cuter");
        ac.insert("cat", "also cute");
        ac.insert("kitty", "extra cute");
        ac.insert("cog", "not an animal");
        ac.insert("kde", "good");
        ac.insert("derive", "macro");
        let ac = ac.build();
        let query = |s: &str| {
            let mut pos = 0;
            for ch in s.chars() {
                pos = ac.query(pos, ch);
            }
            ac.finish(pos).cloned()
        };
        assert_eq!(Some("cute"), query("dog"));
        assert_eq!(Some("not an animal"), query("cog"));
        assert_eq!(Some("good"), query("kde"));
        assert_eq!(Some("macro"), query("derive"));
        assert_eq!(Some("cuter"), query("doggy"));
        assert_eq!(None, query("C++"));
        assert_eq!(None, query("doge"));
        assert_eq!(None, query("k"));
        assert_eq!(None, query("ca"));
        assert_eq!(None, query("catastrophe"));
    }

    #[test]
    fn test_lexer() {
        use token::*;

        let text = "a =0x1b0F;\n  Ty+  \"hello\"//Comment\n1.0e5";
        let reader = Reader::new(text.chars());
        let mut pos = 0;
        let tokens = reader
            .map(|token| {
                let str = &text[pos..pos + token.len];
                pos += token.len;
                (token.kind, str)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            tokens,
            [
                (TokenKind::Ident(Ident::Value), "a"),
                (TokenKind::Spacing(Spacing::Indent), " "),
                (TokenKind::Symbol(Symbol::Assign), "="),
                (TokenKind::Literal(Literal::Integer), "0x1b0F"),
                (TokenKind::Semicolon, ";"),
                (TokenKind::Spacing(Spacing::LineBreak), "\n"),
                (TokenKind::Spacing(Spacing::Indent), "  "),
                (TokenKind::Ident(Ident::Type), "Ty"),
                (TokenKind::BinOp(BinOp::Add), "+"),
                (TokenKind::Spacing(Spacing::Indent), "  "),
                (TokenKind::Literal(Literal::String), "\"hello\""),
                (TokenKind::Comment(Comment::Comment), "//Comment"),
                (TokenKind::Spacing(Spacing::LineBreak), "\n"),
                (TokenKind::Literal(Literal::Float), "1.0e5"),
            ]
        );
    }
}
