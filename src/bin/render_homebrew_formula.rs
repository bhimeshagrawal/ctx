fn main() {
    let mut repository = None;
    let mut version = None;
    let mut sha256 = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repository" => repository = args.next(),
            "--version" => version = args.next(),
            "--sha256" => sha256 = args.next(),
            other => panic!("unexpected argument: {other}"),
        }
    }

    let formula = ctx::release::render_homebrew_formula(
        &repository.expect("missing --repository"),
        &version.expect("missing --version"),
        &sha256.expect("missing --sha256"),
    );
    print!("{formula}");
}
