dev:
	IMGPROXY_URL=https://cdn-app.sberdevices.ru/asset \
	cargo run

release:
	IMGPROXY_URL=https://cdn-app.sberdevices.ru/asset \
	./target/release/imgproxyproxy
	