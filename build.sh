
cargo build --release
mkdir -p dylibs/ && rm -rfv dylibs/* && ls target/release/ && cp -v target/release/*.* dylibs/
