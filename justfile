build: compile assemble

compile: compile-server compile-client

assemble:
	rm -rf dist
	mkdir dist
	cp -R webtarot_client/dist/ dist/public
	cp target/release/webtarot_server dist/

# start a webtarot container with nixos-container
# `boot.enableContainers = true` must be set on local nixos system
local:
	cd container && nix flake lock --update-input nixpkgs --update-input webtarot && cd -
	sudo nixos-container destroy webtarot
	sudo nixos-container create webtarot --flake ./container/
	nixos-container start webtarot
	machinectl

docker-build:
  nix build .#webtarot-docker
docker-run: docker-build
  docker load < ./result
  docker run mmai/webtarot -P
docker-publish: docker-build
  docker push mmai/webtarot

nixcache: build
	NIXPKGS_ALLOW_INSECURE=1 nix build --impure .#webtarot
	cachix push mmai ./result 
	nix build .#webtarot-front
	cachix push mmai ./result 

[working-directory: 'webtarot_server']
server:
	RUST_LOG=debug cargo run

[working-directory: 'webtarot_server']
compile-server:
  cargo build --release

[working-directory: 'webtarot_client']
client:
  trunk serve --open

[working-directory: 'webtarot_client']
compile-client: css
  trunk build --release

[working-directory: 'webtarot_client']
css:
  sass --style compressed scss/webtarot.scss > static/webtarot.css

read_debugbot file:
  cargo run -p webtarot_protocol -- {{file}}

[working-directory: 'webtarot_client']
extracti18n:
	cargo i18n
