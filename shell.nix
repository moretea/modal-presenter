with (import <unstable> {});
with pkgs;
mkShell {
  name = "cn-game-dev";
  buildInputs = [ rustc cargo SDL2 SDL2_image SDL2_ttf binutils xdo xdotool];
}
