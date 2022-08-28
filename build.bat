SET RUSTFLAGS=-C target-cpu=native
cargo build --release

cd engine
cargo build --release
cd ..

set RUSTFLAGS=
engine\target\release\rustypigeon.exe