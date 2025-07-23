readme:
    @which cargo-readme || just binstall cargo-readme
    cargo readme --output README.md
