let
    pinned = fromTOML (builtins.readFile ./pinned.toml);
    tarball = fetchTarball pinned;
in
    import tarball
