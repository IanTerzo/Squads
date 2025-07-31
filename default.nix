{ rustPlatform
, pkg-config
, makeWrapper
, llvmPackages
, mold
, lib
, openssl
, wayland
, libGL
, libxkbcommon
, ...
}:

rustPlatform.buildRustPackage {
  name = "squads";
  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  nativeBuildInputs = [
    rustPlatform.bindgenHook
    pkg-config
    makeWrapper
    llvmPackages.clang
    mold
  ];
  buildInputs = [
    openssl
    wayland
  ];
  postInstall = ''
    mkdir -p $out/share/applications
    cp ${./resources/squads.desktop} $out/share/applications/squads.desktop
    substituteInPlace $out/share/applications/squads.desktop \
      --replace "Exec=Squads" "Exec=$out/bin/Squads"

    mkdir -p $out/share/icons/hicolor/scalable/apps
    cp ${./resources/squads.svg} $out/share/icons/hicolor/scalable/apps/squads.svg
  '';
  postFixup = ''
    wrapProgram $out/bin/Squads --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [openssl wayland libGL libxkbcommon]}
  '';
  shellHook = ''
    export LD_LIBRARY_PATH=${lib.makeLibraryPath [openssl wayland libGL libxkbcommon]}
  '';
  meta = with lib; {
    description = "Alternative Teams client";
    longDescription = ''
    Squads aims to be a minimalist alternative to the official Teams client. Because of major API discrepancies in Teams, Squads only works with accounts that are part of an
    organization, such as school or work accounts.
    '';
    homepage = "https://github.com/IanTerzo/Squads";
    license = licenses.gpl3Plus;
    platforms = platforms.unix;
    mainProgram = "Squads";
  };
}
