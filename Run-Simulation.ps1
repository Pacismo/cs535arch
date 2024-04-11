$benchmark = "matrix"

$configuration = "./benchmarks/$benchmark.toml"
$codefile = "./benchmarks/$benchmark.asm"
$binfile = "./benchmarks/$benchmark.bin"

$cargo_install_dir = "./gui/bin/Release/net8.0-windows/"

cargo run --bin seis-asm -- $codefile -o $binfile

cargo install --path ./seis-sim --root ./$cargo_install_dir
cargo install --path ./seis-asm --root ./$cargo_install_dir
cargo install --path ./seis-disasm --root ./$cargo_install_dir

./gui/bin/Release/net8.0-windows/gui.exe $configuration
