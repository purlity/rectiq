use rectiq_cli::pipeline::Scan;
use proptest::prelude::*;

proptest! {
    #[test]
    fn tokens_cover_all(input in ".*") {
        let scan = Scan::new(&input).lex();
        let tokens = &scan.inner.tokens;
        prop_assert_eq!(tokens.first().unwrap().start, 0);
        prop_assert_eq!(tokens.last().unwrap().end, input.len());
        for w in tokens.windows(2) {
            prop_assert_eq!(w[0].end, w[1].start);
        }
    }

    #[test]
    fn lattice_covers_all(input in ".*") {
        let scan = Scan::new(&input).lex().build_skeleton().build_lattice();
        let regions = &scan.inner.lattice.regions;
        if !regions.is_empty() {
            prop_assert_eq!(regions.first().unwrap().start, 0);
            prop_assert_eq!(regions.last().unwrap().end, input.len());
            for w in regions.windows(2) {
                prop_assert_eq!(w[0].end, w[1].start);
            }
        }
    }
}
