build: compile assemble

client:
	cd webtarot_client && yarn && yarn run start:dev
.PHONY: client

server:
	cd webtarot_server && RUST_LOG=debug cargo run
.PHONY: server

server-reload:
	cd webtarot_server && RUST_LOG=debug systemfd --no-pid -s http::8002 -- cargo watch -x run
.PHONY: server-reload

extracti18n:
	cd webtarot_client && cargo i18n

compile:
	cd webtarot_client && cargo i18n && yarn && yarn run build && yarn run css
	cd webtarot_server && cargo build --release
.PHONY: compile

assemble:
	rm -rf dist
	mkdir dist
	cp -R webtarot_client/dist/ dist/public
	cp target/release/webtarot_server dist/
.PHONY: assemble

runclients:
	firefox -p tarot1&
	firefox -p tarot2&
	firefox -p tarot3&
	firefox -p tarot4&
	firefox -p tarot5&
