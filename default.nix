with import <nixpkgs> {};
let
  zxingNet = builtins.fetchTarball
    https://github.com/micjahn/ZXing.Net/archive/6e1f7264e81459f0659296ec9ed2610ffd0a4c88.tar.gz;
in
stdenv.mkDerivation {
  name = "qrdese";

  BLACKBOX_TESTS = "${zxingNet}/Source/test/data/blackbox/qrcode-1";
}