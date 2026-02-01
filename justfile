# Builds the firmware binary
[group('core-build')]
build:
	cargo build

# Converts the compiled firmware to a .bin format
[group('core-build')]
[group('esp-utils')]
build-bin: build
	espflash save-image --chip esp32s3 target/xtensa-esp32s3-espidf/debug/Onnyx target/xtensa-esp32s3-espidf/debug/firmware.bin

# 
[group('esp-utils')]
flash monitor="": build
        espflash flash target/xtensa-esp32s3-espidf/debug/Onnyx {{ if monitor == "-m" { "--monitor" } else { "" } }}

# Cleans the cargo build cache
[group('cleanup')]
clean:
	cargo clean

# Erases the connected esp32 board
[group('cleanup')]
[group('esp-utils')]
erase:
	espflash erase-flash
