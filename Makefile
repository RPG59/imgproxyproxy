dev:
	PROXYPATH=http://localhost \
	DB_URI="mongodb://127.0.0.1:27017" \
	DB_NAME="presets" \
	DB_COLLECTION="presets" \
	cargo run

release:
	PROXYPATH=http://localhost \
	DB_URI="mongodb://127.0.0.1:27017" \
	DB_NAME="presets" \
	DB_COLLECTION="presets" \
	./target/release/imgproxyproxy
	