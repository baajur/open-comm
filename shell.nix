let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  mozpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rustChannel = (mozpkgs.rustChannelOf { date = "2020-08-29"; channel = "nightly"; }).rust.override {
    targets = [ "x86_64-unknown-linux-musl" "wasm32-unknown-unknown" ];
    extensions = [
      "rust-src"
      "rls-preview"
      "clippy-preview"
      "rustfmt-preview"
    ];
  };
in
  with mozpkgs;
  mkShell {
    name = "moz_overlay_shell";
    buildInputs = [
      rustChannel
      cargo-edit
      diesel-cli
      elmPackages.elm
      elmPackages.elm-test
      elmPackages.elm-format
      ephemeralpg
    ];
    shellHook = ''
      if ! docker ps --format '{{.Names}}' | grep -w open-comm-dev-db &>/dev/null; then
        docker start open-comm-dev-db \
        || docker run -d \
                      --name open-comm-dev-db \
                      -e POSTGRES_HOST_AUTH_METHOD=trust \
                      -e POSTGRES_USER=postgres \
                      -v open-comm-dev-db:/var/lib/postgresql/data \
                      postgres
      fi
      dbip=$(
        docker inspect \
          -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' \
          open-comm-dev-db
      )
      export DATABASE_URL="postgres://postgres@$dbip:5432"
      export ROCKET_DATABASES="{user_db={url=\"$DATABASE_URL\"}}"
      export RUST_LOG=info
      '';
  }

