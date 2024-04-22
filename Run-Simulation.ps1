$benchmark = $args[0]
$dependencies = [System.Collections.ArrayList]::new();

if ($args.Length -gt 1) {
    for ($i = 1; i -lt $args.Length; $i++) {
        [void]$dependencies.Add("./benchmarks/$benchmark/$($args[$i])")
    }
}

$configuration = "./benchmarks/$benchmark/$benchmark.toml"
$codefile = "./benchmarks/$benchmark/$benchmark.asm"
$binfile = "./benchmarks/$benchmark/$benchmark.bin"

cargo run --release --bin seis-asm -- $codefile $dependencies -o $binfile

& "./gui/bin/Release/net8.0-windows/gui.exe" $configuration
