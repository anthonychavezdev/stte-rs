{ pkgs, lib, config, inputs, ... }:

{
  languages.rust.enable = true;

  # https://devenv.sh/basics/
  enterShell = ''
    devenv --version
    rustc --version
    cargo --version
    git --version # Use packages
  '';
}
