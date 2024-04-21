.PHONY: convertcsv
convertcsv:
	@cargo run -- csv -i assets/juventus.csv --format json

.PHONY: genpass
genpass:
	@cargo run -- genpass -l 32

# stdin 输入完后需要 ctrl + d 才能
.PHONY: base64encode
base64encode:
	@cargo run -- base64 encode --format urlsafe


.PHONY: base64encodefile
base64encodefile:
	@cargo run -- base64 encode --format urlsafe -i Cargo.toml > tmp.b64

.PHONY: base64decodefile
base64decodefile:
	@cargo run -- base64 decode --format urlsafe -i tmp.b64

.PHONY: test
test:
	@RUST_BACKTRACE=1 cargo nextest run

.PHONY: sign-blake3
sign-blake3:
	@cargo run -- text sign -k fixtures/blake3.txt

.PHONY: sign-ed25519
sign-ed25519:
	@cargo run -- text sign --format ed25519 -k fixtures/ed25519.sk


.PHONY: generate
generate:
	@cargo run -- text generate -o fixtures --format blake3
	@cargo run -- text generate -o fixtures --format ed25519

.PHONY: http
http:
	@RUST_LOG=info cargo run -- http serve
