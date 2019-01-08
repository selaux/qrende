with import <nixpkgs> {};
let
  zxingNet = builtins.fetchTarball
    https://github.com/micjahn/ZXing.Net/archive/6e1f7264e81459f0659296ec9ed2610ffd0a4c88.tar.gz;
  allBlackboxTests = builtins.concatStringsSep ";" (map
    (i: "${i}:${zxingNet}/Source/test/data/blackbox/qrcode-${i}")
    [ "1" "2" "3" "4" "5" "6" ]
  );
in
stdenv.mkDerivation {
  name = "qrdese";

  BLACKBOX_TESTS = "${allBlackboxTests}";
  TEST_FONT = "${opensans-ttf}/share/fonts/truetype/OpenSans-Bold.ttf";
}