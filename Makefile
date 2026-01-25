build:
	cargo build

build-esp:
	espflash save-image --chip esp32s3 target/debug/Onnyx firmware.bin

clean:
	cargo clean

.PHONY: build build-esp cleans