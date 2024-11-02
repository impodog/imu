#[cfg(test)]
pub mod tests {
    use imuc_lexer::*;

    #[test]
    fn test_reader() {
        const STR: &str = "Dogs🐶 are cu✅te";
        let mut reader = Reader::new(STR.chars());
        assert_eq!(reader.next_char(), 'D');
        assert_eq!(reader.first(), 'o');
        assert_eq!(reader.second(), 'g');
        reader.advance();
        assert_eq!(reader.second(), 's');
        reader.advance_while(|reader| reader.first().len_utf8() == 1);
        assert_eq!(reader.first(), '🐶');
        reader.advance();
        assert_eq!(reader.second(), 'a');
        reader.advance_while(|reader| reader.second().is_alphanumeric());
        assert_eq!(reader.next_char(), 'e');
        reader.advance();
        reader.advance();
        assert_eq!(reader.second(), '✅');
        assert_eq!(reader.first(), 'u');
    }
}
