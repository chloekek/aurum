let
    pinned = fromTOML (builtins.readFile ./pinned.toml);
    tarball = fetchTarball pinned;
    overlays = [
        (import ../nixpkgs-mozilla)
    ];
in
    import tarball { inherit overlays; }
