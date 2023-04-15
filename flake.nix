{
  description = "Musawarah dev environment setup";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {nixpkgs, flake-utils, rust-overlay, ...}:
    flake-utils.lib.eachDefaultSystem (system: 
      let 
        pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
      in
    with pkgs; {
      devShell = mkShell {
          packages = [
            # general utilities
            exa
            fd
            bat
            zoxide
            helix
            
            postgresql
            docker
          ];
          
          buildInputs = [
            # backend
            sqlx-cli
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            })

            # frontend
            nodePackages_latest.pnpm
            nodejs

            # needed for openssl dependant packages
            openssl
            pkg-config
          ];

          shellHook = ''
            # cool aliases
            alias ls='exa --time-style=long-iso --group-directories-first --icons --no-permissions --no-user -l --git'
            alias ll="exa --time-style=long-iso --group-directories-first --icons -la"
            alias find=fd
            alias cat=bat

            # start dev database if available, if not create, and run it on port 5445
            docker start musawarah-dev || \
              docker run \
              --name musawarah-dev \
              -p 5445:5432 \
              -e POSTGRES_PASSWORD=musawarah-dev \
              -d postgres

            # add DATABASE_URL to .env file if not already there
            grep DATABASE_URL .env || echo "DATABASE_URL=postgres://postgres:musawarah-dev@localhost:5445" >> .env

            # export environment variables
            export $(cat .env)

            # run zoxide for directory aliases
            eval "$(zoxide init bash)"
          '';
        };

      formatter.x86_64-linux = legacyPackages.${system}.nixpkgs-fmt;
    });
}
