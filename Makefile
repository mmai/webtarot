build: compile assemble

local:
	cd container && nix flake lock --update-input nixpkgs --update-input webtarot && cd -
	sudo nixos-container destroy webtarot
	sudo nixos-container create webtarot --flake ./container/
	sudo nixos-container start webtarot
dev:
	NIXPKGS_ALLOW_INSECURE=1 nix develop --impure
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
	firefox -p tarot1 http://127.0.0.1:8001 &
	firefox -p tarot2 http://127.0.0.1:8001 &
	firefox -p tarot3 http://127.0.0.1:8001 &
	firefox -p tarot4 http://127.0.0.1:8001 &
	firefox -p tarot5 http://127.0.0.1:8001 &

docker:
	nix build .#webtarot-docker
	docker push mmai/webtarot

nixcache: build
	NIXPKGS_ALLOW_INSECURE=1 nix build --impure .#webtarot
	cachix push mmai ./result 
	nix build .#webtarot-front
	cachix push mmai ./result 
