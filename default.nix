{ rustPlatform
, pkg-config
, makeWrapper
, lib
, chromedriver
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
  ];
  buildInputs = [
    openssl
    wayland
  ];
  env = {
    CHROMEDRIVER_PATH = chromedriver |> lib.getExe;
  };
  postFixup = ''
    wrapProgram $out/bin/squads-iced --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [wayland libGL libxkbcommon]}
  '';
  shellHook = ''
    export LD_LIBRARY_PATH=${lib.makeLibraryPath [wayland libGL libxkbcommon]}
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
    mainProgram = "squads-iced";
  };
}

