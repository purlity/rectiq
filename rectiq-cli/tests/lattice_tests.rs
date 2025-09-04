use rectiq_cli::pipeline::{RegionClass, Scan};

#[test]
fn lattice_precedence_and_cover() {
    let src = "/*c*/\n\"s\" ?";
    let scan = Scan::new(src).lex().build_skeleton().build_lattice();
    let lattice = scan.inner.lattice;
    assert_eq!(lattice.regions.first().unwrap().class, RegionClass::Comment);
    assert_eq!(lattice.regions.last().unwrap().end, src.len());
    for w in lattice.regions.windows(2) {
        assert_eq!(w[0].end, w[1].start);
    }
}

#[test]
fn lattice_bracket_error() {
    let src = "}";
    let scan = Scan::new(src).lex().build_skeleton().build_lattice();
    assert_eq!(
        scan.inner.lattice.regions[0].class,
        RegionClass::BracketError
    );
}
