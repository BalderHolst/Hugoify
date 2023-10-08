{ 
    pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/eabc38219184cc3e04a974fe31857d8e0eac098d.tar.gz") {},
    old_pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/refs/tags/22.05.tar.gz") {},
    unstable_pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") {}
}:

pkgs.mkShell {
  packages = [
    unstable_pkgs.cargo
    old_pkgs.glibc
  ];
}
