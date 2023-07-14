build:
	rm -rf public
	mkdir public
	cargo build --target=wasm32-unknown-unknown --release
	wasm-bindgen --out-dir public/ --target web target/wasm32-unknown-unknown/release/cosmic-spaceball-tactical-action-arena.wasm
	cd web; npm run build
	cp web/*.html public/
