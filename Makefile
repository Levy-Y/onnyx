build:
	cargo build

build-bin:
	espflash save-image --chip esp32s3 target/xtensa-esp32s3-espidf/debug/Onnyx target/xtensa-esp32s3-espidf/debug/firmware.bin

clean:
	cargo clean

flash:
	cargo run

erase:
	espflash erase-flash

.PHONY: build build-bin clean flash erase