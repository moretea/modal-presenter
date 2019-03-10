{ stdenv, rustPlatform, SDL2, SDL2_image, SDL2_ttf, xdo, xdotool, binutils, pkgconfig }:
rustPlatform.buildRustPackage rec {
  name = "modal-presenter-${version}";
  version = "0.1.0";

  src = ./.;
  cargoSha256 = "069shq19xiv9z51lxv5bpiadd12dzm28lppamiq6qybkjwy8kzs8";

  buildInputs = [ SDL2 SDL2_image SDL2_ttf binutils xdo xdotool pkgconfig];

  meta = with stdenv.lib; {
    description = "Tool to automate live coding presentations";
    homepage = https://github.com/moretea/modal-presenter;
    license = licenses.gpl3;
    maintainers = [ maintainers.moretea ];
    platforms = platforms.linux;
  };
}
