# Squads
Squads aims to be a minimalist alternative to the official Microsoft Teams client.
Because of major api differences in Teams, Squads only works with accounts that are part of an organization, such as school or work accounts.

⚠️ Still in early development: expect things to be broken and the code unfinished ⚠️

## Run
```
cargo run
```

## Contributing
If you want to contribute a new feature or a big change, please start a discussion so we can plan its implementation. For small fixes, pull requests are directly accepted. If you don't know what to contribute see the [todos](https://github.com/IanTerzo/Squads/blob/master/TODO.md), to claim one, add it to the in-progress tab with your name.

Feature or improvement ideas are very welcome, feel free to share them in the discussions page.

## Preview


![squads](https://github.com/IanTerzo/Squads/blob/master/images/preview3.png?raw=true)

The official client for comparison...

![squads](https://github.com/IanTerzo/Squads/blob/master/images/teams_preview2.png?raw=true)

## Installation

### NixOS

Squads exposes a Nix flake for local usage. Add the following input to your
system flake.

```nix
squads = {
  url = "github:IanTerzo/Squads";
  # Optionally use your own nixpkgs. Not necessarily future-proof.
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then add the overlay to your `pkgs`.

```nix
pkgs = import nixpkgs {
  inherit system;
  overlays = [
    (import squads)
  ];
};
```

It is also possible to use Squads without flakes by building the default output
of the repository.

## Attribution

- Squads uses icons from [Twemoji](https://github.com/twitter/twemoji). Twemoji is licensed under the MIT license.
- Squads uses icons from [Lucide](https://lucide.dev/). Lucide is licensed under the ISC license.
- The ferris icon used is made by [Karen Rustad Tölva](https://rustacean.net/). (No license)

## Acknowledgments

Thanks to [Brian Stadnicki](https://github.com/BrianStadnicki/opercom-web-app) and Opercom for helping in figuring out some of the API, especially regarding activities, and to [Eion Robb](https://github.com/EionRobb/purple-teams) and Purple Teams for clearing up a lot about how authentication works.
