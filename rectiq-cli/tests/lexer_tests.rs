use rectiq_cli::pipeline::{TokKind, lex};

#[test]
fn lexer_token_coverage() {
    let src = "{}[]//c\n\"s\" 123 ?";
    let tokens = lex(src);
    assert_eq!(tokens.first().unwrap().start, 0);
    assert_eq!(tokens.last().unwrap().end, src.len());
    for w in tokens.windows(2) {
        assert_eq!(w[0].end, w[1].start);
    }
    assert!(tokens.iter().any(|t| t.kind == TokKind::Comment));
    assert!(tokens.iter().any(|t| t.kind == TokKind::StringLit));
    assert!(tokens.iter().any(|t| t.kind == TokKind::NumberLit));
    assert!(tokens.iter().any(|t| t.kind == TokKind::Unknown));
}
