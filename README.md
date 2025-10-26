# Homage

Simple and effective dotfiles manager for your home.

Installs one or more dotfiles directories into the desired location.
Files are installed as symlinks in the target directory and will follow the
relative directory structure as in the source directory.

An example dotfiles can look as follows:

```sh
dotfiles
└── .config
    └── application
        └── config
```

Running `homage install dotfiles` (or `homage install .` if it's your working directory)
will install the file `config` in the `application` directory under
`$HOME/.config`.

Since only files are symlinked it is possible to install multiple "profiles" by
running the program several times on different directories mirroring the
resulting structure you want.

An example of this can look as follows:

```sh
dotfiles
├── profile1
│   └── .config
│       └── application
│           └── config
└── profile2
    └── .config
        └── application
            └── extra_config
```

To install profile 1 run `homage install profile1` and similarly
`homage install profile2` for profile 2. As long as the files are different they
will happily coexist in you home directory as separate symlinked files.

If the exact same file is specified in multiple profiles it will be the last
profile installed that "wins", .i.e. existing symlinks will be removed.
