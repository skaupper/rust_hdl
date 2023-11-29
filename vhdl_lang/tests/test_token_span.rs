use vhdl_lang_macros::with_token_span;

#[test]
fn test_token_span() {
    #[with_token_span]
    struct A<'b> {
        item: &'b str,
    }
}
