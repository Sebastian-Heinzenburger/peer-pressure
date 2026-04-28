{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
  };

  outputs =
    { nixpkgs, ... }:
    {
      devShells.x86_64-linux.default =
        let
          pkgs = nixpkgs.legacyPackages.x86_64-linux;
        in
        pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            bacon
            clippy
            rust-analyzer
          ];
          shellHook = ''
            export SHELL=$(which zsh)
            export PROJECT=$(basename $(pwd))
            tmux has-session -t $PROJECT 2>/dev/null && tmux attach -t $PROJECT || \
            tmux new-session -d -s $PROJECT 'vim src/main.rs' \; \
            split-window -v \; \
            send-keys -t 2 'bacon' C-m \; \
            split-window -h \; \
            send-keys -t 3 'git status' C-m \; \
            attach-session -t $PROJECT
          '';
        };
    };
}
