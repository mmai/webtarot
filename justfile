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
compile-client:
  trunk build --release
compile: compile-server compile-client
read_debugbot file:
  cargo run -p webtarot_protocol -- {{file}}
