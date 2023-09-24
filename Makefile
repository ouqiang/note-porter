.PHONY: release
release:
	cross build --release --target x86_64-pc-windows-gnu
	cross build --release --target x86_64-apple-darwin
	cross build --release --target x86_64-unknown-linux-gnu


