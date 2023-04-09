dev:
	PROXYPATH=http://localhost \
	PRESETS_API_URL= \
	DB_URI="mongodb://127.0.0.1:27017" \
	DB_NAME="presets" \
	cargo run

release:
	PROXYPATH=http://localhost \
	PRESETS_API_URL=
	DB_URI="mongodb://127.0.0.1:27017" \
	DB_NAME="presets" \
	./target/release/imgproxyproxy
	