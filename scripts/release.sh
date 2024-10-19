#!/usr/bin/env bash

echo "update version number in the _Cargo.toml_ files of updated projects and those which depends on them"
read ok

echo "compiling release version..."
make
# XXX : le build du front peut être cassé si lancé depuis le shell nix à cause de la version de rustc : vérifier les erreurs du debut. Il est peut-être nécessaire de lancer séparément le build du front et le build du serveur (celui-ci devant être lancé depuis le shell nix pour profiter des lib ssl nécessaires pour webtarot_bot)

FAKE=$(echo "foobar" | sha256sum)
echo "edit _flake.nix_ : update version number and put this fake cargoSha256 : $FAKE"
read ok

nix build .#webtarot
echo "Copy the correct cargoSha256 in _flake.nix_ from the preceding compilation error"
read ok

git flow release
git push && git push --tags

nix build .#webtarot
cachix push mmai ./result
nix copy --to ssh://root@rhumbs.fr ./result

make docker
