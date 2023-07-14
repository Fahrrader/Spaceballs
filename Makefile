build:
	rm -rf public
	mkdir public
	cargo build --target=wasm32-unknown-unknown
	wasm-bindgen --out-dir web/ --target web target/wasm32-unknown-unknown/debug/cosmic-spaceball-tactical-action-arena.wasm
	cd web; npm install; npm run build
	cp -r web/dist/* public/
